use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::engine_state::EngineState;
use crate::engine::exit::ExitCode;
use crate::engine::signature::Signature;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::parser::argument::ParsedArguments;
use crate::parser::span::Spanned;
use crate::parser::syntax_shape::SyntaxShape;
use crate::parser::value::Value;
use comfy_table::presets::NOTHING;
use comfy_table::{Attribute, Cell, Table};
use jiff::tz::TimeZone;
use jiff::{Timestamp, Zoned};
use std::fmt::Write as FmtWrite;
use std::io::Write;

#[derive(Clone)]
enum Format {
    Table,
    List,
}

#[derive(Clone, PartialEq)]
enum Sort {
    Alphabetical,
    Offset,
    Input,
}

struct NamedSpec<T> {
    name: &'static str,
    description: &'static str,
    short: Option<char>,
    mode: T,
}

const FORMAT_SPECS: &[NamedSpec<Format>] = &[
    NamedSpec {
        name: "list",
        description: "List format",
        short: Some('l'),
        mode: Format::List,
    },
    NamedSpec {
        name: "table",
        description: "Table format",
        short: Some('t'),
        mode: Format::Table,
    },
];

const SORT_SPECS: &[NamedSpec<Sort>] = &[
    NamedSpec {
        name: "sort-a",
        description: "Sort by alphabetical",
        short: None,
        mode: Sort::Alphabetical,
    },
    NamedSpec {
        name: "sort-o",
        description: "Sort by offset",
        short: None,
        mode: Sort::Offset,
    },
    NamedSpec {
        name: "sort-i",
        description: "sort by input order",
        short: None,
        mode: Sort::Input,
    },
];

pub struct Timezone;

impl Command for Timezone {
    fn name(&self) -> &'static str {
        "timezone"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn signature(&self) -> Signature {
        let mut signature = Signature::new(self.name());

        for spec in FORMAT_SPECS {
            signature = signature.switch(spec.name, spec.description, spec.short);
        }

        for spec in SORT_SPECS {
            signature = signature.switch(spec.name, spec.description, spec.short);
        }

        signature.rest(
            "timezones",
            SyntaxShape::String,
            "One or more IANA timezone identifiers",
        )
    }

    fn run(
        &self,
        _cmd: Spanned<String>,
        args: ParsedArguments,
        _job_id: Option<usize>,
        _engine_state: &EngineState,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        let mut final_exit_code = ExitCode::SUCCESS;
        let mut tz_timestamps = vec![];
        let mut format = Format::Table;
        let mut sort = Sort::Input;
        let now = Timestamp::now();

        for spec in FORMAT_SPECS {
            if args.has_switch(spec.name) {
                format = spec.mode.clone();
            }
        }

        for spec in SORT_SPECS {
            if args.has_switch(spec.name) {
                sort = spec.mode.clone();
            }
        }

        for arg in args.rest {
            if let Value::String(spanned_tz) = arg {
                let timezone_name = &spanned_tz.item;
                let timezone = match TimeZone::get(timezone_name) {
                    Ok(tz) => tz,
                    Err(err) => {
                        writeln!(io_streams.error, "{err}")?;
                        final_exit_code = ExitCode::FAILURE;
                        continue;
                    }
                };

                tz_timestamps.push((timezone_name.clone(), now.to_zoned(timezone)));
            }
        }

        if !tz_timestamps.is_empty() {
            match sort {
                Sort::Alphabetical => tz_timestamps.sort_by(|a, b| a.0.cmp(&b.0)),
                Sort::Offset => {
                    tz_timestamps
                        .sort_by(|a, b| a.1.offset().cmp(&b.1.offset()).then(a.0.cmp(&b.0)));
                }
                Sort::Input => {}
            }

            match format {
                Format::List => {
                    write!(io_streams.output, "{}", Self::list_output(&tz_timestamps))?;
                }
                Format::Table => {
                    writeln!(io_streams.output, "{}", Self::table_output(tz_timestamps))?;
                }
            }
        }

        Ok(CommandData::ExitCode(final_exit_code))
    }
}

impl Timezone {
    fn format_tz_abbreviation(ts: &Zoned) -> String {
        ts.strftime("%Z").to_string()
    }

    fn format_ts(ts: &Zoned) -> String {
        ts.strftime("%Y-%m-%dT%H:%M:%S%:z").to_string()
    }

    fn table_output(tz_timestamps: Vec<(String, Zoned)>) -> Table {
        let mut table = Table::new();

        table.load_preset(NOTHING);
        table.set_header(vec![
            Cell::new("TZ").add_attribute(Attribute::Bold),
            Cell::new("TS").add_attribute(Attribute::Bold),
            Cell::new("TZA").add_attribute(Attribute::Bold),
        ]);

        for (tz, ts) in tz_timestamps {
            table.add_row(vec![
                tz,
                Self::format_ts(&ts),
                Self::format_tz_abbreviation(&ts),
            ]);
        }

        table
    }

    fn list_output(tz_timestamps: &[(String, Zoned)]) -> String {
        tz_timestamps
            .iter()
            .fold(String::new(), |mut buffer, (tz, ts)| {
                let _ = writeln!(
                    buffer,
                    "{tz}: {} [{}]",
                    Self::format_ts(ts),
                    Self::format_tz_abbreviation(ts),
                );
                buffer
            })
    }
}
