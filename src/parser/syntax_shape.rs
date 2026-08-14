use crate::error::shell_error::ShellError;
use crate::parser::span::Spanned;
use crate::parser::value::Value;

#[derive(Debug, PartialEq)]
pub enum SyntaxShape {
    Bool,
    Int,
    String,
}

impl SyntaxShape {
    pub fn parse_value(&self, raw: Spanned<String>) -> Result<Value, ShellError> {
        match self {
            SyntaxShape::Bool => {
                let parsed = raw
                    .item
                    .parse::<bool>()
                    .map_err(|_| ShellError::type_mismatch("bool", "string", raw.span))?;

                Ok(Value::Bool(Spanned::new(parsed, raw.span)))
            }

            SyntaxShape::Int => {
                let parsed = raw
                    .item
                    .parse::<i64>()
                    .map_err(|_| ShellError::type_mismatch("int", "string", raw.span))?;

                Ok(Value::Int(Spanned::new(parsed, raw.span)))
            }

            SyntaxShape::String => Ok(Value::String(raw)),
        }
    }
}
