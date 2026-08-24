use std::process::ExitStatus;

#[derive(PartialEq)]
pub struct ExitCode(pub i32);

impl ExitCode {
    pub const SUCCESS: Self = Self(0);
    pub const FAILURE: Self = Self(1);
    pub const SYNTAX_ERROR: Self = Self(2);
    pub const NOT_FOUND: Self = Self(127);
    pub const SIGINT: Self = Self(130);

    pub fn as_i32(&self) -> i32 {
        self.0
    }
}

impl From<i32> for ExitCode {
    fn from(code: i32) -> Self {
        Self(code)
    }
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> Self {
        code.0
    }
}

impl From<ExitStatus> for ExitCode {
    fn from(status: ExitStatus) -> Self {
        match status.code() {
            Some(code) => Self(code),
            None => {
                #[cfg(unix)]
                {
                    use std::os::unix::process::ExitStatusExt;
                    if let Some(signal) = status.signal() {
                        return Self(128 + signal);
                    }
                }

                Self::FAILURE
            }
        }
    }
}
