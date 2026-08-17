use crate::config::menus::Menus;
use crate::config::prompt::Prompt;
use crate::config::suggestions::Suggestions;
use crate::config::theme::Theme;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::{Error as IoError, ErrorKind, Write};
use std::path::Path;
use std::{env, fs};
use thiserror::Error;
use toml::de::Error as TomlDeError;
use toml::ser::Error as TomlSerError;

const CONFIG_PATH_VAR: &str = "RUST_SHELL_CONFIG_PATH";

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
    pub aliases: HashMap<String, String>,
    pub suggestions: Suggestions,
    pub variables: HashMap<String, String>,
    pub menus: Menus,
    pub prompt: Prompt,
    pub theme: Theme,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let Some(path) = Self::init_file()? else {
            return Ok(Self::default());
        };

        let config_content =
            fs::read_to_string(&path).map_err(|e| ConfigError::ReadFile { error: Some(e) })?;

        if config_content.trim().is_empty() {
            let mut config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let mut config: Config =
            toml::from_str(&config_content).map_err(|error| ConfigError::Deserialize { error })?;

        config.save()?;

        Ok(config)
    }

    pub fn load_or_default() -> Self {
        Self::load().unwrap_or_else(|err| {
            eprintln!("{err}");
            Self::default()
        })
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
