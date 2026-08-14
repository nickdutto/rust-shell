pub mod argument;
pub mod command_node;
pub mod error;
pub mod lexer;
#[allow(clippy::module_inception)]
mod parser;
pub mod span;
pub mod statement;
pub mod syntax_shape;
pub mod token_scanner;
pub mod value;
pub mod word;

pub use self::parser::*;
