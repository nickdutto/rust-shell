use std::collections::HashMap;
use std::collections::hash_map::Iter;
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

    pub fn get(&self, key: &str) -> Result<Option<&String>, VariableError> {
        if self.validate_key(key) {
            Ok(self.variables.get(key))
        } else {
            Err(VariableError::InvalidIdentifier {
                key: key.to_string(),
                value: String::new(),
            })
        }
    }

    pub fn get_key_value(&self, key: &str) -> Result<Option<(&String, &String)>, VariableError> {
        if self.validate_key(key) {
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
        if self.validate_key(&key) {
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

    fn validate_key(&self, key: &str) -> bool {
        let mut chars = key.chars();
        match chars.next() {
            Some(first) if first.is_ascii_alphabetic() || first == '_' => {
                chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
            }
            _ => false,
        }
    }
}
