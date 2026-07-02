use crate::io::stream::IoStreams;
use crate::parser::CommandNode;
use std::io::Write;

pub fn handle_echo(tokens: CommandNode, mut io_streams: IoStreams) {
    let output = tokens.arguments.join(" ");
    writeln!(io_streams.output, "{}", output.trim()).unwrap();
}
