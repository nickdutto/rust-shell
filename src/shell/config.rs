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

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub theme: Theme,
    pub prompt: Prompt,
}

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Theme {
    pub colors: ThemeColors,
}

#[derive(Clone, Deserialize, Serialize)]
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
    pub prompt_left: u8,
    pub prompt_right: u8,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Prompt {
    pub left: PromptSegment,
    pub right: PromptSegment,
}

#[derive(Clone, Deserialize, Serialize)]
pub enum PromptMode {
    Basic,
    CurrentDirectory,
    DateTime,
    Empty,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct PromptSegment {
    pub mode: PromptMode,
    pub basic_value: Option<String>,
    pub datetime_format: Option<String>,
}

impl Config {
    pub fn load(&mut self) -> Result<(), ConfigError> {
        if let Some(path) = self.init_file()? {
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

    fn init_file(&mut self) -> Result<Option<OsString>, ConfigError> {
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
            prompt_left: 33,
            prompt_right: 33,
        }
    }
}

impl Default for Prompt {
    fn default() -> Self {
        Self {
            left: PromptSegment::from_mode_current_directory(),
            right: PromptSegment::from_mode_datetime(),
        }
    }
}

impl Default for PromptSegment {
    fn default() -> Self {
        PromptSegment::from_mode_empty()
    }
}

impl PromptSegment {
    pub fn from_mode_current_directory() -> Self {
        Self {
            mode: PromptMode::CurrentDirectory,
            basic_value: None,
            datetime_format: None,
        }
    }

    pub fn from_mode_datetime() -> Self {
        Self {
            mode: PromptMode::DateTime,
            basic_value: None,
            datetime_format: Some(DEFAULT_DATETIME_FORMAT.into()),
        }
    }

    pub fn from_mode_empty() -> Self {
        Self {
            mode: PromptMode::Empty,
            basic_value: None,
            datetime_format: None,
        }
    }
}
