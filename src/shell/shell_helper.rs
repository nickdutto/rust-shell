use crate::command::BUILTIN_COMMANDS;
use crate::shell::config::Config;
use crate::shell::shell_state::ShellState;
use crate::shell::variables::Variables;
use crate::system::env::get_env_path_executables;
use nu_ansi_term::{Color, Style};
use reedline::{Completer, Highlighter, StyledText, Suggestion};
use std::sync::{Arc, RwLock};
use std::thread;

#[derive(Clone)]
pub struct ShellHelper {
    builtin_commands: Vec<&'static str>,
    config: Config,
    path_executables: Arc<RwLock<Vec<String>>>,
    shell_state: Arc<RwLock<ShellState>>,
}

impl ShellHelper {
    pub fn new(config: Config, shell_state: Arc<RwLock<ShellState>>) -> Self {
        ShellHelper {
            builtin_commands: BUILTIN_COMMANDS.to_vec(),
            config,
            path_executables: ShellHelper::get_path_executables(),
            shell_state,
        }
    }

    fn get_path_executables() -> Arc<RwLock<Vec<String>>> {
        let path_executables = Arc::new(RwLock::new(Vec::new()));
        let path_executables_bg = Arc::clone(&path_executables);

        thread::spawn(move || {
            let executables = get_env_path_executables("PATH");
            if let Ok(mut guard) = path_executables_bg.write() {
                *guard = executables;
            }
        });

        path_executables
    }
}

impl Completer for ShellHelper {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        let partial_input = &line[..pos];
        if partial_input.is_empty() {
            return suggestions;
        }

        {
            let guard = self.shell_state.read().unwrap();

            if !partial_input.contains(' ') {
                guard.completions.complete_command(
                    partial_input,
                    pos,
                    &mut suggestions,
                    &self.builtin_commands,
                    &self.path_executables,
                );
            } else {
                let specification_found = guard.completions.complete_specification(
                    line,
                    partial_input,
                    pos,
                    &mut suggestions,
                );
                if !specification_found {
                    guard
                        .completions
                        .complete_filename(partial_input, pos, &mut suggestions);
                }
            }
        }

        suggestions.sort_by(|a, b| a.value.cmp(&b.value));
        suggestions.dedup_by(|a, b| a.value == b.value);

        suggestions
    }
}

impl Highlighter for ShellHelper {
    fn highlight(&self, line: &str, _cursor: usize) -> StyledText {
        let mut buffer = StyledText::new();
        let mut in_quotes = false;
        let mut quote_char = ' ';

        let base_style =
            Style::new().fg(if let Some(color) = self.config.theme.colors.input_base {
                Color::Fixed(color)
            } else {
                Color::Default
            });

        let quote_style = |ch| match ch {
            '"' => Style::new().fg(Color::Fixed(self.config.theme.colors.double_quote_strings)),
            '\'' => Style::new().fg(Color::Fixed(self.config.theme.colors.single_quote_strings)),
            _ => base_style,
        };

        let variable_style = Style::new().fg(Color::Fixed(self.config.theme.colors.variable));
        let variable_invalid_style =
            Style::new().fg(Color::Fixed(self.config.theme.colors.variable_invalid));

        let mut chars = line.chars().peekable();
        while let Some(ch) = chars.next() {
            match ch {
                '"' | '\'' => {
                    if !in_quotes {
                        in_quotes = true;
                        quote_char = ch;
                    } else if in_quotes && ch == quote_char {
                        in_quotes = false;
                        quote_char = ch;
                    }

                    buffer.push((quote_style(quote_char), ch.to_string()));
                }
                _ if in_quotes => {
                    buffer.push((quote_style(quote_char), ch.to_string()));
                }

                '|' => {
                    buffer.push((
                        Style::new().fg(Color::Fixed(self.config.theme.colors.pipe)),
                        ch.to_string(),
                    ));
                }

                redirection_op @ ('>' | '1' | '2') => {
                    let mut redirection_buffer = String::new();

                    let mut append_mode = false;
                    let peek_redirection_char = |c: Option<&char>| {
                        if let Some(&'>') = c {
                            return true;
                        }
                        false
                    };

                    if redirection_op == '>' {
                        redirection_buffer.push(ch);
                        if peek_redirection_char(chars.peek()) {
                            redirection_buffer.extend(chars.next());
                            append_mode = true;
                        }
                    } else {
                        if peek_redirection_char(chars.peek()) {
                            redirection_buffer.push(ch);
                            redirection_buffer.extend(chars.next());

                            if peek_redirection_char(chars.peek()) {
                                redirection_buffer.extend(chars.next());
                                append_mode = true;
                            }
                        } else {
                            buffer.push((base_style, ch.to_string()));
                        }
                    }

                    let style_color = match (redirection_op, append_mode) {
                        ('>' | '1', true) => self.config.theme.colors.redirection_out,
                        ('>' | '1', false) => self.config.theme.colors.redirection_out_append,
                        ('2', true) => self.config.theme.colors.redirection_error,
                        ('2', false) => self.config.theme.colors.redirection_error_append,
                        _ => self.config.theme.colors.redirection_out,
                    };

                    buffer.push((
                        Style::new().fg(Color::Fixed(style_color)),
                        redirection_buffer,
                    ));
                }

                '$' => {
                    let mut var_buffer = String::new();
                    let mut in_braces = false;
                    var_buffer.push(ch);

                    while let Some(&var_ch) = chars.peek() {
                        if var_ch == '{' {
                            in_braces = true;
                            var_buffer.extend(chars.next());
                        } else if var_ch == '}' {
                            var_buffer.extend(chars.next());
                            break;
                        } else if in_braces {
                            var_buffer.extend(chars.next());
                        } else {
                            if !var_ch.is_whitespace() {
                                var_buffer.extend(chars.next());
                            } else if !(var_ch.is_ascii_alphanumeric() || var_ch == '_') {
                                break;
                            }
                        }
                    }

                    let var_key = if in_braces {
                        var_buffer
                            .strip_prefix("${")
                            .and_then(|s| s.strip_suffix('}'))
                            .unwrap_or(&var_buffer)
                    } else {
                        var_buffer.strip_prefix('$').unwrap_or(&var_buffer)
                    };

                    if Variables::validate_key(var_key) {
                        buffer.push((variable_style, var_buffer));
                    } else {
                        buffer.push((variable_invalid_style, var_buffer));
                    }
                }

                _ => {
                    let mut matched_command = false;
                    let current_word = format!("{}{}", ch, chars.clone().collect::<String>());

                    for command in &self.builtin_commands {
                        if current_word.starts_with(command) {
                            let next_ch = current_word.chars().nth(command.chars().count());
                            if next_ch.is_none_or(|c| c.is_whitespace()) {
                                buffer.push((
                                    Style::new()
                                        .fg(Color::Fixed(self.config.theme.colors.builtin_command)),
                                    command.to_string(),
                                ));
                                if command.chars().count() > 1 {
                                    chars.nth(command.chars().count() - 2);
                                }
                                matched_command = true;
                                break;
                            }
                        }
                    }

                    if !matched_command {
                        buffer.push((base_style, ch.to_string()));
                    }
                }
            }
        }

        buffer
    }
}
