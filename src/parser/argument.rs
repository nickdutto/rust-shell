use crate::error::shell_error::ShellError;
use crate::parser::span::Spanned;
use crate::parser::value::{FromValue, Value};
use std::collections::HashMap;

#[derive(Default)]
pub struct ParsedArguments {
    pub cmd: Spanned<String>,
    pub positionals: Vec<Value>,
    pub rest: Vec<Value>,
    pub named: HashMap<String, Value>,
    pub raw_args: Vec<Spanned<String>>,
}

impl ParsedArguments {
    pub fn req<T: FromValue>(&self, index: usize) -> Result<T, ShellError> {
        self.positionals
            .get(index)
            .cloned()
            .ok_or_else(|| ShellError::MissingPositionalArgument {
                cmd: String::new(),
                name: format!("index {index}"),
                span: self.cmd.span,
            })
            .and_then(T::from_value)
    }

    pub fn opt<T: FromValue>(&self, index: usize) -> Result<Option<T>, ShellError> {
        match self.positionals.get(index) {
            Some(v) => T::from_value(v.clone()).map(Some),
            None => Ok(None),
        }
    }

    pub fn req_named<T: FromValue>(&self, name: &str) -> Result<T, ShellError> {
        self.named
            .get(name)
            .cloned()
            .ok_or_else(|| ShellError::MissingNamedArgument {
                cmd: self.cmd.item.clone(),
                name: name.to_owned(),
                span: self.cmd.span,
            })
            .and_then(T::from_value)
    }

    pub fn opt_named<T: FromValue>(&self, name: &str) -> Result<Option<T>, ShellError> {
        match self.named.get(name) {
            Some(v) => T::from_value(v.clone()).map(Some),
            None => Ok(None),
        }
    }

    pub fn has_switch(&self, name: &str) -> bool {
        match self.named.get(name) {
            Some(Value::Bool(v)) => v.item,
            _ => false,
        }
    }

    pub fn rest<T: FromValue>(&self) -> Result<Vec<T>, ShellError> {
        self.rest.iter().cloned().map(T::from_value).collect()
    }

    pub fn get_positional(&self, index: usize) -> Option<&Value> {
        self.positionals.get(index)
    }

    pub fn get_named(&self, name: &str) -> Option<&Value> {
        self.named.get(name)
    }

    pub fn is_empty(&self) -> bool {
        self.positionals.is_empty() && self.named.is_empty()
    }

    pub fn rest_strings(&self) -> impl Iterator<Item = &str> {
        self.rest.iter().filter_map(|v| match v {
            Value::String(spanned) => Some(spanned.item.as_str()),
            _ => None,
        })
    }
}
