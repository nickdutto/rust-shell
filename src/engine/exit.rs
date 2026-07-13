pub struct ExitCode(pub i32);

impl ExitCode {
    pub const SUCCESS: Self = Self(0);
    pub const FAILURE: Self = Self(1);
    pub const SYNTAX_ERROR: Self = Self(2);
    pub const NOT_FOUND: Self = Self(127);

    pub fn as_i32(&self) -> i32 {
        self.0
    }
}
