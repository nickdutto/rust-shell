use crate::writer::{Redirection, RedirectionMode};
use std::iter::Peekable;
use std::str::Chars;

#[derive(Clone, Copy)]
enum TokenState {
    Normal,
    InSingleQuote,
    InDoubleQuote,
    EscapeNormal,
    EscapeDoubleQuote,
    RedirectOut,
    RedirectError,
}

#[derive(Debug)]
pub struct Tokens {
    pub command: String,
    pub arguments: Vec<String>,
    pub redirection: Option<Redirection>,
}

struct TokenScanner<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> TokenScanner<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
        }
    }

    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    fn next(&mut self) -> Option<char> {
        self.chars.next()
    }

    fn next_if_matches(&mut self, expected: char) -> bool {
        if self.peek() == Some(&expected) {
            self.next();
            true
        } else {
            false
        }
    }
}

pub fn tokenize_arguments(input: &str) -> Tokens {
    let mut tokens = vec![];
    let mut token_buffer = String::new();
    let mut state: TokenState = TokenState::Normal;
    let mut redirection_mode: Option<RedirectionMode> = None;
    let mut redirection_location = String::new();

    let mut scanner = TokenScanner::new(input);
    while let Some(ch) = scanner.next() {
        match (ch, state) {
            ('\\', TokenState::Normal) => state = TokenState::EscapeNormal,
            ('\\', TokenState::InDoubleQuote) => state = TokenState::EscapeDoubleQuote,
            (ch, TokenState::EscapeNormal) => {
                token_buffer.push(ch);
                state = TokenState::Normal;
            }
            (ch, TokenState::EscapeDoubleQuote) => {
                if ch == '"' || ch == '\\' {
                    token_buffer.push(ch);
                } else {
                    token_buffer.push('\\');
                    token_buffer.push(ch);
                }
                state = TokenState::InDoubleQuote;
            }
            ('\'', TokenState::Normal) => state = TokenState::InSingleQuote,
            ('"', TokenState::Normal) => state = TokenState::InDoubleQuote,
            ('\'', TokenState::InSingleQuote) => state = TokenState::Normal,
            ('"', TokenState::InDoubleQuote) => state = TokenState::Normal,
            (' ', TokenState::Normal) => {
                if !token_buffer.is_empty() {
                    tokens.push(std::mem::take(&mut token_buffer));
                }
            }

            ('1', TokenState::Normal) => {
                if scanner.next_if_matches('>') {
                    if scanner.next_if_matches('>') {
                        redirection_mode = Some(RedirectionMode::OutputAppend);
                    } else {
                        redirection_mode = Some(RedirectionMode::Output);
                    }
                    state = TokenState::RedirectOut;
                } else {
                    token_buffer.push('1');
                }
            }

            ('2', TokenState::Normal) => {
                if scanner.next_if_matches('>') {
                    if scanner.next_if_matches('>') {
                        redirection_mode = Some(RedirectionMode::ErrorAppend);
                    } else {
                        redirection_mode = Some(RedirectionMode::Error);
                    }
                    state = TokenState::RedirectError;
                } else {
                    token_buffer.push('2');
                }
            }

            ('>', TokenState::Normal) => {
                if scanner.next_if_matches('>') {
                    redirection_mode = Some(RedirectionMode::OutputAppend);
                } else {
                    redirection_mode = Some(RedirectionMode::Output);
                }
                state = TokenState::RedirectOut;
            }

            (ch, TokenState::RedirectOut) | (ch, TokenState::RedirectError) => {
                redirection_location.push(ch);
            }

            (ch, _) => {
                token_buffer.push(ch);
            }
        }
    }

    if !token_buffer.is_empty() {
        tokens.push(token_buffer);
    }

    let redirection: Option<Redirection> = redirection_mode.map(|mode| Redirection {
        mode,
        location: redirection_location.trim().into(),
    });

    let command = tokens.first().unwrap().trim().to_string();
    let arguments = tokens.get(1..).unwrap_or(&[]).to_vec();

    Tokens {
        command,
        arguments,
        redirection,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCase<'a> {
        input: &'a str,
        expected: Vec<&'a str>,
        description: &'a str,
    }

    #[test]
    fn tokenize_arguments_handles_quotes_and_spaces() {
        let test_cases = vec![
            TestCase {
                input: "echo hello    world",
                expected: vec!["hello", "world"],
                description: "Split and collapse on multiple unquoted spaces",
            },
            TestCase {
                input: "echo 'hello    world'",
                expected: vec!["hello    world"],
                description: "Preserve spaces within single quotes",
            },
            TestCase {
                input: "echo 'hello''world'",
                expected: vec!["helloworld"],
                description: "Merge adjacent single quoted strings",
            },
            TestCase {
                input: "echo hello''world",
                expected: vec!["helloworld"],
                description: "Ignore empty single quote pairs",
            },
            TestCase {
                input: "echo \"hello    world\"",
                expected: vec!["hello    world"],
                description: "Preserve spaces within double quotes",
            },
            TestCase {
                input: "echo hello\"\"world",
                expected: vec!["helloworld"],
                description: "Ignore empty double quote pairs",
            },
            TestCase {
                input: "echo \"hello\"world",
                expected: vec!["helloworld"],
                description: "Merge adjacent double quoted and unquoted strings",
            },
            TestCase {
                input: "echo \"hello\" \"world\"",
                expected: vec!["hello", "world"],
                description: "Separate space-delimited double quoted strings",
            },
            TestCase {
                input: "echo \"shell\'s test\"",
                expected: vec!["shell's test"],
                description: "Treat single quotes inside double quotes as literal",
            },
            TestCase {
                input: "echo \"quz  hello\"  \"bar\"",
                expected: vec!["quz  hello", "bar"],
                description: "Treat single quotes inside double quotes as literal",
            },
            TestCase {
                input: "echo \"bar\"  \"shell's\"  \"foo\"",
                expected: vec!["bar", "shell's", "foo"],
                description: "Treat single quotes inside double quotes as literal",
            },
            TestCase {
                input: "echo three\\ \\ \\ spaces",
                expected: vec!["three   spaces"],
                description: "Each \\ creates a literal space as part of one argument.",
            },
            TestCase {
                input: "echo before\\     after",
                expected: vec!["before ", "after"],
                description: "The backslash preserves the first space literally, but the shell collapses the subsequent unescaped spaces.",
            },
            TestCase {
                input: "echo test\\nexample",
                expected: vec!["testnexample"],
                description: "\\n becomes just n.",
            },
            TestCase {
                input: "echo hello\\\\world",
                expected: vec!["hello\\world"],
                description: "The first backslash escapes the second, and the result is a single literal backslash in the argument.",
            },
            TestCase {
                input: "echo \\'hello\\'",
                expected: vec!["'hello'"],
                description: "\\' makes the single quotes literal characters.",
            },
            TestCase {
                input: "echo 'shell\\\nscript'",
                expected: vec!["shell\\\nscript"],
                description: "Every character in single quotes treated literally (including backslashes) - backslash and \\n newline",
            },
            TestCase {
                input: "echo 'example\"test'",
                expected: vec!["example\"test"],
                description: "Every character in single quotes treated literally (including backslashes) - single",
            },
            TestCase {
                input: "echo \"A \\\\ escapes itself\"",
                expected: vec!["A \\ escapes itself"],
                description: "Return literal backslash when escaping another backslash in double quotes",
            },
            TestCase {
                input: "echo \"A \\\" inside double quotes\"",
                expected: vec!["A \" inside double quotes"],
                description: "Return literal \" double quote when escaping within double quote string",
            },
        ];

        for case in test_cases {
            let tokens = tokenize_arguments(case.input);

            assert_eq!(
                tokens.arguments, case.expected,
                "Failed test case [{}] for input: {:?}",
                case.description, case.input
            );
        }
    }

    #[test]
    fn tokenize_arguments_handles_redirection() {
        let tokens = tokenize_arguments("echo hello > output.txt");
        assert_eq!(tokens.command, "echo");
        assert_eq!(tokens.arguments, vec!["hello"]);
        assert!(tokens.redirection.is_some());
        let redir = tokens.redirection.unwrap();
        assert_eq!(redir.mode, RedirectionMode::Output);
        assert_eq!(redir.location, "output.txt");

        let tokens = tokenize_arguments("pwd 1> /tmp/foo/bar.log");
        assert_eq!(tokens.command, "pwd");
        assert!(tokens.arguments.is_empty());
        assert!(tokens.redirection.is_some());
        let redir = tokens.redirection.unwrap();
        assert_eq!(redir.mode, RedirectionMode::Output);
        assert_eq!(redir.location, "/tmp/foo/bar.log");

        let tokens = tokenize_arguments("cd /nonexistent 2> err.txt");
        assert_eq!(tokens.command, "cd");
        assert_eq!(tokens.arguments, vec!["/nonexistent"]);
        assert!(tokens.redirection.is_some());
        let redir = tokens.redirection.unwrap();
        assert_eq!(redir.mode, RedirectionMode::Error);
        assert_eq!(redir.location, "err.txt");

        let tokens = tokenize_arguments("echo 1 target 2");
        assert_eq!(tokens.command, "echo");
        assert_eq!(tokens.arguments, vec!["1", "target", "2"]);
        assert!(tokens.redirection.is_none());
    }

    #[test]
    fn tokenize_arguments_handles_redirection_append() {
        let tokens = tokenize_arguments("echo hello >> output.txt");
        assert_eq!(tokens.command, "echo");
        assert_eq!(tokens.arguments, vec!["hello"]);
        assert!(tokens.redirection.is_some());
        let redir = tokens.redirection.unwrap();
        assert_eq!(redir.mode, RedirectionMode::OutputAppend);
        assert_eq!(redir.location, "output.txt");

        let tokens = tokenize_arguments("pwd 1>> /tmp/foo/bar.log");
        assert_eq!(tokens.command, "pwd");
        assert!(tokens.arguments.is_empty());
        assert!(tokens.redirection.is_some());
        let redir = tokens.redirection.unwrap();
        assert_eq!(redir.mode, RedirectionMode::OutputAppend);
        assert_eq!(redir.location, "/tmp/foo/bar.log");

        let tokens = tokenize_arguments("cd /nonexistent 2>> err.txt");
        assert_eq!(tokens.command, "cd");
        assert_eq!(tokens.arguments, vec!["/nonexistent"]);
        assert!(tokens.redirection.is_some());
        let redir = tokens.redirection.unwrap();
        assert_eq!(redir.mode, RedirectionMode::ErrorAppend);
        assert_eq!(redir.location, "err.txt");

        let tokens = tokenize_arguments("echo 1 target 2");
        assert_eq!(tokens.command, "echo");
        assert_eq!(tokens.arguments, vec!["1", "target", "2"]);
        assert!(tokens.redirection.is_none());
    }
}
