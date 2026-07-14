use crate::engine::command::{CommandData, CommandError};
use crate::engine::exit::ExitCode;
use std::process::Child;
use std::thread::JoinHandle;

pub enum ProcessHandle {
    External(Child),
    Thread(JoinHandle<i32>),
    Immediate(i32),
}

impl ProcessHandle {
    pub fn wait(self) -> i32 {
        match self {
            ProcessHandle::External(mut child) => {
                child.wait().ok().and_then(|s| s.code()).unwrap_or(1)
            }
            ProcessHandle::Thread(handle) => handle.join().unwrap_or(1),
            ProcessHandle::Immediate(code) => code,
        }
    }

    pub fn run_producer(
        f: Box<dyn FnOnce() -> Result<CommandData, CommandError> + Send>,
        needs_thread: bool,
    ) -> ProcessHandle {
        let fun = || {
            f().map_or(ExitCode::FAILURE, CommandData::into_exit_code)
                .as_i32()
        };

        if needs_thread {
            ProcessHandle::Thread(std::thread::spawn(fun))
        } else {
            ProcessHandle::Immediate(fun())
        }
    }
}
