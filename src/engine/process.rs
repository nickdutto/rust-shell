use crate::engine::command::CommandData;
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use std::process::Child;
use std::thread::JoinHandle;

pub enum ProcessHandle {
    External(Child),
    Thread(JoinHandle<Result<ExitCode, ShellError>>),
    Immediate(Result<ExitCode, ShellError>),
}

impl ProcessHandle {
    pub fn wait(self) -> Result<ExitCode, ShellError> {
        match self {
            ProcessHandle::External(mut child) => {
                Ok(child.wait().ok().map_or(ExitCode::FAILURE, ExitCode::from))
            }
            ProcessHandle::Thread(handle) => handle.join().unwrap_or(Ok(ExitCode::FAILURE)),
            ProcessHandle::Immediate(res) => res,
        }
    }

    pub fn run_producer(
        f: Box<dyn FnOnce() -> Result<CommandData, ShellError> + Send>,
        needs_thread: bool,
    ) -> ProcessHandle {
        let fun = || Ok(f()?.into_exit_code());

        if needs_thread {
            ProcessHandle::Thread(std::thread::spawn(fun))
        } else {
            ProcessHandle::Immediate(fun())
        }
    }
}
