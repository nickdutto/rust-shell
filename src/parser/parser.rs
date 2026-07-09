use crate::parser::command_node::CommandNode;
use crate::parser::lexer::Token;
use crate::parser::statement::Statement;
use std::iter::Peekable;
use std::vec::IntoIter;

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens.into_iter().peekable(),
        }
    }

    pub fn parse_statements(&mut self) -> Vec<Statement> {
        let mut statements = vec![];

        while self.tokens.peek().is_some() {
            let Some(mut statement) = self.parse_and_or() else {
                continue;
            };

            if let Some(Token::Background) = self.tokens.peek() {
                self.tokens.next();
                statement = Statement::Background(Box::new(statement));
            }

            if let Some(Token::Sequential) = self.tokens.peek() {
                self.tokens.next();
            }

            statements.push(statement);
        }

        statements
    }

    fn parse_and_or(&mut self) -> Option<Statement> {
        let left = self.parse_pipeline()?;

        if let Some(Token::And) = self.tokens.peek() {
            self.tokens.next();

            let right = self.parse_and_or()?;

            return Some(Statement::And {
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Some(left)
    }

    fn parse_pipeline(&mut self) -> Option<Statement> {
        let mut commands = vec![];

        commands.push(self.parse_command_node());

        while let Some(Token::Pipe) = self.tokens.peek() {
            self.tokens.next();
            commands.push(self.parse_command_node());
        }

        if commands.len() == 1 {
            Some(Statement::Command(commands.pop()?))
        } else {
            Some(Statement::Pipeline(commands))
        }
    }

    fn parse_command_node(&mut self) -> CommandNode {
        let mut command = CommandNode::default();

        while let Some(token) = self.tokens.peek() {
            match token {
                Token::Pipe | Token::Sequential | Token::And | Token::Background => {
                    break;
                }
                _ => {
                    if let Some(tok) = self.tokens.next() {
                        match tok {
                            Token::Word(word) => {
                                if command.cmd.is_empty() {
                                    command.cmd = word;
                                } else {
                                    command.args.push(word);
                                }
                            }
                            Token::Redirection(redirection_mode) => {
                                if let Some(word) = self.tokens.next()
                                    && let Token::Word(w) = word
                                {
                                    command.redirection.mode = redirection_mode;
                                    command.redirection.path = w;
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }
        }

        command
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::redirection::RedirectionMode;
    use crate::parser::command_node::Redirection;
    use crate::parser::lexer::lex;
    use crate::parser::word::Word;

    #[test]
    fn parse_statements_returns_single_command_node() {
        let tokens = lex("echo a");
        let mut parser = Parser::new(tokens);
        let statements = parser.parse_statements();
        let expected = vec![Statement::Command(CommandNode {
            cmd: vec![Word::Literal("echo".to_string())],
            args: vec![vec![Word::Literal("a".to_string())]],
            redirection: Redirection {
                mode: RedirectionMode::Nothing,
                path: vec![],
            },
        })];

        assert_eq!(statements, expected)
    }

    #[test]
    fn parse_statements_returns_correct_precedence() {
        let tokens = lex("echo a | cat && echo b ; ls &");
        let mut parser = Parser::new(tokens);
        let statements = parser.parse_statements();
        let expected = vec![
            Statement::And {
                left: Box::new(Statement::Pipeline(vec![
                    CommandNode {
                        cmd: vec![Word::Literal("echo".to_string())],
                        args: vec![vec![Word::Literal("a".to_string())]],
                        redirection: Redirection {
                            mode: RedirectionMode::Nothing,
                            path: vec![],
                        },
                    },
                    CommandNode {
                        cmd: vec![Word::Literal("cat".to_string())],
                        args: vec![],
                        redirection: Redirection {
                            mode: RedirectionMode::Nothing,
                            path: vec![],
                        },
                    },
                ])),
                right: Box::new(Statement::Command(CommandNode {
                    cmd: vec![Word::Literal("echo".to_string())],
                    args: vec![vec![Word::Literal("b".to_string())]],
                    redirection: Redirection {
                        mode: RedirectionMode::Nothing,
                        path: vec![],
                    },
                })),
            },
            Statement::Background(Box::new(Statement::Command(CommandNode {
                cmd: vec![Word::Literal("ls".to_string())],
                args: vec![],
                redirection: Redirection {
                    mode: RedirectionMode::Nothing,
                    path: vec![],
                },
            }))),
        ];

        assert_eq!(statements, expected)
    }
}
