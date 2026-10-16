//! Non-terminating execution outcomes (M1b-3).
//!
//! These tests execute programs containing valid `System.exit(N)` through
//! the safe `run_outcome` API entirely in-process: the test runner must
//! stay alive, and the API itself must emit nothing (output travels in the
//! returned outcome; warnings travel through the injected platform).
use luca_code::{
    error::LucaError,
    interpreter::{Interpreter, RunOutcome},
    FakePlatform,
};
use std::path::PathBuf;

fn outcome_with_fake(source: &str) -> (FakePlatform, Result<RunOutcome, LucaError>) {
    let fake = FakePlatform::new();
    let mut interpreter = Interpreter::new().with_platform(Box::new(fake.clone()));
    let outcome = interpreter.run_outcome(source);
    (fake, outcome)
}

#[test]
fn exit_returns_code_and_prior_output_without_terminating() {
    let (fake, outcome) = outcome_with_fake(
        "import System\nprint(\"before\")\nSystem.exit(7)\nprint(\"after\")\n",
    );
    match outcome.unwrap() {
        RunOutcome::Exited { code, output } => {
            assert_eq!(code, 7);
            assert_eq!(output, ["before"]);
        }
        RunOutcome::Completed(output) => panic!("expected Exited, got Completed({output:?})"),
    }
    // The API itself emits nothing; the host decides what to do.
    assert!(fake.output().is_empty());
}

#[test]
fn completed_returns_buffer_without_emitting() {
    let (fake, outcome) = outcome_with_fake("print(\"a\")\nprint(\"b\")\n");
    assert_eq!(outcome.unwrap(), RunOutcome::Completed(vec!["a".to_owned(), "b".to_owned()]));
    assert!(fake.output().is_empty());
}

#[test]
fn ordinary_error_returns_err_and_discards_buffer() {
    let (fake, outcome) = outcome_with_fake("print(\"kept\")\nprint(10 / 0)\nprint(\"lost\")\n");
    let error = outcome.unwrap_err();
    assert_eq!(error.to_string(), "Error at 2:10: Division by zero");
    assert!(fake.output().is_empty());
}

#[test]
fn exit_inside_function_carries_code_and_output() {
    let (fake, outcome) = outcome_with_fake(
        "import System\ndef func quit():\n    System.exit(3)\nprint(\"a\")\nquit()\nprint(\"no\")\n",
    );
    match outcome.unwrap() {
        RunOutcome::Exited { code, output } => {
            assert_eq!(code, 3);
            assert_eq!(output, ["a"]);
        }
        RunOutcome::Completed(output) => panic!("expected Exited, got Completed({output:?})"),
    }
    assert!(fake.output().is_empty());
}

#[test]
fn exit_during_module_init_keeps_identity() {
    // Module source comes from the injected fake: module loading now goes
    // through Platform, so scripting here keeps the test hermetic.
    let fake = FakePlatform::new();
    fake.add_file("/fake/lib/Boomer.lucc", "import System\nSystem.exit(5)\n");
    let mut interpreter = Interpreter::with_search_paths(vec![PathBuf::from("/fake/lib")])
        .with_platform(Box::new(fake.clone()));
    let outcome = interpreter.run_outcome("import Boomer\nprint(\"no\")\n");
    match outcome.unwrap() {
        RunOutcome::Exited { code, output } => {
            assert_eq!(code, 5);
            assert!(output.is_empty());
        }
        RunOutcome::Completed(output) => panic!("expected Exited, got Completed({output:?})"),
    }
    assert!(fake.output().is_empty());
}

#[test]
fn exit_in_try_runs_finally_and_skips_capture() {
    let (fake, outcome) = outcome_with_fake(
        "import System\ntry to :\n    print(\"t\")\n    System.exit(9)\ncapture error :\n    print(\"caught\")\nfinally :\n    print(\"fin\")\nprint(\"after\")\n",
    );
    match outcome.unwrap() {
        RunOutcome::Exited { code, output } => {
            assert_eq!(code, 9);
            assert_eq!(output, ["t", "fin"]);
        }
        RunOutcome::Completed(output) => panic!("expected Exited, got Completed({output:?})"),
    }
    assert!(fake.output().is_empty());
}
