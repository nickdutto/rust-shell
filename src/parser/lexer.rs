use crate::io::redirection::RedirectionMode;
use crate::parser::token_scanner::TokenScanner;
use crate::parser::word::{Word, scan_word};

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Word(Vec<Word>),
    Redirection(RedirectionMode),
    Pipe,
    Sequential,
    And,
    Background,
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens = vec![];
    let mut scanner = TokenScanner::new(input);

    while let Some(&ch) = scanner.peek() {
        match ch {
            ' ' | '\t' => {
                scanner.next_char();
            }

            '|' => {
                scanner.next_char();
                tokens.push(Token::Pipe);
            }

            ';' => {
                scanner.next_char();
                tokens.push(Token::Sequential);
            }

            '&' => {
                scanner.next_char();
                match scanner.next_if_matches('&') {
                    true => tokens.push(Token::And),
                    false => tokens.push(Token::Background),
                }
            }

            '>' => {
                scanner.next_char();
                match scanner.next_if_matches('>') {
                    true => tokens.push(Token::Redirection(RedirectionMode::OutAppend)),
                    false => tokens.push(Token::Redirection(RedirectionMode::Out)),
                }
            }

            redir_op @ ('1' | '2') => {
                scanner.next_char();
                if scanner.next_if_matches('>') {
                    match (redir_op, scanner.next_if_matches('>')) {
                        ('1', false) => tokens.push(Token::Redirection(RedirectionMode::Out)),
                        ('1', true) => tokens.push(Token::Redirection(RedirectionMode::OutAppend)),
                        ('2', false) => tokens.push(Token::Redirection(RedirectionMode::Error)),
                        ('2', true) => {
                            tokens.push(Token::Redirection(RedirectionMode::ErrorAppend));
                        }
                        _ => unreachable!(),
                    }
                } else {
                    let word = scan_word(&mut scanner, Some(redir_op));
                    tokens.push(Token::Word(word));
                }
            }

            _ => {
                let word = scan_word(&mut scanner, None);
                tokens.push(Token::Word(word));
            }
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::error::{ParserError, ParserErrorKind};
    use crate::parser::span::Span;

    struct Case<I, E> {
        input: I,
        expected: E,
    }

    #[test]
    fn lex_operators_return_correct_tokens() {
        let cases = vec![
            Case {
                input: "|",
                expected: vec![Token::Pipe],
            },
            Case {
                input: ";",
                expected: vec![Token::Sequential],
            },
            Case {
                input: "&&",
                expected: vec![Token::And],
            },
            Case {
                input: "&",
                expected: vec![Token::Background],
            },
        ];

        for case in cases {
            assert_eq!(lex(case.input), case.expected);
        }
    }

    #[test]
    fn lex_redirections_return_correct_tokens() {
        let cases = vec![
            Case {
                input: ">",
                expected: vec![Token::Redirection(RedirectionMode::Out)],
            },
            Case {
                input: ">>",
                expected: vec![Token::Redirection(RedirectionMode::OutAppend)],
            },
            Case {
                input: "1>",
                expected: vec![Token::Redirection(RedirectionMode::Out)],
            },
            Case {
                input: "1>>",
                expected: vec![Token::Redirection(RedirectionMode::OutAppend)],
            },
            Case {
                input: "2>",
                expected: vec![Token::Redirection(RedirectionMode::Error)],
            },
            Case {
                input: "2>>",
                expected: vec![Token::Redirection(RedirectionMode::ErrorAppend)],
            },
        ];

        for case in cases {
            assert_eq!(lex(case.input), case.expected);
        }
    }

    #[test]
    fn lex_words_return_correct_tokens() {
        let cases = vec![
            Case {
                input: "literal",
                expected: vec![Token::Word(vec![Word::Literal("literal".to_string())])],
            },
            Case {
                input: "'single'",
                expected: vec![Token::Word(vec![Word::SingleQuoted("single".to_string())])],
            },
            Case {
                input: "\"double\"",
                expected: vec![Token::Word(vec![Word::DoubleQuoted("double".to_string())])],
            },
            Case {
                input: "$var",
                expected: vec![Token::Word(vec![Word::Variable("var".to_string())])],
            },
            Case {
                input: "${var_b}",
                expected: vec![Token::Word(vec![Word::Variable("var_b".to_string())])],
            },
            Case {
                input: "1",
                expected: vec![Token::Word(vec![Word::Literal("1".to_string())])],
            },
            Case {
                input: "2",
                expected: vec![Token::Word(vec![Word::Literal("2".to_string())])],
            },
            Case {
                input: "1abc",
                expected: vec![Token::Word(vec![Word::Literal("1abc".to_string())])],
            },
            Case {
                input: "literal 'single' \"double\" 1 2 1abc $ $var ${var_b} $! ${1} ${ }",
                expected: vec![
                    Token::Word(vec![Word::Literal("literal".to_string())]),
                    Token::Word(vec![Word::SingleQuoted("single".to_string())]),
                    Token::Word(vec![Word::DoubleQuoted("double".to_string())]),
                    Token::Word(vec![Word::Literal("1".to_string())]),
                    Token::Word(vec![Word::Literal("2".to_string())]),
                    Token::Word(vec![Word::Literal("1abc".to_string())]),
                    Token::Word(vec![Word::Literal("$".to_string())]),
                    Token::Word(vec![Word::Variable("var".to_string())]),
                    Token::Word(vec![Word::Variable("var_b".to_string())]),
                    Token::Word(vec![Word::Error(ParserError {
                        kind: ParserErrorKind::InvalidVariableName,
                        span: Span::new(51, 53),
                        raw_string: "$!".to_string(),
                    })]),
                    Token::Word(vec![Word::Error(ParserError {
                        kind: ParserErrorKind::InvalidVariableName,
                        span: Span::new(54, 57),
                        raw_string: "${1}".to_string(),
                    })]),
                    Token::Word(vec![Word::Error(ParserError {
                        kind: ParserErrorKind::InvalidVariableName,
                        span: Span::new(59, 62),
                        raw_string: "${ }".to_string(),
                    })]),
                ],
            },
        ];

        for case in cases {
            assert_eq!(lex(case.input), case.expected);
        }
    }
}
