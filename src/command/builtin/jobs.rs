use crate::config::Config;
use crate::engine::command::{Command, CommandData, CommandError, CommandType};
use crate::engine::exit::ExitCode;
use crate::io::stream::IoStreams;
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

    fn run(
        &self,
        _cmd: &str,
        args: Vec<String>,
        _job_id: Option<usize>,
        _config: Arc<Config>,
        shell_state: Arc<RwLock<ShellState>>,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, CommandError> {
        let mut executed = false;

        for arg in &args {
            if arg.as_str() == "-t" {
                let table = shell_state.read().unwrap().background_jobs.to_table();
                writeln!(io_streams.output, "{table}")?;
                executed = true;
            }
        }

        if !executed {
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
