use crate::command::job::Job;
use crate::shell::shell_helper::ShellHelper;
use crate::shell::shell_state::ShellState;
use reedline::{
    ColumnarMenu, DefaultPrompt, Emacs, KeyCode, KeyModifiers, MenuBuilder, Reedline,
    ReedlineEvent, ReedlineMenu, Signal, default_emacs_keybindings,
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
            .with_completer(Box::new(ShellHelper::new(Arc::clone(&self.shell_state))))
            .with_edit_mode(Box::new(Emacs::new(keybindings)))
            .with_menu(ReedlineMenu::EngineCompleter(Box::new(
                ColumnarMenu::default().with_name("completion_menu"),
            )));

        let prompt = DefaultPrompt::default();

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
