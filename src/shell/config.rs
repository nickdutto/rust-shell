use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::{Error as IoError, ErrorKind, Write};
use std::path::Path;
use std::{env, fs};
use thiserror::Error;
use toml::de::Error as TomlDeError;
use toml::ser::Error as TomlSerError;

const CONFIG_PATH_VAR: &str = "RUST_SHELL_CONFIG_PATH";
pub const DEFAULT_DATETIME_FORMAT: &str = "%d/%m/%Y %H:%M:%S";

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("error creating config file")]
    CreateFile { error: Option<IoError> },
    #[error("error opening config file")]
    OpenFile { error: Option<IoError> },
    #[error("error reading config file")]
    ReadFile { error: Option<IoError> },
    #[error("error writing config file")]
    WriteFile { error: Option<IoError> },
    #[error("error creating or using config file path")]
    InvalidConfigFilePath { error: Option<IoError> },
    #[error("error deserializing config file")]
    Deserialize { error: TomlDeError },
    #[error("error serializing config file")]
    Serialize { error: TomlSerError },
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub theme: Theme,
    pub suggestions: Suggestions,
    pub menus: Menus,
    pub prompt: Prompt,
}

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
    pub pipe: u8,
    pub redirection_out: u8,
    pub redirection_out_append: u8,
    pub redirection_error: u8,
    pub redirection_error_append: u8,
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct Suggestions {
    pub cwd_aware: bool,
    pub min_chars: usize,
    pub color: u8,
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct Menus {
    pub completions: Menu,
    pub suggestions: Menu,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Menu {
    pub name: String,
    pub enabled: bool,
    pub key_modifier: String,
    pub key_code: String,
    pub selected_foreground: u8,
}

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

impl Config {
    pub fn load(&mut self) -> Result<(), ConfigError> {
        if let Some(path) = Self::init_file()? {
            let config_content =
                fs::read_to_string(&path).map_err(|e| ConfigError::ReadFile { error: Some(e) })?;

            if config_content.trim().is_empty() {
                self.save()?;
                return Ok(());
            }

            let config: Config = toml::from_str(&config_content)
                .map_err(|error| ConfigError::Deserialize { error })?;

            *self = config;
            self.save()?;
        }

        Ok(())
    }

    pub fn save(&mut self) -> Result<(), ConfigError> {
        if let Some(path) = env::var_os(CONFIG_PATH_VAR) {
            let config_toml =
                toml::to_string_pretty(&self).map_err(|error| ConfigError::Serialize { error })?;

            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)
                .map_err(|e| ConfigError::CreateFile { error: Some(e) })?;

            file.write_all(config_toml.as_bytes())
                .map_err(|e| ConfigError::WriteFile { error: Some(e) })?;
        }

        Ok(())
    }

    fn init_file() -> Result<Option<OsString>, ConfigError> {
        if let Some(path) = env::var_os(CONFIG_PATH_VAR) {
            if let Some(parent) = Path::new(&path).parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| ConfigError::InvalidConfigFilePath { error: Some(e) })?;
            }

            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(_) => {}
                Err(e) if e.kind() == ErrorKind::AlreadyExists => {}
                Err(e) => return Err(ConfigError::CreateFile { error: Some(e) }),
            }

            return Ok(Some(path));
        }

        Ok(None)
    }
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
            pipe: 142,
            redirection_out: 93,
            redirection_out_append: 92,
            redirection_error: 128,
            redirection_error_append: 127,
        }
    }
}

impl Default for Suggestions {
    fn default() -> Self {
        Self {
            cwd_aware: false,
            min_chars: 1,
            color: 8,
        }
    }
}

impl Default for Menus {
    fn default() -> Self {
        Self {
            completions: Menu {
                name: "completions_menu".into(),
                enabled: true,
                key_modifier: "none".into(),
                key_code: "tab".into(),
                selected_foreground: 202,
            },
            suggestions: Menu {
                name: "suggestions_menu".into(),
                enabled: true,
                key_modifier: "control".into(),
                key_code: "w".into(),
                selected_foreground: 202,
            },
        }
    }
}

impl Default for Prompt {
    fn default() -> Self {
        Self {
            left: vec![
                PromptSegment {
                    mode: PromptMode::Username,
                    basic_value: None,
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
                },
                PromptSegment {
                    mode: PromptMode::CurrentDirectory,
                    basic_value: None,
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
                },
            ],
            right: vec![PromptSegment {
                mode: PromptMode::DateTime,
                basic_value: None,
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
            }],
        }
    }
}
