use comfy_table::presets::UTF8_HORIZONTAL_ONLY;
use comfy_table::{Attribute, Cell, Color, Table};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::slice::{Iter, IterMut};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BackgroundJobStatus {
    Done,
    Running,
}

#[derive(Debug, PartialEq)]
pub struct BackgroundJob {
    id: usize,
    pub pids: Vec<u32>,
    command: String,
    status: BackgroundJobStatus,
}

#[derive(Debug, Default)]
pub struct BackgroundJobs {
    jobs: Vec<BackgroundJob>,
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
    pub fn new(id: usize, pids: Vec<u32>, command: String, status: BackgroundJobStatus) -> Self {
        Self {
            id,
            pids,
            command,
            status,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn set_command(&mut self, command: String) {
        self.command = command;
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

    pub fn format_job_running(id: usize) -> String {
        format!("[{id}]{:<3}Running", "")
    }

    pub fn format_job_done(&self, idx: usize, len: usize) -> String {
        format!(
            "[{}]{:<3}{:<9}{}",
            self.id,
            BackgroundJob::format_marker(idx, len),
            self.status.to_string(),
            self.command
        )
    }

    pub fn format_marker(idx: usize, len: usize) -> char {
        match len.saturating_sub(idx) {
            1 => '+',
            2 => '-',
            _ => ' ',
        }
    }
}

impl BackgroundJobs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }

    pub fn len(&self) -> usize {
        self.jobs.len()
    }

    pub fn push(&mut self, background_job: BackgroundJob) {
        self.jobs.push(background_job);
    }

    pub fn iter(&self) -> Iter<'_, BackgroundJob> {
        self.jobs.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, BackgroundJob> {
        self.jobs.iter_mut()
    }

    pub fn add_job(&mut self, pids: Vec<u32>, command: String) -> usize {
        let smallest_id = self.smallest_available_id();

        self.jobs.push(BackgroundJob::new(
            smallest_id,
            pids,
            command,
            BackgroundJobStatus::Running,
        ));

        smallest_id
    }

    pub fn complete_job(&mut self, job_id: usize) -> Option<String> {
        let len = self.jobs.len();
        let (idx, job) = self
            .jobs
            .iter_mut()
            .enumerate()
            .find(|(_, job)| job.id() == job_id)?;

        job.set_status(BackgroundJobStatus::Done);
        job.strip_command_suffix();

        Some(job.format_job_done(idx, len))
    }

    pub fn smallest_available_id(&self) -> usize {
        let existing_ids: HashSet<usize> = self.jobs.iter().map(BackgroundJob::id).collect();

        (1..=self.jobs.len() + 1)
            .find(|id| !existing_ids.contains(id))
            .unwrap_or(1)
    }

    pub fn remove_done_jobs(&mut self) {
        self.jobs
            .retain(|job| job.status == BackgroundJobStatus::Running);
    }

    pub fn to_list_string(&self, filter: Option<BackgroundJobStatus>) -> String {
        let len = self.jobs.len();
        let lines: Vec<String> = self
            .jobs
            .iter()
            .enumerate()
            .filter_map(|(idx, job)| {
                if let Some(status_filter) = filter {
                    if job.status != status_filter {
                        return None;
                    }

                    return Some(job.format_job_done(idx, len));
                }

                Some(job.format_job_done(idx, len))
            })
            .collect();

        String::from(lines.join("\n").trim())
    }

    pub fn to_table(&self) -> Table {
        let mut table = Table::new();
        table.load_preset(UTF8_HORIZONTAL_ONLY).set_header(vec![
            Cell::new("id").add_attribute(Attribute::Bold),
            Cell::new("pids").add_attribute(Attribute::Bold),
            Cell::new("status").add_attribute(Attribute::Bold),
            Cell::new("command").add_attribute(Attribute::Bold),
        ]);

        for (idx, job) in self.jobs.iter().enumerate() {
            table.add_row(vec![
                Cell::new(format!(
                    "{}{}",
                    job.id(),
                    BackgroundJob::format_marker(idx, self.jobs.len())
                )),
                Cell::new(
                    job.pids
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(", "),
                ),
                Cell::new(format!("{}", job.status())).fg(
                    if job.status() == BackgroundJobStatus::Running {
                        Color::Blue
                    } else {
                        Color::Green
                    },
                ),
                Cell::new(job.command().to_string()),
            ]);
        }

        table
    }
}

impl<'a> IntoIterator for &'a mut BackgroundJobs {
    type Item = &'a mut BackgroundJob;
    type IntoIter = IterMut<'a, BackgroundJob>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a> IntoIterator for &'a BackgroundJobs {
    type Item = &'a BackgroundJob;
    type IntoIter = Iter<'a, BackgroundJob>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Case<I, E> {
        input: I,
        expected: E,
    }

    #[test]
    fn format_maker_returns_correct_for_position() {
        let cases = vec![
            Case {
                input: 2,
                expected: '+',
            },
            Case {
                input: 1,
                expected: '-',
            },
            Case {
                input: 3,
                expected: ' ',
            },
        ];

        for case in cases {
            assert_eq!(BackgroundJob::format_marker(case.input, 3), case.expected);
        }
    }

    #[test]
    fn add_job_uses_smallest_id() {
        let cases = vec![
            Case {
                input: vec![],
                expected: vec![BackgroundJob {
                    id: 1,
                    pids: vec![],
                    command: "echo".to_string(),
                    status: BackgroundJobStatus::Running,
                }],
            },
            Case {
                input: vec![BackgroundJob {
                    id: 1,
                    pids: vec![],
                    command: "a".to_string(),
                    status: BackgroundJobStatus::Running,
                }],
                expected: vec![
                    BackgroundJob {
                        id: 1,
                        pids: vec![],
                        command: "a".to_string(),
                        status: BackgroundJobStatus::Running,
                    },
                    BackgroundJob {
                        id: 2,
                        pids: vec![],
                        command: "echo".to_string(),
                        status: BackgroundJobStatus::Running,
                    },
                ],
            },
            Case {
                input: vec![
                    BackgroundJob {
                        id: 1,
                        pids: vec![],
                        command: "a".to_string(),
                        status: BackgroundJobStatus::Running,
                    },
                    BackgroundJob {
                        id: 3,
                        pids: vec![],
                        command: "b".to_string(),
                        status: BackgroundJobStatus::Running,
                    },
                ],
                expected: vec![
                    BackgroundJob {
                        id: 1,
                        pids: vec![],
                        command: "a".to_string(),
                        status: BackgroundJobStatus::Running,
                    },
                    BackgroundJob {
                        id: 3,
                        pids: vec![],
                        command: "b".to_string(),
                        status: BackgroundJobStatus::Running,
                    },
                    BackgroundJob {
                        id: 2,
                        pids: vec![],
                        command: "echo".to_string(),
                        status: BackgroundJobStatus::Running,
                    },
                ],
            },
        ];

        for case in cases {
            let mut background_jobs = BackgroundJobs::new();
            for job in case.input {
                background_jobs.push(job);
            }

            background_jobs.add_job(vec![], "echo".to_string());

            assert_eq!(background_jobs.jobs, case.expected);
        }
    }

    #[test]
    fn remove_done_jobs() {
        let mut background_jobs = BackgroundJobs {
            jobs: vec![
                BackgroundJob {
                    id: 1,
                    pids: vec![],
                    command: "a".to_string(),
                    status: BackgroundJobStatus::Running,
                },
                BackgroundJob {
                    id: 2,
                    pids: vec![],
                    command: "b".to_string(),
                    status: BackgroundJobStatus::Done,
                },
                BackgroundJob {
                    id: 3,
                    pids: vec![],
                    command: "c".to_string(),
                    status: BackgroundJobStatus::Running,
                },
            ],
        };

        background_jobs.remove_done_jobs();

        assert_eq!(
            background_jobs.jobs,
            vec![
                BackgroundJob {
                    id: 1,
                    pids: vec![],
                    command: "a".to_string(),
                    status: BackgroundJobStatus::Running,
                },
                BackgroundJob {
                    id: 3,
                    pids: vec![],
                    command: "c".to_string(),
                    status: BackgroundJobStatus::Running,
                },
            ]
        );
    }
}
