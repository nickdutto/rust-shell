use crate::command::BUILTIN_COMMANDS;
use crate::shell::shell_state::ShellState;
use crate::shell::variables::Variables;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::{CmdKind, Highlighter};
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use std::borrow::Cow;
use std::sync::{Arc, RwLock};

pub struct ShellHelper {
    builtin_commands: Vec<&'static str>,
    path_executables: Arc<RwLock<Vec<String>>>,
    shell_state: Arc<RwLock<ShellState>>,
}

impl ShellHelper {
    pub fn new(
        path_executables: Arc<RwLock<Vec<String>>>,
        shell_state: Arc<RwLock<ShellState>>,
    ) -> Self {
        ShellHelper {
            builtin_commands: BUILTIN_COMMANDS.to_vec(),
            path_executables,
            shell_state,
        }
    }
}

impl Helper for ShellHelper {}
impl Hinter for ShellHelper {
    type Hint = String;
}
impl Highlighter for ShellHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        let mut output = String::new();
        let mut in_quotes = false;
        let mut quote_char = ' ';

        let mut chars = line.chars().peekable();
        while let Some(ch) = chars.next() {
            match ch {
                '"' | '\'' if !in_quotes || quote_char == ch => {
                    if in_quotes {
                        output.push_str(&format!("{ch}\x1b[0m"));
                        in_quotes = false;
                    } else {
                        output.push_str(&format!("\x1b[32m{ch}"));
                        in_quotes = true;
                        quote_char = ch;
                    }
                }

                _ if in_quotes => {
                    output.push(ch);
                }

                '$' => {
                    output.push_str("\x1b[31m$");

                    if chars.peek() == Some(&'{') {
                        output.extend(chars.next());

                        let mut var_buffer = String::new();
                        while let Some(&var_ch) = chars.peek() {
                            var_buffer.extend(chars.next());
                            if var_ch == '}' {
                                break;
                            }
                        }

                        let key_name = var_buffer.strip_suffix('}').unwrap_or(&var_buffer);

                        if Variables::validate_key(key_name) {
                            output.push_str(&var_buffer);
                            output.push_str("\x1b[0m");
                        } else {
                            output.push_str("\x1b[1;38;2;255;45;85m");
                            output.push_str(&var_buffer);
                            output.push_str("\x1b[0m");
                        }
                    } else {
                        let mut var_buffer = String::new();
                        while let Some(&var_ch) = chars.peek() {
                            if !var_ch.is_whitespace() {
                                var_buffer.extend(chars.next());
                            } else if !(var_ch.is_ascii_alphanumeric() || var_ch == '_') {
                                break;
                            }
                        }

                        if Variables::validate_key(&var_buffer) {
                            output.push_str(&var_buffer);
                            output.push_str("\x1b[0m");
                        } else {
                            output.push_str("\x1b[1;38;2;255;45;85m");
                            output.push_str(&var_buffer);
                            output.push_str("\x1b[0m");
                        }
                    }
                }

                _ => {
                    output.push(ch);
                }
            }
        }

        output.push_str("\x1b[0m");
        Cow::Owned(output)
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _kind: CmdKind) -> bool {
        true
    }
}

impl Validator for ShellHelper {}
impl Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let mut pos = pos;
        let mut candidates = Vec::new();

        let partial_input = &line[..pos];
        if partial_input.is_empty() {
            return Ok((pos, candidates));
        }

        {
            let guard = self.shell_state.read().unwrap();

            if !partial_input.contains(' ') {
                guard.completions.complete_command(
                    partial_input,
                    &mut pos,
                    &mut candidates,
                    &self.builtin_commands,
                    &self.path_executables,
                );
            } else {
                let specification_found = guard.completions.complete_specification(
                    line,
                    partial_input,
                    &mut pos,
                    &mut candidates,
                );
                if !specification_found {
                    guard
                        .completions
                        .complete_filename(partial_input, &mut pos, &mut candidates);
                }
            }
        }

        candidates.sort_by(|a, b| a.replacement.cmp(&b.replacement));
        candidates.dedup_by(|a, b| a.replacement == b.replacement);

        Ok((pos, candidates))
    }
}

impl ShellHelper {}
