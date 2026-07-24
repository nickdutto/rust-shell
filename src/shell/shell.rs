use crate::config::Config;
use crate::engine::Engine;
use crate::engine::command::BUILTIN_COMMANDS;
use crate::engine::router::CommandRouter;
use crate::shell::completer::Completer;
use crate::shell::highlighter::SyntaxHighlighter;
use crate::shell::menus::Menus;
use crate::shell::prompt::ShellPrompt;
use crate::shell::shell_state::ShellState;
use crate::shell::suggestions::Suggestions;
use reedline::{Emacs, ExternalPrinter, Reedline, Signal, default_emacs_keybindings};
use std::sync::{Arc, RwLock};

pub struct Shell {
    config: Arc<Config>,
    command_router: Arc<CommandRouter>,
    shell_state: Arc<RwLock<ShellState>>,
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}

impl Shell {
    pub fn new() -> Self {
        let mut config = Config::default();
        match config.load() {
            Ok(()) => {}
            Err(e) => eprintln!("{e}"),
        }

        let mut command_router = CommandRouter::new();
        command_router.register_builtins();

        let mut shell_state = ShellState::new();

        if !config.aliases.is_empty() {
            shell_state.aliases.set(std::mem::take(&mut config.aliases));
        }

        if !config.variables.is_empty() {
            shell_state
                .variables
                .set(std::mem::take(&mut config.variables));
        }

        Self {
            config: Arc::new(config),
            command_router: Arc::new(command_router),
            shell_state: Arc::new(RwLock::new(shell_state)),
        }
    }

    pub fn start_session(&mut self) {
        let mut editor = Reedline::create();
        let mut keybindings = default_emacs_keybindings();
        let printer = ExternalPrinter::default();
        let prompt = ShellPrompt::new(Arc::clone(&self.config), Arc::clone(&self.shell_state));
        let completer = Completer::new(Arc::clone(&self.shell_state), BUILTIN_COMMANDS);
        let suggestions = Suggestions::new(&self.config);
        let syntax_highlighter = SyntaxHighlighter::new(Arc::clone(&self.config), BUILTIN_COMMANDS);

        if self.config.menus.completions.enabled {
            editor = editor.with_menu(Menus::completions_menu(
                &self.config.menus.completions,
                &mut keybindings,
            ));
        }

        if self.config.menus.history.enabled {
            editor = editor.with_menu(Menus::history_menu(
                &self.config.menus.history,
                &mut keybindings,
            ));
        }

        editor = editor
            .with_completer(Box::new(completer))
            .with_edit_mode(Box::new(Emacs::new(keybindings)))
            .with_external_printer(printer.clone())
            .with_highlighter(Box::new(syntax_highlighter))
            .with_hinter(Box::new(suggestions));

        let engine = Engine::new(
            Arc::clone(&self.config),
            Arc::clone(&self.command_router),
            Arc::clone(&self.shell_state),
            printer,
        );

        self.repl(&mut editor, &engine, &prompt);
    }

    fn repl(&mut self, editor: &mut Reedline, engine: &Engine, prompt: &ShellPrompt) {
        loop {
            let sig = editor.read_line(prompt);
            match sig {
                Ok(Signal::Success(buffer)) => {
                    if buffer.trim().is_empty() {
                        continue;
                    }

                    editor.history_mut().sync().unwrap();

                    engine.run_line(&buffer);

                    self.shell_state
                        .write()
                        .unwrap()
                        .background_jobs
                        .remove_done_jobs();
                }
                Ok(Signal::CtrlC) => {}
                Ok(Signal::CtrlD) => {
                    println!("Aborted");
                    break;
                }
                x => {
                    println!("Event: {x:?}");
                }
            }
        }
    }
}
