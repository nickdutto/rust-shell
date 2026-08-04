use crate::config::Config;
use crate::shell::variables::Variables;
use nu_ansi_term::{Color, Style};
use reedline::{Highlighter, StyledText};
use std::iter::Peekable;
use std::str::Chars;
use std::sync::Arc;

pub struct SyntaxHighlighter {
    builtin_commands: &'static [&'static str],
    config: Arc<Config>,
}

impl Highlighter for SyntaxHighlighter {
    fn highlight(&self, line: &str, _cursor: usize) -> StyledText {
        self.highlight_tokens(line)
    }
}

impl SyntaxHighlighter {
    pub fn new(config: Arc<Config>, builtin_commands: &'static [&'static str]) -> Self {
        Self {
            builtin_commands,
            config,
        }
    }

    pub fn highlight_tokens(&self, line: &str) -> StyledText {
        let mut chars = line.chars().peekable();
        let mut buffer = StyledText::new();
        let mut in_quotes = false;
        let mut quote_char = ' ';

        let base_style = Style::new().fg(match self.config.theme.colors.input_base {
            Some(color) => Color::Fixed(color),
            None => Color::Default,
        });

        while let Some(ch) = chars.next() {
            match ch {
                '"' | '\'' => {
                    self.highlight_quote(&mut buffer, &mut in_quotes, &mut quote_char, ch);
                }
                _ if in_quotes => {
                    self.highlight_quote(&mut buffer, &mut in_quotes, &mut quote_char, ch);
                }

                '&' => {
                    self.highlight_ampersand(&mut buffer, &mut chars, ch);
                }

                ';' => {
                    self.highlight_sequential(&mut buffer, ch);
                }

                '|' => {
                    self.highlight_pipe(&mut buffer, ch);
                }

                '$' => self.highlight_variable(&mut buffer, &mut chars, ch),

                '>' | '1' | '2' => {
                    self.highlight_redirection(&mut buffer, &mut chars, ch, base_style);
                }

                _ => self.highlight_command(&mut buffer, &mut chars, ch, base_style),
            }
        }

        buffer
    }

    fn highlight_quote(
        &self,
        buffer: &mut StyledText,
        in_quotes: &mut bool,
        quote_char: &mut char,
        ch: char,
    ) {
        let quote_style_color = |ch| match ch {
            '"' => self.config.theme.colors.double_quote_strings,
            '\'' => self.config.theme.colors.single_quote_strings,
            _ => self.config.theme.colors.double_quote_strings,
        };

        let mut push_ch = |quote_ch: &char| {
            buffer.push((
                Style::new().fg(Color::Fixed(quote_style_color(*quote_ch))),
                ch.to_string(),
            ));
        };

        if matches!(ch, '"' | '\'') {
            push_ch(quote_char);
            return;
        }

        if !*in_quotes {
            *in_quotes = true;
            *quote_char = ch;
        } else if *in_quotes && ch == *quote_char {
            *in_quotes = false;
            *quote_char = ch;
        }

        push_ch(quote_char);
    }

    fn highlight_ampersand(&self, buffer: &mut StyledText, chars: &mut Peekable<Chars>, ch: char) {
        let is_background = chars.peek() == Some(&'&');
        let color = if is_background {
            self.config.theme.colors.background
        } else {
            self.config.theme.colors.and
        };

        if is_background {
            buffer.push((Style::new().fg(Color::Fixed(color)), ch.to_string()));
            chars.next();
        }
        buffer.push((Style::new().fg(Color::Fixed(color)), ch.to_string()));
    }

    fn highlight_sequential(&self, buffer: &mut StyledText, ch: char) {
        buffer.push((
            Style::new().fg(Color::Fixed(self.config.theme.colors.sequential)),
            ch.to_string(),
        ));
    }

    fn highlight_pipe(&self, buffer: &mut StyledText, ch: char) {
        buffer.push((
            Style::new().fg(Color::Fixed(self.config.theme.colors.pipe)),
            ch.to_string(),
        ));
    }

    fn highlight_variable(&self, buffer: &mut StyledText, chars: &mut Peekable<Chars>, ch: char) {
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
            } else if in_braces || !var_ch.is_whitespace() {
                var_buffer.extend(chars.next());
            } else if !(var_ch.is_ascii_alphanumeric() || var_ch == '_') {
                break;
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

        let style_color = match Variables::validate_key(var_key) {
            true => self.config.theme.colors.variable,
            false => self.config.theme.colors.variable_invalid,
        };

        buffer.push((Style::new().fg(Color::Fixed(style_color)), var_buffer));
    }

    fn highlight_redirection(
        &self,
        buffer: &mut StyledText,
        chars: &mut Peekable<Chars>,
        ch: char,
        base_style: Style,
    ) {
        let mut redirection_buffer = String::new();

        let mut append_mode = false;
        let peek_redirection_char = |c: Option<&char>| {
            if let Some(&'>') = c {
                return true;
            }
            false
        };

        if ch == '>' {
            redirection_buffer.push(ch);
            if peek_redirection_char(chars.peek()) {
                redirection_buffer.extend(chars.next());
                append_mode = true;
            }
        } else if peek_redirection_char(chars.peek()) {
            redirection_buffer.push(ch);
            redirection_buffer.extend(chars.next());

            if peek_redirection_char(chars.peek()) {
                redirection_buffer.extend(chars.next());
                append_mode = true;
            }
        } else {
            buffer.push((base_style, ch.to_string()));
        }

        let style_color = match (ch, append_mode) {
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

    fn highlight_command(
        &self,
        buffer: &mut StyledText,
        chars: &mut Peekable<Chars>,
        ch: char,
        base_style: Style,
    ) {
        let mut matched_command = false;
        let current_word = format!("{}{}", ch, chars.clone().collect::<String>());

        for command in self.builtin_commands {
            if current_word.starts_with(command) {
                let next_ch = current_word.chars().nth(command.chars().count());
                if next_ch.is_none_or(char::is_whitespace) {
                    buffer.push((
                        Style::new().fg(Color::Fixed(self.config.theme.colors.builtin_command)),
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
