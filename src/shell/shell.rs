use crate::command::Command;
use crate::shell::shell_helper::ShellHelper;
use crate::system::env::get_env_path_executables;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::{CompletionType, Editor};
use std::io::{stderr, stdout};
use std::sync::{Arc, RwLock};
use std::{process, thread};

pub struct Shell;

impl Shell {
    pub fn start_session() {
        let executable_completions = Arc::new(RwLock::new(Vec::new()));
        let executable_completions_bg = Arc::clone(&executable_completions);

        thread::spawn(move || {
            let executables = get_env_path_executables("PATH");
            if let Ok(mut guard) = executable_completions_bg.write() {
                *guard = executables;
            }
        });

        let mut rl = Editor::new().unwrap();
        rl.set_helper(Some(ShellHelper::new(executable_completions)));
        rl.set_completion_type(CompletionType::List);

        loop {
            let readline = rl.readline("$ ");
            match readline {
                Ok(input) => {
                    if input.trim().is_empty() {
                        continue;
                    }

                    rl.add_history_entry(input.as_str()).ok();

                    Command::run_command(
                        Command::parse_command(&input),
                        &mut stdout(),
                        &mut stderr(),
                    );
                }
                Err(ReadlineError::Interrupted) => {
                    process::exit(0);
                }
                Err(ReadlineError::Eof) => {
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
    }
}
