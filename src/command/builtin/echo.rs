use crate::config::Config;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::exit::ExitCode;
use crate::engine::signature::Signature;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::parser::argument::ParsedArguments;
use crate::parser::span::Spanned;
use crate::parser::syntax_shape::SyntaxShape;
use crate::shell::shell_state::ShellState;
use std::io::Write as IoWrite;
use std::sync::{Arc, RwLock};

pub struct Echo;

impl Command for Echo {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name()).rest("message", SyntaxShape::String, "text to write")
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
        let message = args.rest_strings().collect::<Vec<_>>().join(" ");

        writeln!(io_streams.output, "{}", message)?;

        Ok(CommandData::ExitCode(ExitCode::SUCCESS))
    }
}
