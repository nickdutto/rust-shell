use crate::parser::command_node::CommandNode;
use crate::parser::lexer::{Token, TokenKind};
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
            if let Some(statement) = self.parse_sequential() {
                statements.push(statement);
            } else {
                break;
            }
        }

        statements
    }

    fn parse_sequential(&mut self) -> Option<Statement> {
        let mut left = self.parse_and_or()?;

        if matches!(self.tokens.peek(), Some(token) if token.kind == TokenKind::Background) {
            self.tokens.next();
            left = Statement::Background(Box::new(left));
        }

        if matches!(self.tokens.peek(), Some(token) if token.kind == TokenKind::Sequential) {
            self.tokens.next();

            let right = self.parse_sequential()?;

            return Some(Statement::Sequential {
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Some(left)
    }

    fn parse_and_or(&mut self) -> Option<Statement> {
        let left = self.parse_pipeline()?;

        if matches!(self.tokens.peek(), Some(token) if token.kind == TokenKind::And) {
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

        while let Some(Token {
                           kind: TokenKind::Pipe,
                           ..
                       }) = self.tokens.peek()
        {
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
            match token.kind {
                TokenKind::Pipe
                | TokenKind::Sequential
                | TokenKind::And
                | TokenKind::Background => {
                    break;
                }
                _ => {
                    if let Some(tok) = self.tokens.next() {
                        match tok.kind {
                            TokenKind::Word(word) => {
                                if command.cmd.is_empty() {
                                    command.cmd = word;
                                } else {
                                    command.args.push(word);
                                }
                            }
                            TokenKind::Redirection(redirection_mode) => {
                                if let Some(word) = self.tokens.next()
                                    && let TokenKind::Word(w) = word.kind
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
    use crate::parser::span::{Span, Spanned};
    use crate::parser::word::Word;

    #[test]
    fn parse_statements_returns_single_command_node() {
        let tokens = lex("echo a");
        let mut parser = Parser::new(tokens);
        let statements = parser.parse_statements();
        let expected = vec![Statement::Command(CommandNode {
            cmd: vec![Spanned::new(
                Word::Literal("echo".to_string()),
                Span::new(0, 4),
            )],
            args: vec![vec![Spanned::new(
                Word::Literal("a".to_string()),
                Span::new(5, 6),
            )]],
            redirection: Redirection {
                mode: RedirectionMode::Nothing,
                path: vec![],
            },
        })];

        assert_eq!(statements, expected);
    }

    #[test]
    fn parse_statements_returns_correct_precedence() {
        let tokens = lex("echo a | cat && echo b ; ls &");
        let mut parser = Parser::new(tokens);
        let statements = parser.parse_statements();
        let expected = vec![Statement::Sequential {
            left: Box::new(Statement::And {
                left: Box::new(Statement::Pipeline(vec![
                    CommandNode {
                        cmd: vec![Spanned::new(
                            Word::Literal("echo".to_string()),
                            Span::new(0, 4),
                        )],
                        args: vec![vec![Spanned::new(
                            Word::Literal("a".to_string()),
                            Span::new(5, 6),
                        )]],
                        redirection: Redirection {
                            mode: RedirectionMode::Nothing,
                            path: vec![],
                        },
                    },
                    CommandNode {
                        cmd: vec![Spanned::new(
                            Word::Literal("cat".to_string()),
                            Span::new(9, 12),
                        )],
                        args: vec![],
                        redirection: Redirection {
                            mode: RedirectionMode::Nothing,
                            path: vec![],
                        },
                    },
                ])),
                right: Box::new(Statement::Command(CommandNode {
                    cmd: vec![Spanned::new(
                        Word::Literal("echo".to_string()),
                        Span::new(16, 20),
                    )],
                    args: vec![vec![Spanned::new(
                        Word::Literal("b".to_string()),
                        Span::new(21, 22),
                    )]],
                    redirection: Redirection {
                        mode: RedirectionMode::Nothing,
                        path: vec![],
                    },
                })),
            }),
            right: Box::new(Statement::Background(Box::new(Statement::Command(
                CommandNode {
                    cmd: vec![Spanned::new(
                        Word::Literal("ls".to_string()),
                        Span::new(25, 27),
                    )],
                    args: vec![],
                    redirection: Redirection {
                        mode: RedirectionMode::Nothing,
                        path: vec![],
                    },
                },
            )))),
        }];

        assert_eq!(statements, expected);
    }
}
