use crate::engine::call::Call;
use crate::engine::command::category::Category;
use crate::engine::command::signature::Signature;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::engine_state::EngineState;
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use std::io::Write;

pub struct Jobs;

impl Command for Jobs {
    fn name(&self) -> &'static str {
        "jobs"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name())
            .category(Category::Process)
            .switch("table", "Table format", Some('t'))
    }

    fn run(
        &self,
        call: Call,
        engine_state: &EngineState,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        if call.has_switch("table") {
            let table = engine_state
                .shell_state
                .read()
                .unwrap()
                .background_jobs
                .to_table();

            writeln!(io_streams.output, "{table}")?;
        } else {
            let jobs_list = engine_state
                .shell_state
                .read()
                .unwrap()
                .background_jobs
                .to_list_string(None);

            if !jobs_list.is_empty() {
                writeln!(io_streams.output, "{jobs_list}")?;
            }
        }

        engine_state
            .shell_state
            .write()
            .unwrap()
            .background_jobs
            .remove_done_jobs();

        Ok(CommandData::ExitCode(ExitCode::SUCCESS))
    }
}
