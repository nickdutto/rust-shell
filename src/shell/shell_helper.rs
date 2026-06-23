use crate::command::BUILTIN_COMMANDS;
use crate::shell::config::Config;
use crate::shell::shell_state::ShellState;
use crate::system::env::get_env_path_executables;
use nu_ansi_term::{Color, Style};
use reedline::{Completer, Highlighter, StyledText, Suggestion};
use std::sync::{Arc, RwLock};
use std::thread;

#[derive(Clone)]
pub struct ShellHelper {
    builtin_commands: Vec<&'static str>,
    config: Config,
    path_executables: Arc<RwLock<Vec<String>>>,
    shell_state: Arc<RwLock<ShellState>>,
}

impl ShellHelper {
    pub fn new(config: Config, shell_state: Arc<RwLock<ShellState>>) -> Self {
        ShellHelper {
            builtin_commands: BUILTIN_COMMANDS.to_vec(),
            config,
            path_executables: ShellHelper::get_path_executables(),
            shell_state,
        }
    }

    fn get_path_executables() -> Arc<RwLock<Vec<String>>> {
        let path_executables = Arc::new(RwLock::new(Vec::new()));
        let path_executables_bg = Arc::clone(&path_executables);

        thread::spawn(move || {
            let executables = get_env_path_executables("PATH");
            if let Ok(mut guard) = path_executables_bg.write() {
                *guard = executables;
            }
        });

        path_executables
    }
}

impl Completer for ShellHelper {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        let partial_input = &line[..pos];
        if partial_input.is_empty() {
            return suggestions;
        }

        {
            let guard = self.shell_state.read().unwrap();

            if !partial_input.contains(' ') {
                guard.completions.complete_command(
                    partial_input,
                    pos,
                    &mut suggestions,
                    &self.builtin_commands,
                    &self.path_executables,
                );
            } else {
                let specification_found = guard.completions.complete_specification(
                    line,
                    partial_input,
                    pos,
                    &mut suggestions,
                );
                if !specification_found {
                    guard
                        .completions
                        .complete_filename(partial_input, pos, &mut suggestions);
                }
            }
        }

        suggestions.sort_by(|a, b| a.value.cmp(&b.value));
        suggestions.dedup_by(|a, b| a.value == b.value);

        suggestions
    }
}

impl Highlighter for ShellHelper {
    fn highlight(&self, line: &str, _cursor: usize) -> StyledText {
        let mut buffer = StyledText::new();
        let style = Style::new().fg(Color::Fixed(self.config.theme.colors.input_base));

        buffer.push((style, line.into()));

        buffer
    }
}
