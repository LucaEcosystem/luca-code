//! CLI subprocess integration tests (M1b-3, folds in M1a F1).
//!
//! Every test spawns the compiled `luca` binary via `CARGO_BIN_EXE_luca`
//! (no shell), runs inside an isolated temporary directory that is
//! removed on drop even when assertions fail, and asserts exact exit
//! status, stdout bytes, and stderr bytes. Captured output uses `\n`
//! line endings on all platforms (Rust never translates piped output).
use std::{
    path::PathBuf,
    process::{Command, Output},
};

/// Isolated scratch directory, removed on drop even if a test panics.
struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(tag: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "luca_cli_{}_{}_{}",
            tag,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn write(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.path.join(name);
        std::fs::write(&path, contents).unwrap();
        path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn run_cli(args: &[&str], cwd: &std::path::Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_luca"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("failed to spawn luca CLI")
}

fn stdout(output: &Output) -> &str {
    std::str::from_utf8(&output.stdout).unwrap()
}

fn stderr(output: &Output) -> &str {
    std::str::from_utf8(&output.stderr).unwrap()
}

#[test]
fn cli_runs_successful_program() {
    let dir = TempDir::new("success");
    let main = dir.write("main.lucc", "print(\"a\")\nprint(\"b\")\n");
    let output = run_cli(&[main.to_str().unwrap()], &dir.path);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "a\nb\n");
    assert_eq!(stderr(&output), "");
}

#[test]
fn cli_exits_with_requested_code_after_flushing_output() {
    let dir = TempDir::new("exit");
    let main = dir.write(
        "main.lucc",
        "import System\nprint(\"one\")\nprint(\"two\")\nSystem.exit(7)\nprint(\"never\")\n",
    );
    let output = run_cli(&[main.to_str().unwrap()], &dir.path);
    assert_eq!(output.status.code(), Some(7));
    assert_eq!(stdout(&output), "one\ntwo\n");
    assert_eq!(stderr(&output), "Warning at 5:1: Unreachable code\n");
}

#[test]
fn cli_exits_inside_function() {
    let dir = TempDir::new("exit_func");
    let main = dir.write(
        "main.lucc",
        "import System\ndef func quit():\n    System.exit(3)\nprint(\"a\")\nquit()\n",
    );
    let output = run_cli(&[main.to_str().unwrap()], &dir.path);
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(stdout(&output), "a\n");
    assert_eq!(stderr(&output), "");
}

#[test]
fn cli_exits_during_module_init() {
    let dir = TempDir::new("exit_module");
    dir.write("Boomer.lucc", "import System\nSystem.exit(5)\n");
    let main = dir.write("main.lucc", "import Boomer\nprint(\"no\")\n");
    let output = run_cli(&[main.to_str().unwrap()], &dir.path);
    assert_eq!(output.status.code(), Some(5));
    assert_eq!(stdout(&output), "");
    assert_eq!(stderr(&output), "");
}

#[test]
fn cli_exit_runs_finally_and_skips_capture() {
    let dir = TempDir::new("exit_finally");
    let main = dir.write(
        "main.lucc",
        "import System\ntry to :\n    print(\"t\")\n    System.exit(9)\ncapture error :\n    print(\"caught\")\nfinally :\n    print(\"fin\")\n",
    );
    let output = run_cli(&[main.to_str().unwrap()], &dir.path);
    assert_eq!(output.status.code(), Some(9));
    assert_eq!(stdout(&output), "t\nfin\n");
    assert_eq!(stderr(&output), "");
}

#[test]
fn cli_reports_runtime_error_without_flushing_output() {
    let dir = TempDir::new("error");
    let main = dir.write("main.lucc", "print(\"kept\")\nprint(10 / 0)\n");
    let output = run_cli(&[main.to_str().unwrap()], &dir.path);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(stdout(&output), "");
    assert_eq!(stderr(&output), "Error at 2:10: Division by zero\n");
}

#[test]
fn cli_usage_errors_exit_2() {
    let dir = TempDir::new("usage");
    let no_args = run_cli(&[], &dir.path);
    assert_eq!(no_args.status.code(), Some(2));
    assert!(stderr(&no_args).starts_with("Usage: luca "));

    let bad_ext = dir.write("note.txt", "hello\n");
    let output = run_cli(&[bad_ext.to_str().unwrap()], &dir.path);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(stderr(&output), "Luca Code source files must use the .lucc extension\n");
}

#[test]
fn cli_check_accepts_valid_program() {
    let dir = TempDir::new("check_ok");
    let main = dir.write("main.lucc", "print(\"hi\")\n");
    let output = run_cli(&["check", main.to_str().unwrap()], &dir.path);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "");
    let json = run_cli(&["check", main.to_str().unwrap(), "--format", "json"], &dir.path);
    assert_eq!(json.status.code(), Some(0));
    assert_eq!(stdout(&json), "[]\n");
}

#[test]
fn cli_check_reports_syntax_errors_in_both_formats() {
    let dir = TempDir::new("check_bad");
    let main = dir.write("main.lucc", "def var = 1\n");
    let human = run_cli(&["check", main.to_str().unwrap()], &dir.path);
    assert_eq!(human.status.code(), Some(1));
    assert!(stdout(&human).starts_with("Error at "));
    assert_eq!(stderr(&human), "");
    let json = run_cli(&["check", main.to_str().unwrap(), "--format=json"], &dir.path);
    assert_eq!(json.status.code(), Some(1));
    assert!(stdout(&json).starts_with("[{\"message\":"));
    assert!(stdout(&json).ends_with("}]\n"));
    assert_eq!(stderr(&json), "");
}

#[test]
fn cli_check_missing_entry_is_internal_failure() {
    let dir = TempDir::new("check_missing");
    let missing = dir.path.join("nope.lucc");
    let output = run_cli(&["check", missing.to_str().unwrap()], &dir.path);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(stdout(&output), "");
    assert!(stderr(&output).starts_with("luca check: Could not read "));
}
