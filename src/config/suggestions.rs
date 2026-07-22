use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct Suggestions {
    pub cwd_aware: bool,
    pub min_chars: usize,
    pub color: u8,
}

impl Default for Suggestions {
    fn default() -> Self {
        Self {
            cwd_aware: false,
            min_chars: 1,
            color: 8,
        }
    }
}
