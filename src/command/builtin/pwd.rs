use crate::io::stream::IoStreams;
use std::env;
use std::io::Write;

pub fn handle_pwd(mut io_streams: IoStreams) {
    let path = env::current_dir().unwrap();
    writeln!(io_streams.output, "{}", path.display().to_string().trim()).unwrap();
}
