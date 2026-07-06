use crate::parser::token_scanner::TokenScanner;
use crate::parser::word::{Word, scan_word};

#[derive(Debug, PartialEq)]
pub enum RedirectionMode {
    Out,
    OutAppend,
    Error,
    ErrorAppend,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Word(Vec<Word>),
    Redirection(RedirectionMode),
    Pipe,
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
                            tokens.push(Token::Redirection(RedirectionMode::ErrorAppend))
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

    #[test]
    fn lex_tokens() {
        let tokens = lex("hello 'single' \"double\" | $abc ${efg} > >> 1> 1>> 2> 2>>");
        assert_eq!(
            tokens,
            vec![
                Token::Word(vec![Word::Literal("hello".into())]),
                Token::Word(vec![Word::SingleQuoted("single".into())]),
                Token::Word(vec![Word::DoubleQuoted("double".into())]),
                Token::Pipe,
                Token::Word(vec![Word::Variable("abc".into())]),
                Token::Word(vec![Word::Variable("efg".into())]),
                Token::Redirection(RedirectionMode::Out),
                Token::Redirection(RedirectionMode::OutAppend),
                Token::Redirection(RedirectionMode::Out),
                Token::Redirection(RedirectionMode::OutAppend),
                Token::Redirection(RedirectionMode::Error),
                Token::Redirection(RedirectionMode::ErrorAppend),
            ]
        );
    }
}
