use nu_ansi_term::{Color, Style};
use reedline::StyledText;

pub fn highlight_debug(debug_str: &str) -> StyledText {
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
