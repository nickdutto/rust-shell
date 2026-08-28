use crate::error::shell_error::ShellError;
use crate::value::from_value::FromValue;
use crate::value::span::{Span, Spanned};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum Value {
    Bool(Spanned<bool>),
    Int(Spanned<i64>),
    String(Spanned<String>),
}

impl Value {
    pub fn as_bool(&self) -> Result<bool, ShellError> {
        match self {
            Value::Bool(v) => Ok(v.item),
            v => Err(ShellError::type_mismatch(
                Self::expected_type(),
                v.type_name(),
                v.span(),
            )),
        }
    }

    pub fn as_int(&self) -> Result<i64, ShellError> {
        match self {
            Value::Int(v) => Ok(v.item),
            v => Err(ShellError::type_mismatch(
                Self::expected_type(),
                v.type_name(),
                v.span(),
            )),
        }
    }

    pub fn as_str(&self) -> Result<&str, ShellError> {
        match self {
            Value::String(v) => Ok(&v.item),
            v => Err(ShellError::type_mismatch(
                Self::expected_type(),
                v.type_name(),
                v.span(),
            )),
        }
    }

    pub fn into_string(self) -> Result<String, ShellError> {
        match self {
            Value::String(v) => Ok(v.item),
            v => Err(ShellError::type_mismatch(
                Self::expected_type(),
                v.type_name(),
                v.span(),
            )),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::String(_) => "string",
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Value::Bool(sp) => sp.span,
            Value::Int(sp) => sp.span,
            Value::String(sp) => sp.span,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(sp) => write!(f, "{}", sp.item),
            Value::Int(sp) => write!(f, "{}", sp.item),
            Value::Bool(sp) => write!(f, "{}", sp.item),
        }
    }
}
