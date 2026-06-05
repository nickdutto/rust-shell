use crate::io::tokenize::Tokens;
use crate::io::writer::{OutputType, write_output};
use std::io::Write;

pub fn handle_complete(tokens: Tokens, out_writer: &mut impl Write, err_writer: &mut impl Write) {
    let program = tokens
        .arguments
        .iter()
        .position(|x| x == "-p")
        .and_then(|i| tokens.arguments.get(i + 1));

    if let Some(p) = program {
        write_output(
            &format!("{}: {}: no completion specification", tokens.command, p),
            OutputType::Stdout,
            &tokens,
            err_writer,
        )
    }
}
