use crate::shell::config::MenuConfig;
use crate::system::keys::{parse_key_code, parse_key_modifier};
use nu_ansi_term::{Color, Style};
use reedline::{ColumnarMenu, Keybindings, MenuBuilder, ReedlineEvent, ReedlineMenu};

pub struct Menus;

impl Menus {
    pub fn completions_menu(
        menu_config: &MenuConfig,
        keybindings: &mut Keybindings,
    ) -> ReedlineMenu {
        let menu_name = "completions_menu";
        let menu = Self::columnar_menu(menu_name, menu_config, 4);

        Self::add_keybinding(
            menu_config,
            keybindings,
            ReedlineEvent::UntilFound(vec![
                ReedlineEvent::Menu(menu_name.to_owned()),
                ReedlineEvent::Multiple(vec![
                    ReedlineEvent::Submit,
                    ReedlineEvent::Menu(menu_name.to_owned()),
                ]),
            ]),
        );

        ReedlineMenu::EngineCompleter(Box::new(menu))
    }

    pub fn history_menu(menu_config: &MenuConfig, keybindings: &mut Keybindings) -> ReedlineMenu {
        let menu_name = "history_menu";
        let menu = Self::columnar_menu(menu_name, menu_config, 1);

        Self::add_keybinding(
            menu_config,
            keybindings,
            ReedlineEvent::Menu(menu_name.to_owned()),
        );

        ReedlineMenu::HistoryMenu(Box::new(menu))
    }

    fn columnar_menu(menu_name: &str, menu_config: &MenuConfig, columns: u16) -> ColumnarMenu {
        let style = Style::new();
        let selected_fg = Color::Fixed(menu_config.selected_foreground);

        ColumnarMenu::default()
            .with_name(menu_name)
            .with_columns(columns)
            .with_text_style(style)
            .with_selected_text_style(style.fg(selected_fg))
            .with_match_text_style(style.underline())
            .with_selected_match_text_style(style.fg(selected_fg).underline())
            .with_description_text_style(style.dimmed().reset_before_style())
    }

    fn add_keybinding(
        menu_config: &MenuConfig,
        keybindings: &mut Keybindings,
        event: ReedlineEvent,
    ) {
        keybindings.add_binding(
            parse_key_modifier(menu_config.key_modifier.as_str()),
            parse_key_code(menu_config.key_code.as_str()),
            event,
        );
    }
}
