use std::fmt::{Display, Formatter};
use std::slice::{Iter, IterMut};

#[derive(PartialEq, Copy, Clone)]
pub enum BackgroundJobStatus {
    Done,
    Running,
}

pub struct BackgroundJob {
    id: usize,
    pid: u32,
    command: String,
    status: BackgroundJobStatus,
}

#[derive(Default)]
pub struct BackgroundJobs {
    background_jobs: Vec<BackgroundJob>,
}

impl Display for BackgroundJobStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BackgroundJobStatus::Done => f.pad("Done"),
            BackgroundJobStatus::Running => f.pad("Running"),
        }
    }
}

impl BackgroundJob {
    pub fn new(id: usize, pid: u32, command: String, status: BackgroundJobStatus) -> Self {
        Self {
            id,
            pid,
            command,
            status,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn set_command(&mut self, command: String) {
        self.command = command
    }

    pub fn strip_command_suffix(&mut self) {
        if let Some(stripped) = self.command.strip_suffix(" &") {
            self.command = stripped.to_string();
        }
    }

    pub fn status(&self) -> BackgroundJobStatus {
        self.status
    }

    pub fn set_status(&mut self, background_job_status: BackgroundJobStatus) {
        self.status = background_job_status;
    }

    pub fn format_job_output(&self, idx: usize, len: usize) -> String {
        let marker = match len - idx {
            1 => "+",
            2 => "-",
            _ => " ",
        };

        format!(
            "[{}]{}  {:<24} {}",
            self.id,
            marker,
            self.status.to_string(),
            self.command
        )
    }
}

impl BackgroundJobs {
    pub fn new() -> Self {
        Self {
            background_jobs: vec![],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.background_jobs.is_empty()
    }

    pub fn len(&self) -> usize {
        self.background_jobs.len()
    }

    pub fn push(&mut self, background_job: BackgroundJob) {
        self.background_jobs.push(background_job);
    }

    pub fn iter(&self) -> Iter<'_, BackgroundJob> {
        self.background_jobs.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, BackgroundJob> {
        self.background_jobs.iter_mut()
    }

    pub fn remove_done_jobs(&mut self) {
        self.background_jobs
            .retain(|job| job.status == BackgroundJobStatus::Running);
    }
}
