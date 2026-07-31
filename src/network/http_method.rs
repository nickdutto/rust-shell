use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HttpMethodError {
    #[error("Invalid HttpMethod `{method}`")]
    InvalidMethod { method: String },
}

#[derive(Debug)]
pub enum HttpMethod {
    Get,
}

impl HttpMethod {
    const ALL: &'static [HttpMethod] = &[HttpMethod::Get];

    pub fn available_methods() -> String {
        Self::ALL
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join(", ")
    }

    pub fn parse(method: &str) -> Result<Self, HttpMethodError> {
        match method.to_lowercase().as_str() {
            "get" => Ok(HttpMethod::Get),
            _ => Err(HttpMethodError::InvalidMethod {
                method: method.to_owned(),
            }),
        }
    }
}

impl Display for HttpMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "get"),
        }
    }
}
