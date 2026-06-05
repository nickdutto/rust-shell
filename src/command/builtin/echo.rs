use crate::io::tokenize::Tokens;
use crate::io::writer::{OutputType, write_output};
use std::io::Write;

pub fn handle_echo(tokens: Tokens, out_writer: &mut impl Write) {
    let output = tokens.arguments.join(" ");
    write_output(output.trim(), OutputType::Stdout, &tokens, out_writer);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::tokenize::Tokens;

    #[test]
    fn test_handle_echo_output() {
        let mut stdout_buffer = Vec::new();
        let tokens = Tokens {
            command: "echo".to_string(),
            arguments: vec!["hello".to_string(), "world".to_string()],
            redirection: None,
        };

        handle_echo(tokens, &mut stdout_buffer);
        let result = String::from_utf8(stdout_buffer).unwrap();

        assert_eq!(result, "hello world\n");
    }
}
