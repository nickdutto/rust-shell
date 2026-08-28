use crate::engine::call::Call;
use crate::engine::command::category::Category;
use crate::engine::command::signature::Signature;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::engine_state::EngineState;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::value::syntax_shape::SyntaxShape;
use std::io::Write;
use std::process;

pub struct Exit;

impl Command for Exit {
    fn name(&self) -> &'static str {
        "exit"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name())
            .category(Category::Shell)
            .positional("exit_code", SyntaxShape::Int, "Exit code to return with")
    }

    fn run(
        &self,
        call: Call,
        engine_state: &EngineState,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        let exit_code = call
            .opt(0)?
            .map_or(0, |v: i64| i32::try_from(v).unwrap_or(0));

        if let Err(e) = engine_state
            .shell_state
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
