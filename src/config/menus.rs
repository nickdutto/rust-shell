use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct Menus {
    pub completions: MenuConfig,
    pub history: MenuConfig,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub struct MenuConfig {
    pub enabled: bool,
    pub key_modifier: String,
    pub key_code: String,
    pub selected_foreground: u8,
}

impl Default for Menus {
    fn default() -> Self {
        Self {
            completions: MenuConfig {
                enabled: true,
                key_modifier: "none".into(),
                key_code: "tab".into(),
                selected_foreground: 202,
            },
            history: MenuConfig {
                enabled: true,
                key_modifier: "control".into(),
                key_code: "w".into(),
                selected_foreground: 202,
            },
        }
    }
}
