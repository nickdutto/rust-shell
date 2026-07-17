use comfy_table::presets::NOTHING;
use comfy_table::{Attribute, Cell, Table};
use std::collections::HashMap;
use std::collections::hash_map::Iter;
use std::fmt::Write;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VariableError {
    #[error("`{key}={value}': not a valid identifier")]
    InvalidIdentifier { key: String, value: String },
}

#[derive(Default)]
pub struct Variables {
    variables: HashMap<String, String>,
}

impl Variables {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn set(&mut self, replacement: HashMap<String, String>) {
        self.variables = replacement
            .into_iter()
            .filter(|(key, _)| Self::validate_key(key))
            .collect();
    }

    pub fn get(&self, key: &str) -> Result<Option<&String>, VariableError> {
        if Self::validate_key(key) {
            Ok(self.variables.get(key))
        } else {
            Err(VariableError::InvalidIdentifier {
                key: key.to_string(),
                value: String::new(),
            })
        }
    }

    pub fn get_key_value(&self, key: &str) -> Result<Option<(&String, &String)>, VariableError> {
        if Self::validate_key(key) {
            Ok(self.variables.get_key_value(key))
        } else {
            Err(VariableError::InvalidIdentifier {
                key: key.to_string(),
                value: String::new(),
            })
        }
    }

    pub fn insert(
        &mut self,
        key: String,
        value: String,
    ) -> Result<Option<(&String, &String)>, VariableError> {
        if Self::validate_key(&key) {
            self.variables.insert(key.clone(), value.clone());

            Ok(self.get_key_value(&key)?)
        } else {
            Err(VariableError::InvalidIdentifier { key, value })
        }
    }

    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.variables.remove(key)
    }

    pub fn iter(&self) -> Iter<'_, String, String> {
        self.variables.iter()
    }

    pub fn validate_key(key: &str) -> bool {
        let mut chars = key.chars();
        match chars.next() {
            Some(first) if first.is_ascii_alphabetic() || first == '_' => {
                chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
            }
            _ => false,
        }
    }

    pub fn format_item_string(key: &str, value: &str) -> String {
        format!("{key} = \"{value}\"")
    }

    pub fn to_list_string(&self) -> String {
        self.variables
            .iter()
            .fold(String::new(), |mut buffer, (key, value)| {
                let _ = writeln!(buffer, "{}", Self::format_item_string(key, value));
                buffer
            })
    }

    pub fn to_table(&self) -> Table {
        let mut table = Table::new();

        table.load_preset(NOTHING);
        table.set_header(vec![
            Cell::new("Variable").add_attribute(Attribute::Bold),
            Cell::new("Value").add_attribute(Attribute::Bold),
        ]);

        for (key, value) in &self.variables {
            table.add_row(vec![key, value]);
        }

        table
    }
}

impl<'a> IntoIterator for &'a Variables {
    type Item = (&'a String, &'a String);
    type IntoIter = Iter<'a, String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
