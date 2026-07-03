use crate::parser::token_scanner::TokenScanner;

#[derive(Debug, PartialEq)]
pub enum RedirectionMode {
    Out,
    OutAppend,
    Error,
    ErrorAppend,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Word(String),
    SingleQuoted(String),
    DoubleQuoted(String),
    Variable(String),
    Redirection(RedirectionMode),
    Pipe,
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut word = String::new();
    let mut tokens = vec![];
    let mut scanner = TokenScanner::new(input);

    while let Some(ch) = scanner.next_char() {
        match ch {
            ' ' | '\t' => {}

            '|' => {
                tokens.push(Token::Pipe);
            }

            '>' => {
                if scanner.next_if_matches('>') {
                    tokens.push(Token::Redirection(RedirectionMode::OutAppend));
                } else {
                    tokens.push(Token::Redirection(RedirectionMode::Out));
                }
            }

            '1' => {
                if scanner.next_if_matches('>') {
                    if scanner.next_if_matches('>') {
                        tokens.push(Token::Redirection(RedirectionMode::OutAppend));
                    } else {
                        tokens.push(Token::Redirection(RedirectionMode::Out));
                    }
                } else {
                    word.push(ch);
                }
            }

            '2' => {
                if scanner.next_if_matches('>') {
                    if scanner.next_if_matches('>') {
                        tokens.push(Token::Redirection(RedirectionMode::ErrorAppend));
                    } else {
                        tokens.push(Token::Redirection(RedirectionMode::Error));
                    }
                } else {
                    word.push(ch);
                }
            }

            '$' => {
                let mut var_name = String::new();
                let has_brace = scanner.next_if_matches('{');

                while let Some(&c) = scanner.peek() {
                    if c.is_ascii_alphanumeric() || c == '_' {
                        if let Some(ca) = scanner.next_char() {
                            var_name.push(ca);
                        }
                    } else {
                        break;
                    }
                }

                if has_brace {
                    scanner.next_if_matches('}');
                }
                tokens.push(Token::Variable(var_name));
            }

            '\'' => {
                while let Some(&c) = scanner.peek() {
                    if c == ' ' || c == '\'' {
                        scanner.next_char();
                        break;
                    }

                    if let Some(ca) = scanner.next_char() {
                        word.push(ca);
                    }
                }
                tokens.push(Token::SingleQuoted(std::mem::take(&mut word)));
            }

            '"' => {
                while let Some(&c) = scanner.peek() {
                    if c == ' ' || c == '"' {
                        scanner.next_char();
                        break;
                    }

                    if let Some(ca) = scanner.next_char() {
                        word.push(ca);
                    }
                }
                tokens.push(Token::DoubleQuoted(std::mem::take(&mut word)));
            }

            _ => {
                word.push(ch);
                while let Some(&c) = scanner.peek() {
                    if c == ' ' || c == '|' || c == '>' || c == '\'' || c == '"' {
                        break;
                    }

                    if let Some(ca) = scanner.next_char() {
                        word.push(ca);
                    }
                }
                tokens.push(Token::Word(std::mem::take(&mut word)));
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
        let tokens = lex("hello 'single' \"double\" | $abc > >> 1> 1>> 1abc 2> 2>> 2xyz");
        assert_eq!(
            tokens,
            vec![
                Token::Word("hello".to_owned()),
                Token::SingleQuoted("single".to_owned()),
                Token::DoubleQuoted("double".to_owned()),
                Token::Pipe,
                Token::Variable("abc".to_owned()),
                Token::Redirection(RedirectionMode::Out),
                Token::Redirection(RedirectionMode::OutAppend),
                Token::Redirection(RedirectionMode::Out),
                Token::Redirection(RedirectionMode::OutAppend),
                Token::Word("1abc".to_owned()),
                Token::Redirection(RedirectionMode::Error),
                Token::Redirection(RedirectionMode::ErrorAppend),
                Token::Word("2xyz".to_owned()),
            ]
        );
    }
}
