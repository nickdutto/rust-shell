use crate::shell::background_jobs::BackgroundJobs;
use crate::shell::completions::Completions;
use crate::shell::history::History;
use crate::shell::variables::Variables;
use crate::system::os;
use std::env;
use std::path::PathBuf;

#[derive(Default)]
pub struct ShellState {
    pub background_jobs: BackgroundJobs,
    pub completions: Completions,
    pub history: History,
    pub variables: Variables,
    pub current_directory: PathBuf,
    pub username: String,
}

impl ShellState {
    pub fn new() -> Self {
        Self {
            background_jobs: BackgroundJobs::new(),
            completions: Completions::new(),
            history: History::new(),
            variables: Variables::new(),
            current_directory: env::current_dir().unwrap_or_default(),
            username: os::get_username().unwrap_or("user?".into()),
        }
    }
}
