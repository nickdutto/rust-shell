use crate::config::Config;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::signature::Signature;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::parser::argument::ParsedArguments;
use crate::parser::shape::SyntaxShape;
use crate::parser::span::Spanned;
use crate::shell::shell_state::ShellState;
use std::io::Write;
use std::process;
use std::sync::{Arc, RwLock};

pub struct Exit;

impl Command for Exit {
    fn name(&self) -> &'static str {
        "exit"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name()).positional(
            "exit_code",
            SyntaxShape::Int,
            "Exit code to return with",
        )
    }

    fn run(
        &self,
        _cmd: Spanned<String>,
        args: ParsedArguments,
        _job_id: Option<usize>,
        _config: Arc<Config>,
        shell_state: Arc<RwLock<ShellState>>,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        let exit_code = args.opt(0)?.unwrap_or(0) as i32;

        if let Err(e) = shell_state
            .write()
            .unwrap()
            .history
            .exit_save_history_file()
        {
            writeln!(io_streams.error, "{e}")?;
        }

        let _ = io_streams.output.flush();
        let _ = io_streams.error.flush();

        process::exit(exit_code);
    }
}
