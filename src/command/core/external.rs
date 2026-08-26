use crate::engine::call::Call;
use crate::engine::command::category::Category;
use crate::engine::command::signature::Signature;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::engine_state::EngineState;
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use std::io::ErrorKind;
use std::os::unix::process::CommandExt;

pub struct External;

impl Command for External {
    fn name(&self) -> &'static str {
        "external"
    }

    fn description(&self) -> &'static str {
        "Executes an external command. Called automatically if a builtin doesn't exist for the command name.\n\
        Also allows explicitly bypassing a builtin using `external cmd_name`."
    }

    fn command_type(&self) -> CommandType {
        CommandType::External
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name())
            .category(Category::Core)
            .allow_unknown_args(true)
    }

    fn run(
        &self,
        call: Call,
        engine_state: &EngineState,
        io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        let mut raw_args = call.raw_args;
        let cmd = if call.cmd.item == self.name() {
            raw_args.remove(0)
        } else {
            call.cmd
        };

        if cmd.item.is_empty() {
            return Ok(CommandData::ExitCode(ExitCode::FAILURE));
        }

        let mut command_binding = std::process::Command::new(&cmd.item);
        let command = command_binding
            .stdin(io_streams.input.into_stdio())
            .stdout(io_streams.output.into_stdio())
            .stderr(io_streams.error.into_stdio());

        if call.job_id.is_some() {
            #[cfg(unix)]
            {
                command.process_group(0);
            }

            #[cfg(windows)]
            {
                const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
                command.creation_flags(CREATE_NEW_PROCESS_GROUP);
            }
        }

        match command
            .args(raw_args.iter().map(|s| s.item.as_str()))
            .spawn()
        {
            Ok(child) => {
                if let Some(jb_id) = call.job_id
                    && let Some(job) = engine_state
                        .shell_state
                        .write()
                        .unwrap()
                        .background_jobs
                        .iter_mut()
                        .find(|job| job.id() == jb_id)
                {
                    job.pids.push(child.id());
                }
                Ok(CommandData::Child(child))
            }
            Err(e) if e.kind() == ErrorKind::NotFound => Err(ShellError::ExternalCommand {
                help: format!(
                    "`{}` is neither a built-in nor a known external command",
                    cmd.item
                ),
                label: format!("Command `{}` not found", cmd.item),
                span: cmd.span,
            }),
            Err(e) => Err(ShellError::ExternalCommand {
                help: String::new(),
                label: format!("{e}"),
                span: cmd.span,
            }),
        }
    }
}
