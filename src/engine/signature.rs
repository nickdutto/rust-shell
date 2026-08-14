use crate::error::shell_error::ShellError;
use crate::parser::argument::ParsedArguments;
use crate::parser::span::Spanned;
use crate::parser::syntax_shape::SyntaxShape;
use crate::parser::value::Value;

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

    pub fn parse(
        &self,
        cmd: &Spanned<String>,
        raw_args: Vec<Spanned<String>>,
    ) -> Result<ParsedArguments, ShellError> {
        if self.allows_unknown_args {
            return Ok(ParsedArguments {
                cmd: cmd.clone(),
                raw_args,
                ..ParsedArguments::default()
            });
        }

        let mut parsed = ParsedArguments {
            cmd: cmd.clone(),
            raw_args: raw_args.clone(),
            ..ParsedArguments::default()
        };

        self.parse_arguments(raw_args, &mut parsed)?;
        self.validate_required(cmd, &mut parsed)?;

        Ok(parsed)
    }

    fn parse_arguments(
        &self,
        raw_args: Vec<Spanned<String>>,
        parsed: &mut ParsedArguments,
    ) -> Result<(), ShellError> {
        let mut stop_flags = false;
        let mut raw_args_iter = raw_args.into_iter().peekable();
        let mut pos_args_iter = self.positionals.iter();

        while let Some(arg) = raw_args_iter.next() {
            if !stop_flags && arg.item == "--" {
                stop_flags = true;
                continue;
            }

            if !stop_flags && arg.item.starts_with('-') && arg.item != "-" {
                let named_arg = self.find_named(&arg)?;

                if named_arg.shape == SyntaxShape::Bool {
                    parsed.named.insert(
                        named_arg.name.to_owned(),
                        Value::Bool(Spanned::new(true, arg.span)),
                    );
                } else {
                    let value = raw_args_iter.next().ok_or_else(|| {
                        ShellError::MissingNamedArgumentValue {
                            name: named_arg.name.to_owned(),
                            span: arg.span,
                        }
                    })?;

                    parsed.named.insert(
                        named_arg.name.to_owned(),
                        named_arg.shape.parse_value(value)?,
                    );
                }
            } else if let Some(pos_arg) = pos_args_iter.next() {
                parsed.positionals.push(pos_arg.shape.parse_value(arg)?);
            } else if let Some(rest_pos_arg) = &self.rest_positional {
                parsed.rest.push(rest_pos_arg.shape.parse_value(arg)?);
            } else {
                return Err(ShellError::TooManyArguments {
                    cmd: self.name.to_owned(),
                    span: arg.span,
                });
            }
        }

        Ok(())
    }

    fn validate_required(
        &self,
        cmd: &Spanned<String>,
        parsed: &mut ParsedArguments,
    ) -> Result<(), ShellError> {
        for (i, pos_arg) in self.positionals.iter().enumerate() {
            if pos_arg.required && i >= parsed.positionals.len() {
                return Err(ShellError::MissingPositionalArgument {
                    cmd: self.name.to_owned(),
                    name: pos_arg.name.to_owned(),
                    span: cmd.span,
                });
            }
        }

        for named_arg in &self.named {
            if named_arg.required && !parsed.named.contains_key(named_arg.name) {
                return Err(ShellError::MissingNamedArgument {
                    cmd: self.name.to_owned(),
                    name: named_arg.name.to_owned(),
                    span: cmd.span,
                });
            }
        }

        Ok(())
    }

    fn find_named(&self, arg: &Spanned<String>) -> Result<&NamedArg, ShellError> {
        let name_str = &arg.item;

        let matched = if let Some(long) = name_str.strip_prefix("--") {
            self.named.iter().find(|n| n.name == long)
        } else if let Some(short_str) = name_str.strip_prefix("-") {
            let short = short_str.chars().next();
            self.named.iter().find(|n| n.short == short)
        } else {
            None
        };

        matched.ok_or_else(|| ShellError::UnknownNamedArgument {
            cmd: self.name.to_owned(),
            name: arg.item.clone(),
            span: arg.span,
        })
    }
}
