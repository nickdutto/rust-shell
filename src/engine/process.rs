use crate::engine::command::CommandData;
use crate::engine::exit::ExitCode;
use crate::engine::signals::Signals;
use crate::error::shell_error::ShellError;
use crate::value::span::Span;
use std::os::unix::process::ExitStatusExt;
use std::process::Child;
use std::thread::JoinHandle;
use std::time::Duration;

pub enum ProcessHandle {
    External {
        child: Child,
        span: Span,
    },
    Thread {
        handle: JoinHandle<Result<ExitCode, ShellError>>,
        span: Span,
    },
    Immediate(Result<ExitCode, ShellError>),
}

impl ProcessHandle {
    pub fn wait(self, signals: &Signals) -> Result<ExitCode, ShellError> {
        match self {
            ProcessHandle::External { mut child, span } => loop {
                if signals.is_interrupted() {
                    _ = child.kill();
                    _ = child.wait();
                    return Err(ShellError::Interrupted { span });
                }

                match child.try_wait() {
                    Ok(Some(status)) => {
                        #[cfg(unix)]
                        {
                            if status.signal() == Some(2) || signals.is_interrupted() {
                                return Err(ShellError::Interrupted { span });
                            }
                        }

                        #[cfg(not(unix))]
                        {
                            if status.code() == Some(130) || signals.is_interrupted() {
                                return Err(ShellError::Interrupted { span });
                            }
                        }

                        return Ok(ExitCode::from(status));
                    }

                    Ok(None) => {}
                    Err(err) => return Err(ShellError::Io(err)),
                }

                std::thread::sleep(Duration::from_millis(5));
            },

            ProcessHandle::Thread { handle, span } => handle
                .join()
                .unwrap_or_else(|_| Err(ShellError::Interrupted { span })),

            ProcessHandle::Immediate(res) => res,
        }
    }

    pub fn run_producer(
        f: Box<dyn FnOnce() -> Result<CommandData, ShellError> + Send>,
        needs_thread: bool,
        span: Span,
    ) -> ProcessHandle {
        if needs_thread {
            let handle = std::thread::spawn(move || Ok(f()?.into_exit_code()));
            ProcessHandle::Thread { handle, span }
        } else {
            match f() {
                Ok(CommandData::Child(child)) => ProcessHandle::External { child, span },
                Ok(CommandData::ExitCode(code)) => ProcessHandle::Immediate(Ok(code)),
                Err(err) => ProcessHandle::Immediate(Err(err)),
            }
        }
    }
}
