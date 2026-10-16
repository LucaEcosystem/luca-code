//! Read-only validation for Luca Code (`luca check`).
//!
//! This module never executes application code. It only:
//!
//! - lexes (`crate::lexer::lex`) and parses (`crate::parser::parse`)
//! - resolves `import Name` to `Name.lucc` via explicit search paths
//! - reads files through the injected [`crate::platform::Platform`]
//!   (validation input only; never writes, never calls `stdlib`, never
//!   reads stdin, never prints application output, never exits the process;
//!   [`check_file`] uses [`crate::platform::NativePlatform`])
//!
//! Dynamic checks that require execution (call arity with runtime values,
//! division by zero on computed values, `ask` conversion of actual input,
//! `File`/`System` effects, declaration type checks on runtime values) stay
//! in the interpreter and are out of scope for `check`.

use std::{
    collections::HashSet,
    fmt,
    path::{Path, PathBuf},
};

use crate::{
    ast::{Block, Program, Statement},
    error::LucaError,
    platform::{NativePlatform, Platform},
};

/// Maximum number of distinct module files `check_file` will read.
/// Bounds filesystem work and prevents runaway import graphs.
const MAX_CHECK_FILES: usize = 100;

/// Maximum import nesting depth. Mirrors the interpreter's cycle guard
/// (`loading` stack) with an additional depth cap for safety.
const MAX_CHECK_DEPTH: usize = 32;

/// A single validation error found without executing code.
///
/// `Display` is intentionally identical to [`LucaError`]'s
/// (`Error at line:column: message`) so existing tooling that parses
/// `Error at (\d+):(\d+)` keeps working.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckDiagnostic {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl CheckDiagnostic {
    pub fn new(message: impl Into<String>, line: usize, column: usize) -> Self {
        Self { message: message.into(), line, column }
    }
}

impl From<LucaError> for CheckDiagnostic {
    fn from(error: LucaError) -> Self {
        Self { message: error.message, line: error.line, column: error.column }
    }
}

impl fmt::Display for CheckDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Error at {}:{}: {}", self.line, self.column, self.message)
    }
}

impl std::error::Error for CheckDiagnostic {}

/// An internal validator failure (not a validation error in the source).
///
/// Examples: the entry file could not be read, the entry path is unusable.
/// These are reported with a `luca check:` prefix and exit code 2, never as
/// `Error at line:column` diagnostics and never as JSON diagnostic arrays.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckFailure {
    pub message: String,
}

impl CheckFailure {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

impl fmt::Display for CheckFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "luca check: {}", self.message)
    }
}

impl std::error::Error for CheckFailure {}

/// Validate a source string without executing it and without touching the
/// filesystem.
///
/// Returns every lexer/parser error as a diagnostic (at most one: lexing
/// and parsing stop at the first error, matching `run`). Imports are not
/// resolved here; use [`check_file`] when file resolution is wanted.
/// An empty vector means the source is syntactically valid.
pub fn check_source(source: &str) -> Vec<CheckDiagnostic> {
    let tokens = match crate::lexer::lex(source) {
        Ok(tokens) => tokens,
        Err(error) => return vec![error.into()],
    };
    match crate::parser::parse(tokens) {
        Ok(_) => Vec::new(),
        Err(error) => vec![error.into()],
    }
}

/// Validate a `.lucc` file plus its transitive `import` graph without
/// executing any application code.
///
/// `search_paths` uses the same convention as the `luca run` CLI:
/// program-directory first, then working directory (constructed by the
/// caller; see `src/main.rs`).
///
/// Returns `Ok(diagnostics)` where an empty vector means valid. Returns
/// `Err(CheckFailure)` only for internal validator failures (entry file
/// unreadable). Problems inside the source graph — including missing or
/// unreadable *imported* modules, circular imports, and syntax errors in any
/// file — are `Ok` diagnostics, mirroring the interpreter's messages
/// (`missing module 'X'`, `Circular import 'X'`,
/// `Could not read module 'X': ...`, `In module 'X': ...`).
pub fn check_file(entry: &Path, search_paths: &[PathBuf]) -> Result<Vec<CheckDiagnostic>, CheckFailure> {
    check_file_with_platform(entry, search_paths, &NativePlatform)
}

/// Validate a `.lucc` file plus its transitive `import` graph through an
/// injected [`Platform`], without executing any application code.
///
/// Same contract as [`check_file`], except every filesystem read and every
/// module-candidate probe goes through `platform` instead of the host
/// directly. `NativePlatform` reproduces `check_file` exactly; fakes enable
/// fully deterministic validation tests.
///
/// Unsaved-buffer limitation: validation is file-graph oriented — the entry
/// and every import resolve through filesystem paths. A buffer with no
/// path can only be syntax-checked via [`check_source`]; resolving its
/// imports would require a virtual entry identity, which is future work,
// a deliberate non-goal of this additive overload.
pub fn check_file_with_platform(
    entry: &Path,
    search_paths: &[PathBuf],
    platform: &dyn Platform,
) -> Result<Vec<CheckDiagnostic>, CheckFailure> {
    let source = platform.read_file(entry)
        .map_err(|e| CheckFailure::new(format!("Could not read {}: {e}", entry.display())))?;
    let mut checker = Checker {
        search_paths: search_paths.to_vec(),
        visited: HashSet::new(),
        loading: Vec::new(),
        diagnostics: Vec::new(),
        platform,
    };
    checker.check_source_graph(&source);
    Ok(checker.diagnostics)
}

struct Checker<'a> {
    search_paths: Vec<PathBuf>,
    visited: HashSet<String>,
    loading: Vec<String>,
    diagnostics: Vec<CheckDiagnostic>,
    platform: &'a dyn Platform,
}

impl Checker<'_> {
    fn check_source_graph(&mut self, source: &str) {
        let tokens = match crate::lexer::lex(source) {
            Ok(tokens) => tokens,
            Err(error) => {
                self.diagnostics.push(error.into());
                return;
            }
        };
        let program = match crate::parser::parse(tokens) {
            Ok(program) => program,
            Err(error) => {
                self.diagnostics.push(error.into());
                return;
            }
        };
        self.check_program(&program);
    }

    fn check_program(&mut self, program: &Program) {
        for (name, line, column) in collect_imports(&program.statements) {
            self.check_import(&name, line, column);
            if self.diagnostics.len() >= MAX_CHECK_FILES {
                break;
            }
        }
    }

    fn check_import(&mut self, name: &str, line: usize, column: usize) {
        // Built-in standard modules never touch the filesystem.
        if crate::stdlib::is_builtin(name) {
            return;
        }
        // Once-only: an already-validated module is not re-read.
        if self.visited.contains(name) {
            return;
        }
        if self.loading.iter().any(|n| n == name) {
            self.diagnostics.push(CheckDiagnostic::new(format!("Circular import '{name}'"), line, column));
            return;
        }
        if self.loading.len() >= MAX_CHECK_DEPTH {
            self.diagnostics.push(CheckDiagnostic::new("Import depth exceeded", line, column));
            return;
        }
        if self.visited.len() >= MAX_CHECK_FILES {
            self.diagnostics.push(CheckDiagnostic::new("Too many modules", line, column));
            return;
        }
        let path = match self.resolve_module_file(name) {
            Some(path) => path,
            None => {
                self.diagnostics.push(CheckDiagnostic::new(format!("missing module '{name}'"), line, column));
                return;
            }
        };
        self.loading.push(name.to_owned());
        let source = match self.platform.read_file(&path) {
            Ok(source) => source,
            Err(e) => {
                self.diagnostics.push(CheckDiagnostic::new(
                    format!("Could not read module '{name}': {e}"),
                    line,
                    column,
                ));
                self.loading.pop();
                self.visited.insert(name.to_owned());
                return;
            }
        };
        let tokens = match crate::lexer::lex(&source) {
            Ok(tokens) => tokens,
            Err(e) => {
                self.diagnostics.push(CheckDiagnostic::new(format!("In module '{name}': {}", e.message), e.line, e.column));
                self.loading.pop();
                self.visited.insert(name.to_owned());
                return;
            }
        };
        let program = match crate::parser::parse(tokens) {
            Ok(program) => program,
            Err(e) => {
                self.diagnostics.push(CheckDiagnostic::new(format!("In module '{name}': {}", e.message), e.line, e.column));
                self.loading.pop();
                self.visited.insert(name.to_owned());
                return;
            }
        };
        self.check_program(&program);
        self.loading.pop();
        self.visited.insert(name.to_owned());
    }

    fn resolve_module_file(&self, name: &str) -> Option<PathBuf> {
        let file_name = format!("{name}.lucc");
        for dir in &self.search_paths {
            let candidate = dir.join(&file_name);
            if self.platform.is_file(&candidate) {
                return Some(candidate);
            }
        }
        None
    }
}

/// Collect every `import Name` in a statement list, including imports nested
/// inside `if`/`try`/`for`/`while`/`func`/`error` blocks.
///
/// Validation is conservative: any import appearing anywhere is resolved,
/// matching the interpreter's ability to execute imports in nested blocks.
fn collect_imports(statements: &[Statement]) -> Vec<(String, usize, usize)> {
    let mut out = Vec::new();
    collect_imports_into(statements, &mut out);
    out
}

fn collect_imports_into(statements: &[Statement], out: &mut Vec<(String, usize, usize)>) {
    for statement in statements {
        match statement {
            Statement::Import { name, line, column } => out.push((name.clone(), *line, *column)),
            Statement::If { branches, else_branch, .. } => {
                for (_, block) in branches {
                    collect_block_into(block, out);
                }
                if let Some(block) = else_branch {
                    collect_block_into(block, out);
                }
            }
            Statement::FuncDef { body, .. } => collect_block_into(body, out),
            Statement::Try { try_block, capture_block, finally_block, .. } => {
                collect_block_into(try_block, out);
                if let Some(block) = capture_block {
                    collect_block_into(block, out);
                }
                if let Some(block) = finally_block {
                    collect_block_into(block, out);
                }
            }
            Statement::ErrorDef { body, .. } => collect_block_into(body, out),
            Statement::ForTimes { body, .. }
            | Statement::ForEach { body, .. }
            | Statement::While { body, .. } => collect_block_into(body, out),
            Statement::Declare { .. }
            | Statement::DeclareCollection { .. }
            | Statement::Assign { .. }
            | Statement::Print { .. }
            | Statement::Forget { .. }
            | Statement::Clear { .. }
            | Statement::Change { .. }
            | Statement::ChangeIndex { .. }
            | Statement::CollectionAdd { .. }
            | Statement::CollectionRemove { .. }
            | Statement::Return { .. }
            | Statement::Call { .. }
            | Statement::MemberCall { .. }
            | Statement::Raise { .. }
            | Statement::Stop { .. }
            | Statement::Skip { .. } => {}
        }
    }
}

fn collect_block_into(block: &Block, out: &mut Vec<(String, usize, usize)>) {
    collect_imports_into(&block.statements, out);
}

/// Render diagnostics as a stable JSON array.
///
/// Format: `[{"message":"...","line":1,"column":2}]` with keys in
/// `message,line,column` order. Hand-rolled escaping keeps the crate
/// dependency-free and offline-capable. Only `"`, `\`, and control
/// characters below `0x20` are escaped (`\n`, `\r`, `\t`, `\b`, `\f`,
/// `\u00XX`); all other characters pass through unescaped.
pub fn diagnostics_to_json(diagnostics: &[CheckDiagnostic]) -> String {
    let mut out = String::from("[");
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str("{\"message\":\"");
        out.push_str(&escape_json_string(&diagnostic.message));
        out.push_str("\",\"line\":");
        out.push_str(&diagnostic.line.to_string());
        out.push_str(",\"column\":");
        out.push_str(&diagnostic.column.to_string());
        out.push('}');
    }
    out.push(']');
    out
}

fn escape_json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_display_matches_luca_error_format() {
        let diagnostic = CheckDiagnostic::new("oops", 3, 7);
        assert_eq!(diagnostic.to_string(), "Error at 3:7: oops");
    }

    #[test]
    fn json_escapes_strings_stably() {
        let diagnostics = vec![CheckDiagnostic::new("say \"hi\"\nbye\\done", 1, 2)];
        assert_eq!(
            diagnostics_to_json(&diagnostics),
            "[{\"message\":\"say \\\"hi\\\"\\nbye\\\\done\",\"line\":1,\"column\":2}]"
        );
        assert_eq!(diagnostics_to_json(&[]), "[]");
    }
}
