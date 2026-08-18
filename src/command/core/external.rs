use crate::engine::call::Call;
use crate::engine::command::category::Category;
use crate::engine::command::signature::Signature;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::engine_state::EngineState;
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use std::io::ErrorKind;

pub struct External;

impl Command for External {
    fn name(&self) -> &'static str {
        "external"
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
        if call.cmd.item.is_empty() {
            return Ok(CommandData::ExitCode(ExitCode::FAILURE));
        }

        let mut command_binding = std::process::Command::new(&call.cmd.item);
        let command = command_binding
            .stdin(io_streams.input.into_stdio())
            .stdout(io_streams.output.into_stdio())
            .stderr(io_streams.error.into_stdio());

        match command
            .args(call.raw_args.iter().map(|s| s.item.as_str()))
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
                    call.cmd.item
                ),
                label: format!("Command `{}` not found", call.cmd.item),
                span: call.cmd.span,
            }),
            Err(e) => Err(ShellError::ExternalCommand {
                help: String::new(),
                label: format!("{e}"),
                span: call.cmd.span,
            }),
        }
    }
}
