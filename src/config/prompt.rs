use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct Prompt {
    pub left: Vec<PromptSegment>,
    pub right: Vec<PromptSegment>,
}

#[derive(Default, PartialEq, Deserialize, Serialize)]
pub enum PromptMode {
    Basic,
    CurrentDirectory,
    DateTime,
    #[default]
    Empty,
    Username,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub struct PromptSegment {
    pub mode: PromptMode,
    pub basic_value: Option<String>,
    pub full_directory_path: Option<bool>,
    pub datetime_format: Option<String>,
    pub icon_unicode: Option<String>,
    pub background: u8,
    pub foreground: u8,
    pub bold: bool,
    pub arrow_left: bool,
    pub arrow_left_color: u8,
    pub arrow_right: bool,
    pub arrow_right_color: u8,
    pub gap: bool,
}

impl Default for Prompt {
    fn default() -> Self {
        Self {
            left: vec![
                PromptSegment::mode_username(),
                PromptSegment::mode_current_directory(),
            ],
            right: vec![PromptSegment::mode_datetime()],
        }
    }
}

impl PromptSegment {
    fn mode_username() -> Self {
        Self {
            mode: PromptMode::Username,
            basic_value: None,
            full_directory_path: None,
            datetime_format: None,
            icon_unicode: None,
            background: 33,
            foreground: 255,
            bold: false,
            arrow_left: false,
            arrow_left_color: 33,
            arrow_right: true,
            arrow_right_color: 33,
            gap: true,
        }
    }

    fn mode_current_directory() -> Self {
        Self {
            mode: PromptMode::CurrentDirectory,
            basic_value: None,
            full_directory_path: Some(false),
            datetime_format: None,
            icon_unicode: Some("ea83".into()),
            background: 33,
            foreground: 255,
            bold: false,
            arrow_left: true,
            arrow_left_color: 33,
            arrow_right: true,
            arrow_right_color: 33,
            gap: true,
        }
    }

    fn mode_datetime() -> Self {
        Self {
            mode: PromptMode::DateTime,
            basic_value: None,
            full_directory_path: None,
            datetime_format: Some("%H:%M:%S".into()),
            icon_unicode: Some("f017".into()),
            background: 93,
            foreground: 255,
            bold: false,
            arrow_left: true,
            arrow_left_color: 93,
            arrow_right: false,
            arrow_right_color: 93,
            gap: true,
        }
    }
}
