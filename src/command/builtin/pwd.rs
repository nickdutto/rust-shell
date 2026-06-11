use crate::io::tokenize::Tokens;
use crate::io::writer::{OutputType, write_output};
use std::env;

pub fn handle_pwd(tokens: Tokens) {
    let path = env::current_dir().unwrap();
    write_output(
        format!("{}", path.display()).trim(),
        OutputType::Stdout,
        &tokens,
    );
}
