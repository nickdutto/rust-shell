use crate::config::Config;
use crate::engine::command::{Command, CommandData, CommandError, CommandType};
use crate::engine::exit::ExitCode;
use crate::io::stream::IoStreams;
use crate::shell::shell_state::ShellState;
use comfy_table::presets::NOTHING;
use comfy_table::{Attribute, Cell, Table};
use jiff::tz::TimeZone;
use jiff::{Timestamp, Zoned};
use std::fmt::Write as FmtWrite;
use std::io::Write;
use std::sync::{Arc, RwLock};

enum OutputMode {
    Table,
    List,
}

#[derive(PartialEq)]
enum SortMode {
    Alphabetical,
    Offset,
}

pub struct Timezone;

impl Command for Timezone {
    fn name(&self) -> &'static str {
        "timezone"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn run(
        &self,
        _cmd: &str,
        args: Vec<String>,
        _job_id: Option<usize>,
        _config: Arc<Config>,
        _shell_state: Arc<RwLock<ShellState>>,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, CommandError> {
        let mut final_exit_code = ExitCode::SUCCESS;
        let mut tz_timestamps = vec![];
        let mut output_mode = OutputMode::Table;
        let mut sort_mode = Some(SortMode::Alphabetical);
        let now = Timestamp::now();

        for arg in &args {
            match arg.as_str() {
                "-t" => {
                    output_mode = OutputMode::Table;
                }
                "-l" => {
                    output_mode = OutputMode::List;
                }
                "-sort-a" => {
                    sort_mode = Some(SortMode::Alphabetical);
                }
                "-sort-o" => {
                    sort_mode = Some(SortMode::Offset);
                }
                "-no-sort" => {
                    sort_mode = None;
                }
                _ => {
                    let tz = match TimeZone::get(arg) {
                        Ok(t) => t,
                        Err(e) => {
                            writeln!(io_streams.error, "{e}")?;
                            final_exit_code = ExitCode::FAILURE;
                            continue;
                        }
                    };

                    tz_timestamps.push((arg.clone(), now.to_zoned(tz)));
                }
            }
        }

        if !tz_timestamps.is_empty() {
            match sort_mode {
                Some(SortMode::Alphabetical) => tz_timestamps.sort_by(|a, b| a.0.cmp(&b.0)),
                Some(SortMode::Offset) => {
                    tz_timestamps
                        .sort_by(|a, b| a.1.offset().cmp(&b.1.offset()).then(a.0.cmp(&b.0)));
                }
                None => {}
            }

            match output_mode {
                OutputMode::Table => {
                    writeln!(io_streams.output, "{}", Self::table_output(tz_timestamps))?;
                }
                OutputMode::List => {
                    write!(io_streams.output, "{}", Self::list_output(&tz_timestamps))?;
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
