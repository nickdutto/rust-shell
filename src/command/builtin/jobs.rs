use crate::io::stream::IoStreams;
use crate::parser::CommandNode;
use crate::shell::shell_state::ShellState;
use std::io::Write;
use std::sync::{Arc, RwLock};

pub fn handle_jobs(
    tokens: CommandNode,
    shell_state: Arc<RwLock<ShellState>>,
    mut io_streams: IoStreams,
) {
    let mut executed = false;

    for arg in tokens.arguments.iter() {
        if arg.as_str() == "-t" {
            let table = shell_state.read().unwrap().background_jobs.to_table();
            writeln!(io_streams.output, "{}", table).unwrap();
            executed = true;
        }
    }

    if !executed {
        let jobs_list = shell_state
            .read()
            .unwrap()
            .background_jobs
            .to_list_string(None);

        if !jobs_list.is_empty() {
            writeln!(io_streams.output, "{jobs_list}").unwrap();
        }
    }

    shell_state
        .write()
        .unwrap()
        .background_jobs
        .remove_done_jobs();
}
