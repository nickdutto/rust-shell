use crate::error::shell_error::ShellError;
use crate::value::data_size_unit::DataSizeUnit;
use crate::value::span::Spanned;
use crate::value::value::Value;
use std::str::FromStr;

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
            value => Err(ShellError::type_mismatch(
                Self::expected_type(),
                value.type_name(),
                value.span(),
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
            value => Err(ShellError::type_mismatch(
                Self::expected_type(),
                value.type_name(),
                value.span(),
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
            value => Err(ShellError::type_mismatch(
                Self::expected_type(),
                value.type_name(),
                value.span(),
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

impl FromValue for DataSizeUnit {
    fn expected_type() -> &'static str {
        "DataSizeUnit"
    }

    fn from_value(v: Value) -> Result<Self, ShellError> {
        match v {
            Value::String(sp) => DataSizeUnit::from_str(&sp.item),
            value => Err(ShellError::type_mismatch(
                Self::expected_type(),
                value.type_name(),
                value.span(),
            )),
        }
    }
}
