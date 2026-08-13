use crate::error::shell_error::ShellError;
use crate::parser::span::{Span, Spanned};

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

pub trait FromValue: Sized {
    fn expected_type() -> &'static str {
        std::any::type_name::<Self>()
            .split("::")
            .last()
            .unwrap_or("value")
    }

    fn from_value(v: Value) -> Result<Self, ShellError>;
}

impl FromValue for Value {
    fn from_value(v: Value) -> Result<Self, ShellError> {
        Ok(v)
    }
}

impl FromValue for bool {
    fn expected_type() -> &'static str {
        "bool"
    }

    fn from_value(v: Value) -> Result<Self, ShellError> {
        match v {
            Value::Bool(sp) => Ok(sp.item),
            v => Err(ShellError::type_mismatch(
                Self::expected_type(),
                v.type_name(),
                v.span(),
            )),
        }
    }
}

impl FromValue for i64 {
    fn expected_type() -> &'static str {
        "int"
    }

    fn from_value(v: Value) -> Result<Self, ShellError> {
        match v {
            Value::Int(sp) => Ok(sp.item),
            v => Err(ShellError::type_mismatch(
                Self::expected_type(),
                v.type_name(),
                v.span(),
            )),
        }
    }
}

impl FromValue for String {
    fn expected_type() -> &'static str {
        "string"
    }

    fn from_value(v: Value) -> Result<Self, ShellError> {
        match v {
            Value::String(sp) => Ok(sp.item),
            v => Err(ShellError::type_mismatch(
                Self::expected_type(),
                v.type_name(),
                v.span(),
            )),
        }
    }
}

impl<T> FromValue for Option<T>
where
    T: FromValue,
{
    fn expected_type() -> &'static str {
        T::expected_type()
    }

    fn from_value(v: Value) -> Result<Self, ShellError> {
        T::from_value(v).map(Some)
    }
}

impl<T> FromValue for Vec<T>
where
    T: FromValue,
{
    fn expected_type() -> &'static str {
        T::expected_type()
    }

    fn from_value(v: Value) -> Result<Self, ShellError> {
        T::from_value(v).map(|val| vec![val])
    }
}

impl<T> FromValue for Spanned<T>
where
    T: FromValue,
{
    fn expected_type() -> &'static str {
        T::expected_type()
    }

    fn from_value(v: Value) -> Result<Self, ShellError> {
        let span = v.span();
        Ok(Spanned {
            item: T::from_value(v)?,
            span,
        })
    }
}
