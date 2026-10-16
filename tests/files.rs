//! Deterministic standard-library filesystem tests through `Platform`
//! (M1b-5 B2).
//!
//! Every test scripts an in-memory filesystem on a shared `FakePlatform`
//! and asserts frozen diagnostics, warning capture, and values. Existing
//! temp-file tests in `tests/interpreter.rs` stay green unmodified as
//! native-parity evidence.
use luca_code::{
    error::LucaError,
    interpreter::{Interpreter, RunOutcome},
    FakePlatform, PlatformError, PlatformErrorKind,
};

fn collect_with_fake(fake: &FakePlatform, source: &str) -> Result<Vec<String>, LucaError> {
    Interpreter::new()
        .with_platform(Box::new(fake.clone()))
        .run_collect(source)
}

fn run_with_fake(fake: &FakePlatform, source: &str) -> Result<(), LucaError> {
    let mut interpreter = Interpreter::new().with_platform(Box::new(fake.clone()));
    interpreter.run(source)
}

#[test]
fn writes_and_reads_round_trip() {
    let fake = FakePlatform::new();
    let output = collect_with_fake(
        &fake,
        "import File\nFile.write(\"/fake/a.txt\", \"hello\")\nprint(File.read(\"/fake/a.txt\"))\n",
    )
    .unwrap();
    assert_eq!(output, ["hello"]);
}

#[test]
fn appends_to_existing_and_new_files() {
    let fake = FakePlatform::new();
    fake.add_file("/fake/a.txt", "a");
    let output = collect_with_fake(
        &fake,
        "import File\nFile.append(\"/fake/a.txt\", \"+b\")\nFile.append(\"/fake/n.txt\", \"n\")\nprint(File.read(\"/fake/a.txt\"))\nprint(File.read(\"/fake/n.txt\"))\n",
    )
    .unwrap();
    assert_eq!(output, ["a+b", "n"]);
}

#[test]
fn reports_existence_accurately() {
    let fake = FakePlatform::new();
    fake.add_file("/fake/a.txt", "x");
    let output = collect_with_fake(
        &fake,
        "import File\nprint(File.exists(\"/fake/a.txt\"))\nprint(File.exists(\"/fake/no.txt\"))\n",
    )
    .unwrap();
    assert_eq!(output, ["true", "false"]);
}

#[test]
fn deletes_and_warns_on_missing_with_none_result() {
    let fake = FakePlatform::new();
    fake.add_file("/fake/d.txt", "x");
    run_with_fake(
        &fake,
        "import File\nFile.delete(\"/fake/d.txt\")\nprint(File.exists(\"/fake/d.txt\"))\nFile.delete(\"/fake/gone.txt\")\nprint(\"after\")\n",
    )
    .unwrap();
    assert_eq!(fake.output(), ["false", "after"]);
    assert_eq!(
        fake.warnings(),
        [(4, 1, "File '/fake/gone.txt' does not exist".to_owned())]
    );
}

#[test]
fn maps_filesystem_errors_with_frozen_messages() {
    let fake = FakePlatform::new();
    fake.add_file("/fake/r.txt", "x");
    fake.fail_file("/fake/r.txt", PlatformError::new(PlatformErrorKind::PermissionDenied, "denied"));
    let error = collect_with_fake(&fake, "import File\nprint(File.read(\"/fake/r.txt\"))\n").unwrap_err();
    assert_eq!(error.to_string(), "Error at 2:11: Could not read file '/fake/r.txt': denied");

    fake.fail_file("/fake/w.txt", PlatformError::new(PlatformErrorKind::PermissionDenied, "denied"));
    let error = collect_with_fake(&fake, "import File\nFile.write(\"/fake/w.txt\", \"x\")\n").unwrap_err();
    assert_eq!(error.to_string(), "Error at 2:1: Could not write file '/fake/w.txt': denied");

    fake.fail_file("/fake/a.txt", PlatformError::new(PlatformErrorKind::PermissionDenied, "denied"));
    let error = collect_with_fake(&fake, "import File\nFile.append(\"/fake/a.txt\", \"x\")\n").unwrap_err();
    assert_eq!(error.to_string(), "Error at 2:1: Could not append to file '/fake/a.txt': denied");

    fake.add_file("/fake/d.txt", "x");
    fake.fail_file("/fake/d.txt", PlatformError::new(PlatformErrorKind::PermissionDenied, "denied"));
    let error = collect_with_fake(&fake, "import File\nFile.delete(\"/fake/d.txt\")\n").unwrap_err();
    assert_eq!(error.to_string(), "Error at 2:1: Could not delete file '/fake/d.txt': denied");
}

#[test]
fn sorts_environment_and_reports_os_arch() {
    let fake = FakePlatform::new();
    fake.set_env(vec![("Z".to_owned(), "1".to_owned()), ("A".to_owned(), "2".to_owned())]);
    fake.set_os_arch("windows", "x86_64");
    let output = collect_with_fake(
        &fake,
        "import System\nprint(System.env())\nprint(System.os)\nprint(System.arch)\n",
    )
    .unwrap();
    assert_eq!(output, ["{\"A\" = \"2\", \"Z\" = \"1\"}", "windows", "x86_64"]);
}

#[test]
fn pure_natives_still_dispatch() {
    let fake = FakePlatform::new();
    let output = collect_with_fake(
        &fake,
        "import Math\nimport Text\nprint(Math.abs(0 - 3))\nprint(Text.upper(\"hi\"))\n",
    )
    .unwrap();
    assert_eq!(output, ["3", "HI"]);
}

#[test]
fn outcome_carries_file_output() {
    let fake = FakePlatform::new();
    let mut interpreter = Interpreter::new().with_platform(Box::new(fake.clone()));
    match interpreter
        .run_outcome("import File\nFile.write(\"/fake/o.txt\", \"v\")\nprint(File.read(\"/fake/o.txt\"))\n")
        .unwrap()
    {
        RunOutcome::Completed(output) => assert_eq!(output, ["v"]),
        RunOutcome::Exited { code, output } => panic!("expected Completed, got Exited({code}, {output:?})"),
    }
    assert!(fake.output().is_empty());
}
