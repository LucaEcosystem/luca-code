use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct LucaError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    /// `Some(code)` marks whole-process termination via `System.exit(code)`.
    /// It is not an error: `try to` never captures it, but `finally` blocks
    /// still run, and pending output is flushed before exiting.
    pub exit_code: Option<i32>,
    /// `Some(value)` carries a raised custom error to the nearest `capture`
    /// block, where it is bound as a dictionary value. `None` for ordinary
    /// runtime errors (whose `message`/`line`/`column` still describe them).
    pub error_value: Option<crate::types::Value>,
}

impl LucaError {
    pub fn new(message: impl Into<String>, line: usize, column: usize) -> Self {
        Self { message: message.into(), line, column, exit_code: None, error_value: None }
    }

    pub fn exit(code: i32, line: usize, column: usize) -> Self {
        Self { message: format!("System.exit({code})"), line, column, exit_code: Some(code), error_value: None }
    }

    pub fn raised(message: String, value: crate::types::Value, line: usize, column: usize) -> Self {
        Self { message, line, column, exit_code: None, error_value: Some(value) }
    }

    pub fn is_exit(&self) -> bool {
        self.exit_code.is_some()
    }
}

impl fmt::Display for LucaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Error at {}:{}: {}", self.line, self.column, self.message)
    }
}

impl std::error::Error for LucaError {}
