use crate::io::redirection::RedirectionMode;
use crate::parser::span::{Span, Spanned};
use crate::parser::token_scanner::TokenScanner;
use crate::parser::word::{Word, scan_word, total_word_span};

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Word(Vec<Spanned<Word>>),
    Redirection(RedirectionMode),
    Pipe,
    Sequential,
    And,
    Background,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens = vec![];
    let mut scanner = TokenScanner::new(input);

    while let Some(&ch) = scanner.peek() {
        let start_idx = scanner.current_index();

        match ch {
            ' ' | '\t' => {
                scanner.next_char();
            }

            '|' => {
                scanner.next_char();
                tokens.push(Token::new(
                    TokenKind::Pipe,
                    Span::new(start_idx, scanner.current_index()),
                ));
            }

            ';' => {
                scanner.next_char();
                tokens.push(Token::new(
                    TokenKind::Sequential,
                    Span::new(start_idx, scanner.current_index()),
                ));
            }

            '&' => {
                scanner.next_char();
                match scanner.next_if_matches('&') {
                    true => tokens.push(Token::new(
                        TokenKind::And,
                        Span::new(start_idx, scanner.current_index()),
                    )),
                    false => tokens.push(Token::new(
                        TokenKind::Background,
                        Span::new(start_idx, scanner.current_index()),
                    )),
                }
            }

            '>' => {
                scanner.next_char();
                match scanner.next_if_matches('>') {
                    true => tokens.push(Token::new(
                        TokenKind::Redirection(RedirectionMode::OutAppend),
                        Span::new(start_idx, scanner.current_index()),
                    )),
                    false => tokens.push(Token::new(
                        TokenKind::Redirection(RedirectionMode::Out),
                        Span::new(start_idx, scanner.current_index()),
                    )),
                }
            }

            redir_op @ ('1' | '2') => {
                scanner.next_char();
                if scanner.next_if_matches('>') {
                    match (redir_op, scanner.next_if_matches('>')) {
                        ('1', false) => tokens.push(Token::new(
                            TokenKind::Redirection(RedirectionMode::Out),
                            Span::new(start_idx, scanner.current_index()),
                        )),
                        ('1', true) => tokens.push(Token::new(
                            TokenKind::Redirection(RedirectionMode::OutAppend),
                            Span::new(start_idx, scanner.current_index()),
                        )),
                        ('2', false) => tokens.push(Token::new(
                            TokenKind::Redirection(RedirectionMode::Error),
                            Span::new(start_idx, scanner.current_index()),
                        )),
                        ('2', true) => {
                            tokens.push(Token::new(
                                TokenKind::Redirection(RedirectionMode::ErrorAppend),
                                Span::new(start_idx, scanner.current_index()),
                            ));
                        }
                        _ => unreachable!(),
                    }
                } else {
                    let word = scan_word(&mut scanner, Some(redir_op));
                    let span = total_word_span(&word);
                    tokens.push(Token::new(TokenKind::Word(word), span));
                }
            }

            _ => {
                let word = scan_word(&mut scanner, None);
                let span = total_word_span(&word);
                tokens.push(Token::new(TokenKind::Word(word), span));
            }
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::error::ParserError;
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
                expected: vec![Token::new(TokenKind::Pipe, Span::new(0, 1))],
            },
            Case {
                input: ";",
                expected: vec![Token::new(TokenKind::Sequential, Span::new(0, 1))],
            },
            Case {
                input: "&&",
                expected: vec![Token::new(TokenKind::And, Span::new(0, 2))],
            },
            Case {
                input: "&",
                expected: vec![Token::new(TokenKind::Background, Span::new(0, 1))],
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
                expected: vec![Token::new(
                    TokenKind::Redirection(RedirectionMode::Out),
                    Span::new(0, 1),
                )],
            },
            Case {
                input: ">>",
                expected: vec![Token::new(
                    TokenKind::Redirection(RedirectionMode::OutAppend),
                    Span::new(0, 2),
                )],
            },
            Case {
                input: "1>",
                expected: vec![Token::new(
                    TokenKind::Redirection(RedirectionMode::Out),
                    Span::new(0, 2),
                )],
            },
            Case {
                input: "1>>",
                expected: vec![Token::new(
                    TokenKind::Redirection(RedirectionMode::OutAppend),
                    Span::new(0, 3),
                )],
            },
            Case {
                input: "2>",
                expected: vec![Token::new(
                    TokenKind::Redirection(RedirectionMode::Error),
                    Span::new(0, 2),
                )],
            },
            Case {
                input: "2>>",
                expected: vec![Token::new(
                    TokenKind::Redirection(RedirectionMode::ErrorAppend),
                    Span::new(0, 3),
                )],
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
                expected: vec![Token::new(
                    TokenKind::Word(vec![Spanned::new(
                        Word::Literal("literal".to_string()),
                        Span::new(0, 7),
                    )]),
                    Span::new(0, 7),
                )],
            },
            Case {
                input: "'single'",
                expected: vec![Token::new(
                    TokenKind::Word(vec![Spanned::new(
                        Word::SingleQuoted("single".to_string()),
                        Span::new(0, 8),
                    )]),
                    Span::new(0, 8),
                )],
            },
            Case {
                input: "\"double\"",
                expected: vec![Token::new(
                    TokenKind::Word(vec![Spanned::new(
                        Word::DoubleQuoted("double".to_string()),
                        Span::new(0, 8),
                    )]),
                    Span::new(0, 8),
                )],
            },
            Case {
                input: "$var",
                expected: vec![Token::new(
                    TokenKind::Word(vec![Spanned::new(
                        Word::Variable("var".to_string()),
                        Span::new(0, 4),
                    )]),
                    Span::new(0, 4),
                )],
            },
            Case {
                input: "${var_b}",
                expected: vec![Token::new(
                    TokenKind::Word(vec![Spanned::new(
                        Word::Variable("var_b".to_string()),
                        Span::new(0, 8),
                    )]),
                    Span::new(0, 8),
                )],
            },
            Case {
                input: "1",
                expected: vec![Token::new(
                    TokenKind::Word(vec![Spanned::new(
                        Word::Literal("1".to_string()),
                        Span::new(0, 1),
                    )]),
                    Span::new(0, 1),
                )],
            },
            Case {
                input: "2",
                expected: vec![Token::new(
                    TokenKind::Word(vec![Spanned::new(
                        Word::Literal("2".to_string()),
                        Span::new(0, 1),
                    )]),
                    Span::new(0, 1),
                )],
            },
            Case {
                input: "1abc",
                expected: vec![Token::new(
                    TokenKind::Word(vec![Spanned::new(
                        Word::Literal("1abc".to_string()),
                        Span::new(0, 4),
                    )]),
                    Span::new(0, 4),
                )],
            },
            Case {
                input: "literal 'single' \"double\" 1 2 1abc $ $var ${var_b} $! ${1} ${ }",
                expected: vec![
                    Token::new(
                        TokenKind::Word(vec![Spanned::new(
                            Word::Literal("literal".to_string()),
                            Span::new(0, 7),
                        )]),
                        Span::new(0, 7),
                    ),
                    Token::new(
                        TokenKind::Word(vec![Spanned::new(
                            Word::SingleQuoted("single".to_string()),
                            Span::new(8, 16),
                        )]),
                        Span::new(8, 16),
                    ),
                    Token::new(
                        TokenKind::Word(vec![Spanned::new(
                            Word::DoubleQuoted("double".to_string()),
                            Span::new(17, 25),
                        )]),
                        Span::new(17, 25),
                    ),
                    Token::new(
                        TokenKind::Word(vec![Spanned::new(
                            Word::Literal("1".to_string()),
                            Span::new(26, 27),
                        )]),
                        Span::new(26, 27),
                    ),
                    Token::new(
                        TokenKind::Word(vec![Spanned::new(
                            Word::Literal("2".to_string()),
                            Span::new(28, 29),
                        )]),
                        Span::new(28, 29),
                    ),
                    Token::new(
                        TokenKind::Word(vec![Spanned::new(
                            Word::Literal("1abc".to_string()),
                            Span::new(30, 34),
                        )]),
                        Span::new(30, 34),
                    ),
                    Token::new(
                        TokenKind::Word(vec![Spanned::new(
                            Word::Literal("$".to_string()),
                            Span::new(35, 36),
                        )]),
                        Span::new(35, 36),
                    ),
                    Token::new(
                        TokenKind::Word(vec![Spanned::new(
                            Word::Variable("var".to_string()),
                            Span::new(37, 41),
                        )]),
                        Span::new(37, 41),
                    ),
                    Token::new(
                        TokenKind::Word(vec![Spanned::new(
                            Word::Variable("var_b".to_string()),
                            Span::new(42, 50),
                        )]),
                        Span::new(42, 50),
                    ),
                    Token::new(
                        TokenKind::Word(vec![Spanned::new(
                            Word::Error(
                                "$!".to_string(),
                                ParserError::InvalidVariableName {
                                    span: Span::new(51, 53),
                                },
                            ),
                            Span::new(51, 53),
                        )]),
                        Span::new(51, 53),
                    ),
                    Token::new(
                        TokenKind::Word(vec![Spanned::new(
                            Word::Error(
                                "${1}".to_string(),
                                ParserError::InvalidVariableName {
                                    span: Span::new(54, 57),
                                },
                            ),
                            Span::new(54, 58),
                        )]),
                        Span::new(54, 58),
                    ),
                    Token::new(
                        TokenKind::Word(vec![Spanned::new(
                            Word::Error(
                                "${ }".to_string(),
                                ParserError::InvalidVariableName {
                                    span: Span::new(59, 62),
                                },
                            ),
                            Span::new(59, 63),
                        )]),
                        Span::new(59, 63),
                    ),
                ],
            },
        ];

        for case in cases {
            assert_eq!(lex(case.input), case.expected);
        }
    }
}
