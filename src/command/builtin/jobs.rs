use crate::config::Config;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::exit::ExitCode;
use crate::engine::signature::Signature;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::parser::argument::ParsedArguments;
use crate::parser::span::Spanned;
use crate::shell::shell_state::ShellState;
use std::io::Write;
use std::sync::{Arc, RwLock};

pub struct Jobs;

impl Command for Jobs {
    fn name(&self) -> &'static str {
        "jobs"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name()).switch("table", "Table format", Some('t'))
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
        if args.has_switch("table") {
            let table = shell_state.read().unwrap().background_jobs.to_table();
            writeln!(io_streams.output, "{table}")?;
        } else {
            let jobs_list = shell_state
                .read()
                .unwrap()
                .background_jobs
                .to_list_string(None);

            if !jobs_list.is_empty() {
                writeln!(io_streams.output, "{jobs_list}")?;
            }
        }

        shell_state
            .write()
            .unwrap()
            .background_jobs
            .remove_done_jobs();

        Ok(CommandData::ExitCode(ExitCode::SUCCESS))
    }
}
