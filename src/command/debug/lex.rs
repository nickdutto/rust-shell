use crate::config::Config;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::exit::ExitCode;
use crate::engine::signature::Signature;
use crate::error::shell_error::ShellError;
use crate::format::debug::highlight_debug;
use crate::io::stream::IoStreams;
use crate::parser::argument::ParsedArguments;
use crate::parser::lexer::lex;
use crate::parser::shape::SyntaxShape;
use crate::parser::span::Spanned;
use crate::shell::shell_state::ShellState;
use std::io::Write;
use std::sync::{Arc, RwLock};

pub struct Lex;

impl Command for Lex {
    fn name(&self) -> &'static str {
        "lex"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name())
            .required_positional("line", SyntaxShape::String, "Line to build lex tokens from")
            .switch("pretty", "Pretty print", Some('p'))
    }

    fn run(
        &self,
        _cmd: Spanned<String>,
        args: ParsedArguments,
        _job_id: Option<usize>,
        _config: Arc<Config>,
        _shell_state: Arc<RwLock<ShellState>>,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        let line = args.req::<String>(0)?;
        let pretty_print = args.has_named("pretty");

        let tokens = lex(&line);
        let formatted = if pretty_print {
            format!("{tokens:#?}")
        } else {
            format!("{tokens:?}")
        };

        let styled = highlight_debug(&formatted);
        writeln!(io_streams.output, "{}", styled.render_simple())?;

        Ok(CommandData::ExitCode(ExitCode::SUCCESS))
    }
}
