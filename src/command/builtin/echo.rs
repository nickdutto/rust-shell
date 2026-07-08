use crate::io::stream::IoStreams;
use std::io::Write as IoWrite;

pub fn handle_echo(args: Vec<String>, mut io_streams: IoStreams) {
    writeln!(io_streams.output, "{}", args.join(" ").trim()).unwrap();
}
