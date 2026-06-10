use crate::io::tokenize::Tokens;
use crate::io::writer::{OutputType, write_output};
use crate::shell::ShellState;

pub fn handle_jobs(tokens: Tokens, shell_state: &ShellState) {
    let jobs_output: Vec<String> = shell_state
        .background_jobs
        .iter()
        .enumerate()
        .map(|(idx, job)| {
            let marker = match shell_state.background_jobs.len() - idx {
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
        })
        .collect();

    write_output(
        &jobs_output.join("\n").to_string(),
        OutputType::Stdout,
        &tokens,
    );
}
