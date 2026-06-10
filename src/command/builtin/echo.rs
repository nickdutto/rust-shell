use crate::io::tokenize::Tokens;
use crate::io::writer::{OutputType, write_output};

pub fn handle_echo(tokens: Tokens) {
    let output = tokens.arguments.join(" ");
    write_output(output.trim(), OutputType::Stdout, &tokens);
}
