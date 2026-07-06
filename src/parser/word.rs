use crate::parser::token_scanner::TokenScanner;

#[derive(Debug, PartialEq)]
pub enum Word {
    Literal(String),
    SingleQuoted(String),
    DoubleQuoted(String),
    Variable(String),
}

pub fn scan_word(scanner: &mut TokenScanner, initial: Option<char>) -> Vec<Word> {
    let mut word = vec![];
    let mut initial_content = initial.map(|c| c.to_string());

    while let Some(&ch) = scanner.peek() {
        if matches!(ch, ' ' | '\t' | '&' | '|' | ';' | '>') {
            break;
        }

        match ch {
            '\'' => {
                let single_quoted = scan_single_quoted(scanner, initial_content.take());
                word.push(single_quoted);
            }

            '"' => {
                let double_quoted = scan_double_quoted(scanner, initial_content.take());
                word.push(double_quoted);
            }

            // TODO: handle positional and special parameters
            '$' => {
                let variable = scan_variable(scanner, initial_content.take());
                word.push(variable);
            }

            _ => {
                let literal = scan_literal(scanner, initial_content.take());
                word.push(literal);
            }
        }
    }

    word
}

fn scan_single_quoted(scanner: &mut TokenScanner, initial_content: Option<String>) -> Word {
    let mut content = initial_content.unwrap_or_default();

    scanner.next_char();

    while let Some(ch) = scanner.next_char() {
        if ch == '\'' {
            break;
        }
        content.push(ch);
    }

    Word::SingleQuoted(content)
}

fn scan_double_quoted(scanner: &mut TokenScanner, initial_content: Option<String>) -> Word {
    let mut content = initial_content.unwrap_or_default();

    scanner.next_char();

    while let Some(&ch) = scanner.peek() {
        if ch == '"' {
            scanner.next_char();
            break;
        }

        let Some(current_ch) = scanner.next_char() else {
            break;
        };

        if current_ch != '\\' {
            content.push(current_ch);
            continue;
        }

        match scanner.peek() {
            Some('\n') => {
                scanner.next_char();
            }
            Some('$' | '`' | '"' | '\\') => {
                content.extend(scanner.next_char());
            }
            _ => {
                content.push('\\');
                content.extend(scanner.next_char());
            }
        }
    }

    Word::DoubleQuoted(content)
}

fn scan_variable(scanner: &mut TokenScanner, initial_content: Option<String>) -> Word {
    let mut content = initial_content.unwrap_or_default();

    scanner.next_char();
    scanner.next_if_matches('{');

    while let Some(&ch) = scanner.peek() {
        // TODO: Handle full expected variable format
        if ch.is_ascii_alphanumeric() || ch == '_' {
            content.extend(scanner.next_char());
        } else {
            break;
        }
    }

    scanner.next_if_matches('}');

    Word::Variable(content)
}

fn scan_literal(scanner: &mut TokenScanner, initial_content: Option<String>) -> Word {
    let mut content = initial_content.unwrap_or_default();

    while let Some(&ch) = scanner.peek() {
        if matches!(
            ch,
            '\'' | '"' | '$' | '`' | ' ' | '\t' | '&' | '|' | ';' | '>'
        ) {
            break;
        }

        let current_ch = scanner.next_char();
        if current_ch == Some('\\') {
            content.extend(scanner.next_char());
        } else {
            content.extend(current_ch);
        }
    }

    Word::Literal(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_single_quoted_returns_on_single_quote_char() {
        let mut scanner = TokenScanner::new("'hello'w'orld'");
        assert_eq!(
            scan_single_quoted(&mut scanner, None),
            Word::SingleQuoted("hello".into())
        )
    }

    #[test]
    fn scan_double_quoted_returns_on_double_quote_char() {
        let mut scanner = TokenScanner::new("\"hello\"w\"orld\"");
        assert_eq!(
            scan_double_quoted(&mut scanner, None),
            Word::DoubleQuoted("hello".into())
        )
    }

    #[test]
    fn scan_double_quoted_handles_escaping() {
        let mut scanner = TokenScanner::new("\"hello \\$ \\` \\\" \\\\ \\\n world\"");
        assert_eq!(
            scan_double_quoted(&mut scanner, None),
            Word::DoubleQuoted("hello $ ` \" \\  world".into())
        )
    }

    #[test]
    fn scan_variable_handles_no_braces() {
        let mut scanner = TokenScanner::new("$abc");
        assert_eq!(
            scan_variable(&mut scanner, None),
            Word::Variable("abc".into())
        )
    }

    #[test]
    fn scan_variable_handles_with_braces() {
        let mut scanner = TokenScanner::new("${abc}");
        assert_eq!(
            scan_variable(&mut scanner, None),
            Word::Variable("abc".into())
        )
    }
}
