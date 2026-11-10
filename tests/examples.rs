//! End-to-end checks for the programs shipped in `examples/`.
//!
//! These run the real `luca` binary from the repository root, so they cover the
//! CLI, module resolution, stdin handling, and the documented output in the
//! examples gallery. If an example's behavior changes, its expected output here
//! must change with it.
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn example_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

/// Run `luca <relative>` from the repository root.
fn run_example(relative: &str, stdin: Option<&str>) -> (i32, String, String) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_luca"));
    command
        .arg(example_path(relative))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().expect("luca binary starts");
    if let Some(input) = stdin {
        child
            .stdin
            .as_mut()
            .expect("stdin is piped")
            .write_all(input.as_bytes())
            .expect("stdin write");
    }
    let output = child.wait_with_output().expect("luca finishes");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn assert_example(relative: &str, expected_stdout: &[&str]) {
    let (code, stdout, stderr) = run_example(relative, None);
    assert_eq!(code, 0, "{relative} exited with {code}; stderr:\n{stderr}");
    assert_eq!(
        stdout,
        format!("{}\n", expected_stdout.join("\n")),
        "stdout for {relative}"
    );
    assert_eq!(stderr, "", "{relative} wrote to stderr");
}

#[test]
fn hello_example() {
    assert_example(
        "examples/hello.lucc",
        &["Hello, Luca!", "Visit 2", "Six divided by two is:", "3.0"],
    );
}

#[test]
fn arithmetic_example() {
    assert_example(
        "examples/arithmetic.lucc",
        &["2.5", "14", "40", "6", "2", "21.50", "5.00", "0.3"],
    );
}

#[test]
fn inference_example() {
    assert_example(
        "examples/inference.lucc",
        &["Hello, Luca! Visit 2", "3.14", "none", "5"],
    );
}

#[test]
fn comparison_example() {
    assert_example(
        "examples/comparison.lucc",
        &[
            "true", "true", "true", "true", "true", "false", "true", "true", "true",
        ],
    );
}

#[test]
fn conditional_example() {
    assert_example(
        "examples/conditional.lucc",
        &[
            "Hello, Luca!",
            "B",
            "nested",
            "after inner",
            "and true",
            "not false",
            "inner",
            "outer",
        ],
    );
}

#[test]
fn functions_example() {
    assert_example(
        "examples/functions.lucc",
        &[
            "Hello, Luca!",
            "Hello, Alex!",
            "5",
            "30",
            "120",
            "2",
            "15",
            "5",
            "hi",
        ],
    );
}

#[test]
fn loops_example() {
    assert_example(
        "examples/loops.lucc",
        &[
            "5", "10", "hi", "hi", "hi", "3", "13", "nested", "nested", "nested", "nested", "5",
            "4", "3", "done", "3", "2", "1",
        ],
    );
}

#[test]
fn collections_example() {
    assert_example(
        "examples/collections.lucc",
        &[
            "[\"Luca\", \"Alex\"]",
            "Luca",
            "2",
            "true",
            "[\"Luca\", \"Alex\", \"Sam\"]",
            "[\"Alex\", \"Sam\"]",
            "(123, true, \"Luca\")",
            "Luca",
            "(456, true, \"Luca\")",
            "{\"alpha\", \"stable\"}",
            "true",
            "{\"alpha\", \"stable\", \"beta\"}",
            "{\"Name\" = \"Luca\", \"Age\" = 16}",
            "Luca",
            "true",
            "true",
            "{\"Name\" = \"Luca\", \"Age\" = 16, \"City\" = \"Berlin\"}",
            "{\"Name\" = \"Luca\", \"Age\" = 17, \"City\" = \"Berlin\"}",
            "Alex",
            "Sam",
            "true",
            "2",
            "2",
        ],
    );
}

#[test]
fn modules_example_resolves_sibling_file() {
    assert_example(
        "examples/modules/main.lucc",
        &[
            "hi World", "hi Luca", "1.0", "demo", "true", "1", "2", "demo", "modules",
        ],
    );
}

#[test]
fn errors_example() {
    assert_example(
        "examples/errors.lucc",
        &[
            "caught division",
            "cleanup",
            "no trouble",
            "always",
            "inner finally",
            "outer caught",
            "5.0",
            "0",
            "503",
            "unavailable",
            "41",
            "500",
            "boom",
        ],
    );
}

#[test]
fn ask_example_reads_stdin() {
    let (code, stdout, stderr) = run_example("examples/ask.lucc", Some("Luca\n16\n"));
    assert_eq!(code, 0, "stderr:\n{stderr}");
    assert_eq!(
        stdout,
        "What is your name?\nHow old are you?\nHello, Luca. You are 16 years old.\n"
    );
    assert_eq!(stderr, "");
}

#[test]
fn stdlib_example_warns_about_the_missing_demo_file() {
    let (code, stdout, stderr) = run_example("examples/stdlib.lucc", None);
    assert_eq!(code, 0, "stderr:\n{stderr}");
    assert_eq!(
        stdout,
        "5.0\n1024.0\n3\n1\nHELLO\na-b-c\ntrue\ntrue\ntrue\ntrue\n"
    );
    assert_eq!(
        stderr,
        "Warning at 14:1: File 'examples/.no-such-demo-file' does not exist\n"
    );
}

#[test]
fn text_example() {
    assert_example(
        "examples/text.lucc",
        &[
            "Luca Code",
            "LUCA CODE",
            "luca code",
            "Luca-Code",
            "[\"list\", \"tuple\", \"set\", \"dict\"]",
            "list / tuple / set",
            "[\"\", \"a\", \"b\", \"c\", \"\"]",
            ".a.b.c.",
        ],
    );
}

#[test]
fn math_example() {
    assert_example(
        "examples/math.lucc",
        &[
            "5.0",
            "1024.0",
            "7",
            "2.50",
            "3",
            "-3",
            "-3",
            "-2",
            "1.50",
            "4",
            "2.5",
            "0.3333333333333333333333333333",
            "0.25",
        ],
    );
}

#[test]
fn files_example_cleans_up() {
    let temp = example_path("luca-example.tmp");
    let _ = std::fs::remove_file(&temp);
    assert_example(
        "examples/files.lucc",
        &["true", "first and second", "false"],
    );
    assert!(!temp.exists(), "files.lucc must delete its temporary file");
}

#[test]
fn utility_example() {
    assert_example(
        "examples/utility.lucc",
        &[
            "32.0",
            "212.0",
            "0",
            "42",
            "100",
            "3",
            "[\"alpha\", \"beta\", true, 3.50]",
            "now a scalar",
        ],
    );
}

#[test]
fn integration_example() {
    assert_example(
        "examples/integration.lucc",
        &[
            "skipped oops",
            "35.50",
            "skipped oops",
            "1 over 10.00",
            "{\"retries\" = 5}",
            "true",
            "no crash",
            "done",
        ],
    );
}
