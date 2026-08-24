use crate::error::shell_error::ShellError;
use crate::parser::span::Span;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Default)]
pub struct Signals {
    interrupted: Arc<AtomicBool>,
}

impl Signals {
    pub fn new() -> Self {
        Self {
            interrupted: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_interrupted(&self) -> bool {
        self.interrupted.load(Ordering::Relaxed)
    }

    pub fn trigger(&self) {
        self.interrupted.store(true, Ordering::Relaxed);
    }

    pub fn reset(&self) {
        self.interrupted.store(false, Ordering::Relaxed);
    }

    pub fn check_interrupted(&self, span: Span) -> Result<(), ShellError> {
        if self.is_interrupted() {
            Err(ShellError::Interrupted { span })
        } else {
            Ok(())
        }
    }
}
