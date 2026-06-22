use crate::command::job::Job;
use crate::shell::shell_helper::ShellHelper;
use crate::shell::shell_state::ShellState;
use nu_ansi_term::{Color, Style};
use reedline::{
    ColumnarMenu, DefaultPrompt, DefaultPromptSegment, Emacs, KeyCode, KeyModifiers, MenuBuilder,
    Reedline, ReedlineEvent, ReedlineMenu, Signal, default_emacs_keybindings,
};
use std::sync::{Arc, RwLock};

pub struct Shell {
    shell_state: Arc<RwLock<ShellState>>,
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}

impl Shell {
    pub fn new() -> Self {
        Self {
            shell_state: Arc::new(RwLock::new(ShellState::new())),
        }
    }

    pub fn start_session(&mut self) {
        let shell_helper = ShellHelper::new(Arc::clone(&self.shell_state));

        let mut keybindings = default_emacs_keybindings();
        keybindings.add_binding(
            KeyModifiers::NONE,
            KeyCode::Tab,
            ReedlineEvent::UntilFound(vec![
                ReedlineEvent::Menu("completion_menu".to_string()),
                ReedlineEvent::MenuNext,
            ]),
        );

        let mut editor = Reedline::create()
            .with_completer(Box::new(shell_helper.clone()))
            .with_edit_mode(Box::new(Emacs::new(keybindings)))
            .with_highlighter(Box::new(shell_helper))
            .with_menu(ReedlineMenu::EngineCompleter(Box::new(
                ColumnarMenu::default()
                    .with_name("completion_menu")
                    .with_text_style(Style::new())
                    .with_selected_text_style(Style::new().fg(Color::Fixed(202)))
                    .with_match_text_style(Style::new().underline())
                    .with_selected_match_text_style(Style::new().fg(Color::Fixed(202)).underline())
                    .with_description_text_style(Style::new().dimmed().reset_before_style()),
            )));

        let prompt = DefaultPrompt::new(
            DefaultPromptSegment::WorkingDirectory,
            DefaultPromptSegment::Empty,
        );

        loop {
            let sig = editor.read_line(&prompt);
            match sig {
                Ok(Signal::Success(buffer)) => {
                    if buffer.trim().is_empty() {
                        continue;
                    }

                    editor.history_mut().sync().unwrap();

                    if let Some(job) =
                        { Job::parse_line(&buffer, &self.shell_state.read().unwrap().variables) }
                    {
                        job.run(Arc::clone(&self.shell_state));
                    }

                    self.shell_state.write().unwrap().print_background_jobs();
                }
                Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
                    println!("\nAborted!");
                    break;
                }
                x => {
                    println!("Event: {:?}", x);
                }
            }
        }
    }
}
