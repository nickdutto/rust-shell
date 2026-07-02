use crate::io::stream::IoStreams;
use crate::parser::tokenize::Tokens;
use std::io::Write;

pub fn handle_echo(tokens: Tokens, mut io_streams: IoStreams) {
    let output = tokens.arguments.join(" ");
    writeln!(io_streams.output, "{}", output.trim()).unwrap();
}
