use crate::engine::call::Call;
use crate::engine::command::category::Category;
use crate::engine::command::signature::Signature;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::engine_state::EngineState;
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::value::syntax_shape::SyntaxShape;
use jiff::Zoned;
use std::io::Write;

struct NamedSpec {
    name: &'static str,
    description: &'static str,
    short: Option<char>,
    format: &'static str,
}

const NAMED_SPECS: &[NamedSpec] = &[
    NamedSpec {
        name: "day",
        description: "day",
        short: Some('d'),
        format: "%d",
    },
    NamedSpec {
        name: "month",
        description: "month",
        short: Some('m'),
        format: "%m",
    },
    NamedSpec {
        name: "year",
        description: "year",
        short: Some('y'),
        format: "%Y",
    },
    NamedSpec {
        name: "hour",
        description: "hour",
        short: None,
        format: "%H",
    },
    NamedSpec {
        name: "minutes",
        description: "minutes",
        short: None,
        format: "%M",
    },
    NamedSpec {
        name: "seconds",
        description: "seconds",
        short: None,
        format: "%S",
    },
    NamedSpec {
        name: "full-day",
        description: "full day name",
        short: Some('a'),
        format: "%A",
    },
    NamedSpec {
        name: "full-month",
        description: "full month name",
        short: Some('b'),
        format: "%B",
    },
    NamedSpec {
        name: "iana",
        description: "IANA timezone identifier",
        short: Some('q'),
        format: "%Q",
    },
    NamedSpec {
        name: "offset",
        description: "timezone offset [+-]HH:MM[:SS]",
        short: Some('o'),
        format: "%z",
    },
    NamedSpec {
        name: "tz",
        description: "timezone offset [+-]HHMM[SS]",
        short: None,
        format: "%Z",
    },
    NamedSpec {
        name: "iso",
        description: "iso datetime",
        short: None,
        format: "%Y-%m-%dT%H:%M:%S%:z",
    },
    NamedSpec {
        name: "date",
        description: "only date (dd/mm/yy)",
        short: None,
        format: "%d/%m/%y",
    },
    NamedSpec {
        name: "time",
        description: "only time (HH:MM:SS)",
        short: None,
        format: "%T",
    },
];

pub struct Now;

impl Command for Now {
    fn name(&self) -> &'static str {
        "now"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn signature(&self) -> Signature {
        let mut signature = Signature::new(self.name()).category(Category::Date);

        for spec in NAMED_SPECS {
            signature = signature.switch(spec.name, spec.description, spec.short);
        }

        signature.switch("raw", "raw datetime", None).named(
            "f",
            SyntaxShape::String,
            "custom strftime format",
            Some('f'),
        )
    }

    fn run(
        &self,
        call: Call,
        _engine_state: &EngineState,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        let now = Zoned::now();

        if call.is_args_empty() || call.has_switch("raw") {
            writeln!(io_streams.output, "{now}")?;
        }

        for spec in NAMED_SPECS {
            if call.has_switch(spec.name) {
                writeln!(io_streams.output, "{}", now.strftime(spec.format))?;
            }
        }

        if let Some(format) = call.opt_named::<String>("f")? {
            match jiff::fmt::strtime::format(format, &now) {
                Ok(f) => writeln!(io_streams.output, "{f}")?,
                Err(e) => {
                    writeln!(io_streams.error, "now: {e}")?;
                    return Ok(CommandData::ExitCode(ExitCode::FAILURE));
                }
            }
        }

        Ok(CommandData::ExitCode(ExitCode::SUCCESS))
    }
}
