use crate::error::shell_error::ShellError;
use crate::parser::argument::ParsedArguments;
use crate::parser::shape::SyntaxShape;
use crate::parser::span::Spanned;
use crate::parser::value::Value;
use std::collections::HashMap;

pub struct PositionalArg {
    pub name: &'static str,
    pub shape: SyntaxShape,
    pub required: bool,
    pub description: &'static str,
}

pub struct NamedArg {
    pub name: &'static str,
    pub short: Option<char>,
    pub shape: SyntaxShape,
    pub required: bool,
    pub description: &'static str,
}

pub struct Signature {
    name: &'static str,
    positionals: Vec<PositionalArg>,
    rest_positional: Option<PositionalArg>,
    named: Vec<NamedArg>,
    allows_unknown_args: bool,
}

impl Signature {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            positionals: vec![],
            named: vec![],
            rest_positional: None,
            allows_unknown_args: false,
        }
    }

    pub fn allow_unknown_args(mut self, allow: bool) -> Self {
        self.allows_unknown_args = allow;
        self
    }

    pub fn positional(
        mut self,
        name: &'static str,
        shape: SyntaxShape,
        description: &'static str,
    ) -> Self {
        self.positionals.push(PositionalArg {
            name,
            shape,
            required: false,
            description,
        });

        self
    }

    pub fn required_positional(
        mut self,
        name: &'static str,
        shape: SyntaxShape,
        description: &'static str,
    ) -> Self {
        self.positionals.push(PositionalArg {
            name,
            shape,
            required: true,
            description,
        });

        self
    }

    pub fn rest(
        mut self,
        name: &'static str,
        shape: SyntaxShape,
        description: &'static str,
    ) -> Self {
        self.rest_positional = Some(PositionalArg {
            name,
            shape,
            required: false,
            description,
        });

        self
    }

    pub fn named(
        mut self,
        name: &'static str,
        shape: SyntaxShape,
        description: &'static str,
        short: Option<char>,
    ) -> Self {
        self.named.push(NamedArg {
            name,
            short,
            shape,
            required: false,
            description,
        });

        self
    }

    pub fn required_named(
        mut self,
        name: &'static str,
        shape: SyntaxShape,
        description: &'static str,
        short: Option<char>,
    ) -> Self {
        self.named.push(NamedArg {
            name,
            short,
            shape,
            required: true,
            description,
        });

        self
    }

    pub fn switch(
        mut self,
        name: &'static str,
        description: &'static str,
        short: Option<char>,
    ) -> Self {
        self.named.push(NamedArg {
            name,
            short,
            shape: SyntaxShape::Bool,
            required: false,
            description,
        });

        self
    }

    pub fn parse(&self, raw_args: Vec<Spanned<String>>) -> Result<ParsedArguments, ShellError> {
        if self.allows_unknown_args {
            return Ok(ParsedArguments {
                positionals: vec![],
                rest: vec![],
                named: HashMap::new(),
                raw_args,
            });
        }

        let mut parsed = ParsedArguments {
            raw_args: raw_args.clone(),
            ..ParsedArguments::default()
        };

        let mut stop_flags = false;
        let mut raw_args_iter = raw_args.into_iter().peekable();
        let mut pos_specs_iter = self.positionals.iter();

        while let Some(arg) = raw_args_iter.next() {
            if !stop_flags && arg.item == "--" {
                stop_flags = true;
                continue;
            }

            if !stop_flags && arg.item.starts_with('-') && arg.item != "-" {
                let (named_arg, _) = self.find_named(&arg)?;

                if named_arg.shape == SyntaxShape::Bool {
                    parsed.named.insert(
                        named_arg.name.to_owned(),
                        Value::Bool(Spanned::new(true, arg.span)),
                    );
                } else {
                    let value = raw_args_iter.next().ok_or_else(|| {
                        ShellError::Generic(format!(
                            "Named arg '--{}' requires a value",
                            named_arg.name
                        ))
                    })?;

                    let parsed_value = parse_value_shape(value, &named_arg.shape)?;
                    parsed.named.insert(named_arg.name.to_owned(), parsed_value);
                }
            } else if let Some(pos_spec) = pos_specs_iter.next() {
                let parsed_value = parse_value_shape(arg, &pos_spec.shape)?;
                parsed.positionals.push(parsed_value);
            } else if let Some(rest_spec) = &self.rest_positional {
                let parsed_value = parse_value_shape(arg, &rest_spec.shape)?;
                parsed.rest.push(parsed_value);
            } else {
                return Err(ShellError::Generic(format!(
                    "Too many arguments passed to command: {}",
                    self.name
                )));
            }
        }

        for (i, pos_arg) in self.positionals.iter().enumerate() {
            if pos_arg.required && i >= parsed.positionals.len() {
                return Err(ShellError::Generic(format!(
                    "Missing required positional argument: {}",
                    pos_arg.name
                )));
            }
        }

        for named_arg in &self.named {
            if named_arg.required && !parsed.named.contains_key(named_arg.name) {
                return Err(ShellError::Generic(format!(
                    "Missing required named arg: --{}",
                    named_arg.name
                )));
            }
        }

        Ok(parsed)
    }

    fn find_named(&self, arg: &Spanned<String>) -> Result<(&NamedArg, String), ShellError> {
        let name_str = &arg.item;

        if let Some(long) = name_str.strip_prefix("--") {
            self.named
                .iter()
                .find(|n| n.name == long)
                .map(|na| (na, long.to_owned()))
                .ok_or_else(|| ShellError::Generic(format!("Unknown named arg (long): --{long}")))
        } else if let Some(short_str) = name_str.strip_prefix("-") {
            let short = short_str.chars().next().unwrap_or_default();
            self.named
                .iter()
                .find(|n| n.short == Some(short))
                .map(|na| (na, short.to_string()))
                .ok_or_else(|| ShellError::Generic(format!("Unknown named arg (short): --{short}")))
        } else {
            Err(ShellError::Generic("Error finding named arg".to_owned()))
        }
    }
}

fn parse_value_shape(raw: Spanned<String>, shape: &SyntaxShape) -> Result<Value, ShellError> {
    match shape {
        SyntaxShape::Bool => {
            let parsed = raw
                .item
                .parse::<bool>()
                .map_err(|_| ShellError::Generic(format!("Expected boolean, got {}", raw.item)))?;

            Ok(Value::Bool(Spanned::new(parsed, raw.span)))
        }

        SyntaxShape::Int => {
            let parsed = raw
                .item
                .parse::<i64>()
                .map_err(|_| ShellError::Generic(format!("Expected int, got {}", raw.item)))?;

            Ok(Value::Int(Spanned::new(parsed, raw.span)))
        }

        SyntaxShape::String => Ok(Value::String(raw)),
    }
}
