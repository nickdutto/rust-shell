use crate::io::tokenize::Tokens;
use crate::io::writer::{OutputType, write_output};
use crate::shell::jobs::{BackgroundJob, BackgroundJobStatus};
use crate::shell::shell_state::ShellState;
use std::collections::HashSet;
use std::sync::{Arc, RwLock};

pub fn handle_jobs(tokens: Tokens, shell_state: Arc<RwLock<ShellState>>) {
    let mut job_pids_to_remove: HashSet<u32> = HashSet::new();
    let jobs_output: Vec<String>;

    {
        let guard = shell_state.read().unwrap();
        jobs_output = guard
            .background_jobs
            .iter()
            .enumerate()
            .map(|(idx, job)| {
                if let BackgroundJobStatus::Done = job.status {
                    job_pids_to_remove.insert(job.pid);
                };

                format_job_output(job, idx, guard.background_jobs.len())
            })
            .collect();
    }

    write_output(
        jobs_output.join("\n").to_string().trim(),
        OutputType::Stdout,
        &tokens,
    );

    {
        let mut guard = shell_state.write().unwrap();
        guard
            .background_jobs
            .retain(|job| !job_pids_to_remove.contains(&job.pid));
    }
}

pub fn format_job_output(job: &BackgroundJob, idx: usize, len: usize) -> String {
    let marker = match len - idx {
        1 => "+",
        2 => "-",
        _ => " ",
    };

    format!(
        "[{}]{}  {:<24} {}",
        job.id,
        marker,
        job.status.to_string(),
        job.command
    )
}
