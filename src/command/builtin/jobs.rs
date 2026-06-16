use crate::io::stream::IoStreams;
use crate::shell::shell_state::ShellState;
use std::io::Write;
use std::sync::{Arc, RwLock};

pub fn handle_jobs(shell_state: Arc<RwLock<ShellState>>, mut io_streams: IoStreams) {
    let jobs_output: Vec<String>;

    {
        let guard = shell_state.read().unwrap();
        jobs_output = guard
            .background_jobs
            .iter()
            .enumerate()
            .map(|(idx, job)| job.format_job_output(idx, guard.background_jobs.len()))
            .collect();
    }

    if !jobs_output.is_empty() {
        writeln!(
            io_streams.output,
            "{}",
            jobs_output.join("\n").to_string().trim()
        )
            .unwrap();
    }
    {
        let mut guard = shell_state.write().unwrap();
        guard.background_jobs.remove_done_jobs();
    }
}
