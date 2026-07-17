use comfy_table::presets::NOTHING;
use comfy_table::{Attribute, Cell, Table};
use std::collections::HashMap;
use std::collections::hash_map::Iter;
use std::fmt::Write;

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

    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.aliases.remove(key)
    }

    pub fn iter(&self) -> Iter<'_, String, String> {
        self.aliases.iter()
    }

    pub fn format_item_string(key: &str, value: &str) -> String {
        format!("{key} = \"{value}\"")
    }

    pub fn to_list_string(&self) -> String {
        self.aliases
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
            Cell::new("Alias").add_attribute(Attribute::Bold),
            Cell::new("Aliased").add_attribute(Attribute::Bold),
        ]);

        for (key, value) in &self.aliases {
            table.add_row(vec![key, value]);
        }

        table
    }
}

impl<'a> IntoIterator for &'a Aliases {
    type Item = (&'a String, &'a String);
    type IntoIter = Iter<'a, String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
