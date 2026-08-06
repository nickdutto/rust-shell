use crate::config::Config;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::parser::lexer::lex;
use crate::parser::span::Spanned;
use crate::shell::shell_state::ShellState;
use nu_ansi_term::{Color, Style};
use reedline::StyledText;
use std::io::Write;
use std::sync::{Arc, RwLock};

pub struct Lex;

impl Command for Lex {
    fn name(&self) -> &'static str {
        "lex"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn run(
        &self,
        cmd: Spanned<String>,
        args: Vec<Spanned<String>>,
        _job_id: Option<usize>,
        _config: Arc<Config>,
        _shell_state: Arc<RwLock<ShellState>>,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        let mut args_iter = args.iter();
        let mut pretty_print = false;

        let Some(line_arg) = args_iter.next() else {
            return Err(ShellError::CommandArgument {
                span: cmd.span,
                help: "Add line to be lexed inside quotes. Example: 'echo \"hello \\\"world\\\"\" && ls -la'"
                    .to_owned(),
                label: "Empty `line` argument".to_owned(),
            });
        };

        for arg in args_iter {
            if arg.item.as_str() == "-pretty" {
                pretty_print = true;
            }
        }

        let tokens = lex(&line_arg.item);
        let formatted = if pretty_print {
            format!("{tokens:#?}")
        } else {
            format!("{tokens:?}")
        };

        let styled = highlight_debug(&formatted);
        writeln!(io_streams.output, "{}", styled.render_simple())?;

        Ok(CommandData::ExitCode(ExitCode::SUCCESS))
    }
}

fn highlight_debug(debug_str: &str) -> StyledText {
    let mut chars = debug_str.chars().peekable();
    let mut buffer = StyledText::new();
    let style = Style::new();

    while let Some(&ch) = chars.peek() {
        match ch {
            '[' | ']' | '{' | '}' | '(' | ')' | ',' | ':' => {
                buffer.push((style.dimmed(), ch.to_string()));
                chars.next();
            }

            '"' => {
                buffer.push((style.fg(Color::Fixed(130)), ch.to_string()));
                chars.next();

                let mut escaped = false;
                while let Some(&c) = chars.peek() {
                    buffer.push((style.fg(Color::Fixed(130)), c.to_string()));
                    chars.next();

                    if escaped {
                        escaped = false;
                    } else if c == '\\' {
                        escaped = true;
                    } else if c == '"' {
                        break;
                    }
                }
            }

            _ if ch.is_numeric() => {
                buffer.push((style.fg(Color::Fixed(61)), ch.to_string()));
                chars.next();
            }

            _ if ch.is_uppercase() => {
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_alphabetic() {
                        buffer.push((style.fg(Color::Fixed(106)), c.to_string()));
                        chars.next();
                    } else {
                        break;
                    }
                }
            }

            _ => {
                buffer.push((style.fg(Color::Fixed(137)), ch.to_string()));
                chars.next();
            }
        }
    }

    buffer
}
