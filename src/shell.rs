use crate::command::{BUILTIN_COMMANDS, Command};
use crate::env::get_env_path_executables;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Editor, Helper};
use std::io::{stderr, stdout};
use std::process;

struct ShellHelper<'a> {
    builtin_commands: Vec<&'a str>,
    path_executables: Vec<String>,
}

impl<'a> ShellHelper<'a> {
    pub fn new() -> Self {
        ShellHelper {
            builtin_commands: BUILTIN_COMMANDS.to_vec(),
            path_executables: get_env_path_executables("PATH"),
        }
    }
}

impl<'a> Helper for ShellHelper<'a> {}
impl<'a> Hinter for ShellHelper<'a> {
    type Hint = String;
}
impl<'a> Highlighter for ShellHelper<'a> {}
impl<'a> Validator for ShellHelper<'a> {}
impl<'a> Completer for ShellHelper<'a> {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let mut candidates = Vec::new();

        let partial_input = &line[..pos];
        if !partial_input.contains(' ') && !partial_input.is_empty() {
            for command in &self.builtin_commands {
                if command.starts_with(partial_input) {
                    candidates.push(Pair {
                        display: command.to_string(),
                        replacement: format!("{} ", command),
                    });
                }
            }

            for executable in &self.path_executables {
                if executable.starts_with(partial_input) {
                    candidates.push(Pair {
                        display: executable.to_string(),
                        replacement: format!("{} ", executable),
                    });
                }
            }

            return Ok((0, candidates));
        }

        Ok((pos, candidates))
    }
}

pub struct Shell;

impl Shell {
    pub fn start_session() {
        let shell_helper = ShellHelper::new();
        let mut rl = Editor::new().unwrap();
        rl.set_helper(Some(shell_helper));

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
