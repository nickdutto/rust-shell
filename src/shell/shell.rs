use crate::engine::command::BUILTIN_COMMANDS;
use crate::engine::router::CommandRouter;
use crate::engine::run::run_line;
use crate::shell::completer::Completer;
use crate::shell::config::{Config, Menu as MenuConfig};
use crate::shell::highlighter::SyntaxHighlighter;
use crate::shell::prompt::ShellPrompt;
use crate::shell::shell_state::ShellState;
use crate::shell::suggestions::Suggestions;
use nu_ansi_term::{Color, Style};
use reedline::{
    ColumnarMenu, Emacs, ExternalPrinter, KeyCode, KeyModifiers, Keybindings, Menu, MenuBuilder,
    Reedline, ReedlineEvent, ReedlineMenu, Signal, default_emacs_keybindings,
};
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

        Self {
            config: Arc::new(config),
            command_router: Arc::new(command_router),
            shell_state: Arc::new(RwLock::new(ShellState::new())),
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
            editor = editor.with_menu(ReedlineMenu::EngineCompleter(Self::configure_menu(
                &self.config.menus.completions,
                &mut keybindings,
            )));
        }
        if self.config.menus.suggestions.enabled {
            editor = editor.with_menu(ReedlineMenu::HistoryMenu(Self::configure_menu(
                &self.config.menus.suggestions,
                &mut keybindings,
            )));
        }

        editor = editor
            .with_completer(Box::new(completer))
            .with_edit_mode(Box::new(Emacs::new(keybindings)))
            .with_external_printer(printer.clone())
            .with_highlighter(Box::new(syntax_highlighter))
            .with_hinter(Box::new(suggestions));

        self.repl(&mut editor, &printer, &prompt);
    }

    fn repl(
        &mut self,
        editor: &mut Reedline,
        printer: &ExternalPrinter<String>,
        prompt: &ShellPrompt,
    ) {
        loop {
            let sig = editor.read_line(prompt);
            match sig {
                Ok(Signal::Success(buffer)) => {
                    if buffer.trim().is_empty() {
                        continue;
                    }

                    editor.history_mut().sync().unwrap();

                    run_line(
                        &buffer,
                        &self.config,
                        &self.command_router,
                        &self.shell_state,
                        printer,
                    );

                    self.shell_state
                        .write()
                        .unwrap()
                        .background_jobs
                        .remove_done_jobs();
                }
                Ok(Signal::CtrlD | Signal::CtrlC) => {
                    println!("\nAborted!");
                    break;
                }
                x => {
                    println!("Event: {x:?}");
                }
            }
        }
    }

    fn configure_menu(menu_config: &MenuConfig, keybindings: &mut Keybindings) -> Box<dyn Menu> {
        let style = Style::new();
        let selected_fg = Color::Fixed(menu_config.selected_foreground);

        // TODO: support all key modifiers
        let key_modifier = match menu_config.key_modifier.as_str() {
            "control" => KeyModifiers::CONTROL,
            _ => KeyModifiers::NONE,
        };

        // TODO: support more key code variants
        let key_code = match menu_config.key_code.as_str() {
            "tab" => KeyCode::Tab,
            code => match code.parse::<char>() {
                Ok(ch) => KeyCode::Char(ch),
                Err(_) => KeyCode::Null,
            },
        };

        keybindings.add_binding(
            key_modifier,
            key_code,
            ReedlineEvent::Menu(menu_config.name.clone()),
        );

        Box::new(
            ColumnarMenu::default()
                .with_name(menu_config.name.as_str())
                .with_text_style(style)
                .with_selected_text_style(style.fg(selected_fg))
                .with_match_text_style(style.underline())
                .with_selected_match_text_style(style.fg(selected_fg).underline())
                .with_description_text_style(style.dimmed().reset_before_style()),
        )
    }
}
