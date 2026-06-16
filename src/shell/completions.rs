use std::collections::HashMap;
use std::collections::hash_map::Iter;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompletionError {
    #[error("complete: {key}: no completion specification")]
    MissingSpecification { key: String },
}

#[derive(Default)]
pub struct Completions {
    specifications: HashMap<String, String>,
}

impl Completions {
    pub fn new() -> Self {
        Self {
            specifications: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.specifications.get(key)
    }

    pub fn get_key_value(&self, key: &str) -> Result<(&String, &String), CompletionError> {
        if let Some(script) = self.specifications.get_key_value(key) {
            Ok(script)
        } else {
            Err(CompletionError::MissingSpecification {
                key: key.to_string(),
            })
        }
    }

    pub fn insert(&mut self, key: String, value: String) -> Option<String> {
        self.specifications.insert(key.clone(), value.clone())
    }

    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.specifications.remove(key)
    }

    pub fn iter(&self) -> Iter<'_, String, String> {
        self.specifications.iter()
    }
}
