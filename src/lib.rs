pub mod ast;
pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod stdlib;
pub mod types;

pub const PRODUCT_VERSION: &str = "1.0 Beta";

pub fn run(source: &str) -> Result<Vec<String>, error::LucaError> {
    interpreter::Interpreter::new().run_collect(source)
}

pub fn run_with_input(source: &str, input: Vec<String>) -> Result<Vec<String>, error::LucaError> {
    interpreter::Interpreter::new().with_input(input).run_collect(source)
}
