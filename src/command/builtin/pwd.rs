use crate::io::tokenize::Tokens;
use crate::io::writer::{OutputType, write_output};
use std::env;
use std::io::Write;

pub fn handle_pwd(tokens: Tokens, out_writer: &mut impl Write) {
    let path = env::current_dir().unwrap();
    write_output(
        &format!("{}", path.display()),
        OutputType::Stdout,
        &tokens,
        out_writer,
    );
}
