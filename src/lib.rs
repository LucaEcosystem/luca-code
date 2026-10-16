pub mod ast;
pub mod check;
pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod platform;
pub mod stdlib;
pub mod types;

pub const PRODUCT_VERSION: &str = "1.0 Beta";

/// Run a program and collect its output.
///
/// Transitional behavior, retained for existing callers: a valid
/// `System.exit(code)` terminates the host process instead of returning.
/// New embedding code must drive [`interpreter::Interpreter::run_outcome`]
/// directly, which returns the request as data.
pub fn run(source: &str) -> Result<Vec<String>, error::LucaError> {
    interpreter::Interpreter::new().run_collect(source)
}

/// Run with scripted `ask` input. Same transitional behavior as [`run`].
pub fn run_with_input(source: &str, input: Vec<String>) -> Result<Vec<String>, error::LucaError> {
    interpreter::Interpreter::new().with_input(input).run_collect(source)
}

pub use interpreter::RunOutcome;

pub use check::{check_file, check_file_with_platform, check_source, diagnostics_to_json, CheckDiagnostic, CheckFailure};
pub use platform::{FakePlatform, NativePlatform, Platform, PlatformError, PlatformErrorKind};
