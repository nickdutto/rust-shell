use crate::command::{BUILTIN_COMMANDS, Command};
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Editor, Helper};
use std::io::{stderr, stdout};
use std::process;

struct ShellHelper;
impl Helper for ShellHelper {}
impl Hinter for ShellHelper {
    type Hint = String;
}
impl Highlighter for ShellHelper {}
impl Validator for ShellHelper {}
impl Completer for ShellHelper {
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
            for command in BUILTIN_COMMANDS {
                if command.starts_with(partial_input) {
                    candidates.push(Pair {
                        display: command.to_string(),
                        replacement: format!("{} ", command),
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
        let mut rl = Editor::new().unwrap();
        rl.set_helper(Some(ShellHelper));

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
