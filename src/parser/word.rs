use crate::parser::error::ParserError;
use crate::parser::span::{Span, Spanned};
use crate::parser::token_scanner::TokenScanner;
use crate::shell::variables::Variables;
use std::fmt::{Display, Formatter, Write};

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Word {
    Literal(String),
    SingleQuoted(String),
    DoubleQuoted(String),
    Variable(String),
    Error(String, ParserError),
    #[default]
    Nothing,
}

impl Word {
    pub fn to_original_string(&self) -> String {
        match self {
            Word::Literal(w) => w.clone(),
            Word::SingleQuoted(w) => format!("'{w}'"),
            Word::DoubleQuoted(w) => format!("\"{w}\""),
            // TODO: doesn't account for whether the original variable was open or braced
            Word::Variable(w) => format!("${w}"),
            Word::Error(w, _) => w.clone(),
            Word::Nothing => String::new(),
        }
    }
}

impl Display for Word {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Word::Literal(w)
            | Word::SingleQuoted(w)
            | Word::DoubleQuoted(w)
            | Word::Variable(w) => write!(f, "{w}"),
            Word::Error(w, _) => write!(f, "{w}"),
            Word::Nothing => write!(f, ""),
        }
    }
}

pub fn scan_word(scanner: &mut TokenScanner, mut initial_char: Option<char>) -> Vec<Spanned<Word>> {
    let mut words = vec![];

    if let Some(initial_ch) = initial_char
        && scanner.peek().is_none_or(|ch| ch.is_whitespace())
    {
        let end_idx = scanner.current_index();
        let start_idx = end_idx - initial_ch.len_utf8();

        words.push(Spanned::new(
            Word::Literal(initial_ch.to_string()),
            Span::new(start_idx, end_idx),
        ));

        return words;
    }

    while let Some(&ch) = scanner.peek() {
        if matches!(ch, ' ' | '\t' | '&' | '|' | ';' | '>') {
            break;
        }

        if matches!(ch, '\'' | '"' | '$')
            && let Some(initial_ch) = initial_char.take()
        {
            let end_idx = scanner.current_index();
            let start_idx = end_idx - initial_ch.len_utf8();

            words.push(Spanned::new(
                Word::Literal(initial_ch.to_string()),
                Span::new(start_idx, end_idx),
            ));
        }

        let start_idx = if let Some(initial_ch) = initial_char {
            scanner.current_index() - initial_ch.len_utf8()
        } else {
            scanner.current_index()
        };

        let word = match ch {
            '\'' => scan_single_quoted(scanner),
            '"' => scan_double_quoted(scanner),
            '$' => scan_variable(scanner),
            _ => scan_literal(scanner, initial_char.take()),
        };

        words.push(Spanned::new(
            word,
            Span::new(start_idx, scanner.current_index()),
        ));
    }

    words
}

fn scan_single_quoted(scanner: &mut TokenScanner) -> Word {
    let mut content = String::new();

    scanner.next_char();

    while let Some(ch) = scanner.next_char() {
        if ch == '\'' {
            break;
        }
        content.push(ch);
    }

    Word::SingleQuoted(content)
}

fn scan_double_quoted(scanner: &mut TokenScanner) -> Word {
    let mut content = String::new();

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

fn scan_variable(scanner: &mut TokenScanner) -> Word {
    let mut content = String::new();
    let mut invalid_in_brace = false;
    let start_index = scanner.current_index();

    scanner.next_char();
    let has_start_brace = scanner.next_if_matches('{');
    let first_ch = scanner.peek();
    if !has_start_brace && first_ch.is_none_or(|first_c| first_c.is_whitespace()) {
        return Word::Literal("$".to_string());
    }

    if let Some(&ch) = first_ch
        && (ch.is_ascii_alphabetic() || ch == '_')
    {
        content.extend(scanner.next_char());

        while let Some(&c) = scanner.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                content.extend(scanner.next_char());
            } else {
                if has_start_brace && c != '}' {
                    invalid_in_brace = true;
                }
                break;
            }
        }
    }

    if content.is_empty() || invalid_in_brace {
        while let Some(&ch) = scanner.peek() {
            if has_start_brace && ch == '}' || !has_start_brace && ch.is_whitespace() {
                break;
            }
            content.extend(scanner.next_char());
        }

        return Word::Error(
            if scanner.next_if_matches('}') {
                format!("${{{content}}}")
            } else if has_start_brace {
                format!("${{{content}")
            } else {
                format!("${content}")
            },
            ParserError::InvalidVariableName {
                span: Span::new(start_index, scanner.current_index()),
            },
        );
    }

    if has_start_brace && !scanner.next_if_matches('}') {
        return Word::Error(
            format!("${{{content}"),
            ParserError::UnclosedVariableBrace {
                span: Span::new(start_index, scanner.current_index()),
            },
        );
    }

    Word::Variable(content)
}

fn scan_literal(scanner: &mut TokenScanner, initial_char: Option<char>) -> Word {
    let mut content = initial_char.map(|c| c.to_string()).unwrap_or_default();

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

pub fn words_to_string(words: Vec<Spanned<Word>>, variables: &Variables) -> String {
    let mut word_buffer = String::new();

    for word in words {
        match word.item {
            Word::Literal(w) | Word::SingleQuoted(w) | Word::DoubleQuoted(w) => {
                let _ = write!(word_buffer, "{w}");
            }
            Word::Variable(w) => {
                if let Ok(Some(variable)) = variables.get(&w) {
                    let _ = write!(word_buffer, "{variable}");
                }
            }
            Word::Error(w, _) => {
                let _ = write!(word_buffer, "{w}");
            }
            Word::Nothing => {}
        }
    }

    word_buffer
}

pub fn total_word_span(words: &[Spanned<Word>]) -> Span {
    match (words.first(), words.last()) {
        (Some(first), Some(last)) => Span::new(first.span.start, last.span.end),
        _ => Span::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Case<I, E> {
        input: I,
        expected: E,
    }

    #[test]
    fn scan_word_returns_correct_words() {
        let cases = vec![
            Case {
                input: "literal",
                expected: vec![Spanned::new(
                    Word::Literal("literal".to_string()),
                    Span::new(0, 7),
                )],
            },
            Case {
                input: "'single'",
                expected: vec![Spanned::new(
                    Word::SingleQuoted("single".to_string()),
                    Span::new(0, 8),
                )],
            },
            Case {
                input: "\"double\"",
                expected: vec![Spanned::new(
                    Word::DoubleQuoted("double".to_string()),
                    Span::new(0, 8),
                )],
            },
            Case {
                input: "$var",
                expected: vec![Spanned::new(
                    Word::Variable("var".to_string()),
                    Span::new(0, 4),
                )],
            },
        ];

        for case in cases {
            assert_eq!(
                scan_word(&mut TokenScanner::new(case.input), None),
                case.expected
            );
        }
    }

    #[test]
    fn scan_word_with_initial_char_returns_correct_words() {
        let cases = vec![
            Case {
                input: "1literal",
                expected: vec![Spanned::new(
                    Word::Literal("1literal".to_string()),
                    Span::new(0, 8),
                )],
            },
            Case {
                input: "1'single'",
                expected: vec![
                    Spanned::new(Word::Literal("1".to_string()), Span::new(0, 1)),
                    Spanned::new(Word::SingleQuoted("single".to_string()), Span::new(1, 9)),
                ],
            },
            Case {
                input: "1\"double\"",
                expected: vec![
                    Spanned::new(Word::Literal("1".to_string()), Span::new(0, 1)),
                    Spanned::new(Word::DoubleQuoted("double".to_string()), Span::new(1, 9)),
                ],
            },
            Case {
                input: "1$var",
                expected: vec![
                    Spanned::new(Word::Literal("1".to_string()), Span::new(0, 1)),
                    Spanned::new(Word::Variable("var".to_string()), Span::new(1, 5)),
                ],
            },
        ];

        for case in cases {
            let mut scanner = TokenScanner::new(case.input);
            let initial_char = scanner.next_char();
            assert_eq!(scan_word(&mut scanner, initial_char), case.expected);
        }
    }

    #[test]
    fn scan_single_quoted_returns_on_single_quote_char() {
        assert_eq!(
            scan_single_quoted(&mut TokenScanner::new("'hello'w'orld'")),
            Word::SingleQuoted("hello".to_string())
        );
    }

    #[test]
    fn scan_double_quoted_returns_on_double_quote_char() {
        assert_eq!(
            scan_double_quoted(&mut TokenScanner::new("\"hello\"w\"orld\"")),
            Word::DoubleQuoted("hello".to_string())
        );
    }

    #[test]
    fn scan_double_quoted_handles_escaping() {
        assert_eq!(
            scan_double_quoted(&mut TokenScanner::new(
                "\"hello \\$ \\` \\\" \\\\ \\\n world\""
            )),
            Word::DoubleQuoted("hello $ ` \" \\  world".to_string())
        );
    }

    #[test]
    fn scan_variable_empty_trailing_returns_literal() {
        let cases = vec![
            Case {
                input: "$",
                expected: Word::Literal("$".to_string()),
            },
            Case {
                input: "$ ",
                expected: Word::Literal("$".to_string()),
            },
        ];

        for case in cases {
            assert_eq!(
                scan_variable(&mut TokenScanner::new(case.input)),
                case.expected
            );
        }
    }

    #[test]
    fn scan_variable_valid_format_returns_variable() {
        let mut cases = vec![
            Case {
                input: TokenScanner::new("$abc"),
                expected: Word::Variable("abc".to_string()),
            },
            Case {
                input: TokenScanner::new("${abc}"),
                expected: Word::Variable("abc".to_string()),
            },
            Case {
                input: TokenScanner::new("$_abc"),
                expected: Word::Variable("_abc".to_string()),
            },
            Case {
                input: TokenScanner::new("${_abc}"),
                expected: Word::Variable("_abc".to_string()),
            },
            Case {
                input: TokenScanner::new("$a_1"),
                expected: Word::Variable("a_1".to_string()),
            },
            Case {
                input: TokenScanner::new("${a_1}"),
                expected: Word::Variable("a_1".to_string()),
            },
            Case {
                input: TokenScanner::new("$_"),
                expected: Word::Variable("_".to_string()),
            },
            Case {
                input: TokenScanner::new("${_}"),
                expected: Word::Variable("_".to_string()),
            },
        ];

        for case in &mut cases {
            assert_eq!(scan_variable(&mut case.input), case.expected);
        }
    }

    #[test]
    fn scan_variable_invalid_format_returns_error() {
        let mut cases = vec![
            Case {
                input: TokenScanner::new("${a-1@b}"),
                expected: Word::Error(
                    "${a-1@b}".to_string(),
                    ParserError::InvalidVariableName {
                        span: Span::new(0, 7),
                    },
                ),
            },
            Case {
                input: TokenScanner::new("$1"),
                expected: Word::Error(
                    "$1".to_string(),
                    ParserError::InvalidVariableName {
                        span: Span::new(0, 2),
                    },
                ),
            },
            Case {
                input: TokenScanner::new("${1}"),
                expected: Word::Error(
                    "${1}".to_string(),
                    ParserError::InvalidVariableName {
                        span: Span::new(0, 3),
                    },
                ),
            },
            Case {
                input: TokenScanner::new("$!"),
                expected: Word::Error(
                    "$!".to_string(),
                    ParserError::InvalidVariableName {
                        span: Span::new(0, 2),
                    },
                ),
            },
            Case {
                input: TokenScanner::new("${!}"),
                expected: Word::Error(
                    "${!}".to_string(),
                    ParserError::InvalidVariableName {
                        span: Span::new(0, 3),
                    },
                ),
            },
            Case {
                input: TokenScanner::new("${}"),
                expected: Word::Error(
                    "${}".to_string(),
                    ParserError::InvalidVariableName {
                        span: Span::new(0, 2),
                    },
                ),
            },
            Case {
                input: TokenScanner::new("${ }"),
                expected: Word::Error(
                    "${ }".to_string(),
                    ParserError::InvalidVariableName {
                        span: Span::new(0, 3),
                    },
                ),
            },
        ];

        for case in &mut cases {
            assert_eq!(scan_variable(&mut case.input), case.expected);
        }
    }

    #[test]
    fn scan_variable_missing_close_brace_returns_error() {
        assert_eq!(
            scan_variable(&mut TokenScanner::new("${abc")),
            Word::Error(
                "${abc".to_string(),
                ParserError::UnclosedVariableBrace {
                    span: Span::new(0, 5),
                }
            )
        );
    }
}
