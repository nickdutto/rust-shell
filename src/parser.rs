#[derive(Clone, Copy)]
enum TokenState {
    Normal,
    InSingleQuote,
    InDoubleQuote,
}

pub fn tokenize_arguments(input: &str) -> Vec<String> {
    let mut tokens = vec![];
    let mut token_buffer = String::new();
    let mut state: TokenState = TokenState::Normal;

    for ch in input.chars() {
        match (ch, state) {
            ('\'', TokenState::Normal) => state = TokenState::InSingleQuote,
            ('"', TokenState::Normal) => state = TokenState::InDoubleQuote,
            ('\'', TokenState::InSingleQuote) => state = TokenState::Normal,
            ('"', TokenState::InDoubleQuote) => state = TokenState::Normal,
            (' ', TokenState::Normal) => {
                if !token_buffer.is_empty() {
                    tokens.push(std::mem::take(&mut token_buffer));
                }
            }
            _ => token_buffer.push(ch),
        }
    }

    if !token_buffer.is_empty() {
        tokens.push(token_buffer);
    }

    tokens
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
    fn tokenize_args_handles_quotes_and_spaces() {
        let test_cases = vec![
            TestCase {
                input: "hello    world",
                expected: vec!["hello", "world"],
                description: "Split and collapse on multiple unquoted spaces",
            },
            TestCase {
                input: "'hello    world'",
                expected: vec!["hello    world"],
                description: "Preserve spaces within single quotes",
            },
            TestCase {
                input: "'hello''world'",
                expected: vec!["helloworld"],
                description: "Merge adjacent single quoted strings",
            },
            TestCase {
                input: "hello''world",
                expected: vec!["helloworld"],
                description: "Ignore empty single quote pairs",
            },
            TestCase {
                input: "\"hello    world\"",
                expected: vec!["hello    world"],
                description: "Preserve spaces within double quotes",
            },
            TestCase {
                input: "hello\"\"world",
                expected: vec!["helloworld"],
                description: "Ignore empty double quote pairs",
            },
            TestCase {
                input: "\"hello\"world",
                expected: vec!["helloworld"],
                description: "Merge adjacent double quoted and unquoted strings",
            },
            TestCase {
                input: "\"hello\" \"world\"",
                expected: vec!["hello", "world"],
                description: "Separate space-delimited double quoted strings",
            },
            TestCase {
                input: "\"shell\'s test\"",
                expected: vec!["shell's test"],
                description: "Treat single quotes inside double quotes as literal",
            },
            TestCase {
                input: "\"quz  hello\"  \"bar\"",
                expected: vec!["quz  hello", "bar"],
                description: "Treat single quotes inside double quotes as literal",
            },
            TestCase {
                input: "\"bar\"  \"shell's\"  \"foo\"",
                expected: vec!["bar", "shell's", "foo"],
                description: "Treat single quotes inside double quotes as literal",
            },
        ];

        for case in test_cases {
            let actual = tokenize_arguments(case.input);

            assert_eq!(
                actual, case.expected,
                "Failed test case [{}] for input: {:?}",
                case.description, case.input
            );
        }
    }
}
