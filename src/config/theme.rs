use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct Theme {
    pub enable_icons: bool,
    pub colors: ThemeColors,
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct ThemeColors {
    pub input_base: Option<u8>,
    pub builtin_command: u8,
    pub double_quote_strings: u8,
    pub single_quote_strings: u8,
    pub variable: u8,
    pub variable_invalid: u8,
    pub and: u8,
    pub background: u8,
    pub sequential: u8,
    pub pipe: u8,
    pub redirection_out: u8,
    pub redirection_out_append: u8,
    pub redirection_error: u8,
    pub redirection_error_append: u8,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            enable_icons: true,
            colors: ThemeColors::default(),
        }
    }
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            input_base: None,
            builtin_command: 34,
            double_quote_strings: 130,
            single_quote_strings: 131,
            variable: 9,
            variable_invalid: 124,
            and: 71,
            background: 31,
            sequential: 172,
            pipe: 142,
            redirection_out: 93,
            redirection_out_append: 92,
            redirection_error: 128,
            redirection_error_append: 127,
        }
    }
}
