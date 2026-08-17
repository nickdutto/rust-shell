use crate::engine::Engine;
use crate::engine::command::BUILTIN_COMMANDS;
use crate::engine::engine_state::EngineState;
use crate::shell::completer::Completer;
use crate::shell::highlighter::SyntaxHighlighter;
use crate::shell::menus::Menus;
use crate::shell::prompt::{ShellPrompt, make_transient_prompt};
use crate::shell::suggestions::Suggestions;
use reedline::{Emacs, ExternalPrinter, Reedline, Signal, default_emacs_keybindings};
use std::sync::Arc;

pub struct Shell;

impl Shell {
    pub fn start_session() {
        let printer = ExternalPrinter::default();
        let engine_state = EngineState::new();
        let engine = Engine::new(engine_state.clone(), printer.clone());
        let prompt = ShellPrompt::from_engine_state(engine.engine_state());

        let mut editor =
            Self::build_editor(&engine.engine_state(), engine.printer().clone(), &prompt);

        Self::repl(&mut editor, &engine, prompt);
    }

    fn repl(editor: &mut Reedline, engine: &Engine, mut prompt: ShellPrompt) {
        loop {
            prompt.refresh_time();

            match editor.read_line(&prompt) {
                Ok(Signal::Success(buffer)) => {
                    if buffer.trim().is_empty() {
                        continue;
                    }

                    editor.history_mut().sync().unwrap();
                    engine.run_line(&buffer);
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

    fn build_editor(
        engine_state: &EngineState,
        external_printer: ExternalPrinter<String>,
        shell_prompt: &ShellPrompt,
    ) -> Reedline {
        let mut editor = Reedline::create();
        let mut keybindings = default_emacs_keybindings();

        let completer = Completer::new(Arc::clone(&engine_state.shell_state), BUILTIN_COMMANDS);
        let suggestions = Suggestions::new(&engine_state.config);
        let syntax_highlighter =
            SyntaxHighlighter::new(Arc::clone(&engine_state.config), BUILTIN_COMMANDS);

        if engine_state.config.menus.completions.enabled {
            editor = editor.with_menu(Menus::completions_menu(
                &engine_state.config.menus.completions,
                &mut keybindings,
            ));
        }

        if engine_state.config.menus.history.enabled {
            editor = editor.with_menu(Menus::history_menu(
                &engine_state.config.menus.history,
                &mut keybindings,
            ));
        }

        editor = editor
            .with_completer(Box::new(completer))
            .with_edit_mode(Box::new(Emacs::new(keybindings)))
            .with_external_printer(external_printer)
            .with_highlighter(Box::new(syntax_highlighter))
            .with_hinter(Box::new(suggestions))
            .with_transient_prompt(make_transient_prompt(shell_prompt));

        editor
    }
}
