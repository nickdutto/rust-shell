use std::collections::HashMap;
use std::collections::hash_map::Iter;

#[derive(Debug, Default)]
pub struct Aliases {
    aliases: HashMap<String, String>,
}

impl Aliases {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, replacement: HashMap<String, String>) {
        self.aliases = replacement;
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.aliases.get(key)
    }

    pub fn insert(&mut self, key: String, value: String) -> Option<String> {
        self.aliases.insert(key, value)
    }

    pub fn iter(&self) -> Iter<'_, String, String> {
        self.aliases.iter()
    }
}

impl<'a> IntoIterator for &'a Aliases {
    type Item = (&'a String, &'a String);
    type IntoIter = Iter<'a, String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
