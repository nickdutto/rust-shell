use std::fmt::{Display, Formatter};

#[derive(PartialEq)]
pub enum BackgroundJobStatus {
    Done,
    Running,
}

impl Display for BackgroundJobStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BackgroundJobStatus::Done => f.pad("Done"),
            BackgroundJobStatus::Running => f.pad("Running"),
        }
    }
}

pub struct BackgroundJob {
    pub id: usize,
    pub pid: u32,
    pub command: String,
    pub status: BackgroundJobStatus,
}
