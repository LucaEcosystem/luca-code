//! Interpreter output/warning routing through `Platform` (M1b-2).
//!
//! These tests run real Luca programs against a shared `FakePlatform` and
//! assert captured output ordering plus exact warning line, column, and
//! message. Coordinates below follow the implementation: warnings carry
//! the location of the statement or expression the interpreter is
//! evaluating (e.g. `warn_unreachable_after` reports the *next*
//! statement's location via `statement_location`; index warnings carry the
//! `[` bracket's location; dict-entry warnings carry the duplicate key's
//! location). The clone-moved-into-interpreter pattern throughout also
//! proves `FakePlatform` clones share observations with the original.
use luca_code::{interpreter::Interpreter, FakePlatform};

fn run_with_shared_fake(source: &str) -> (FakePlatform, Result<(), luca_code::error::LucaError>) {
    let fake = FakePlatform::new();
    let mut interpreter = Interpreter::new().with_platform(Box::new(fake.clone()));
    let result = interpreter.run(source);
    (fake, result)
}

fn run_platform_input(source: &str, fake: &FakePlatform) -> Result<(), luca_code::error::LucaError> {
    Interpreter::new()
        .with_platform(Box::new(fake.clone()))
        .run(source)
}

#[test]
fn platform_provides_normal_input_line() {
    let fake = FakePlatform::new();
    fake.set_input(vec!["Luca".to_owned()]);
    run_platform_input(
        "def const name (str) = ask \"Name?\"\nprint(\"hi {name}\")\n",
        &fake,
    )
    .unwrap();
    assert_eq!(fake.output(), ["Name?", "hi Luca"]);
}

#[test]
fn platform_empty_input_reports_eof_diagnostic() {
    let fake = FakePlatform::new();
    let error = run_platform_input("def const name (str) = ask \"Name?\"\nprint(name)\n", &fake)
        .unwrap_err();
    assert_eq!(error.to_string(), "Error at 1:24: No more input available");
    // The prompt sits in the output buffer, which ordinary errors discard.
    assert!(fake.output().is_empty());
}

#[test]
fn platform_input_error_keeps_failure_wrapper() {
    let fake = FakePlatform::new();
    fake.fail_input(luca_code::PlatformError::new(luca_code::PlatformErrorKind::Other, "boom"));
    let error = run_platform_input("def const name (str) = ask \"Name?\"\nprint(name)\n", &fake)
        .unwrap_err();
    assert_eq!(error.to_string(), "Error at 1:24: Failed to read input: boom");
    // Ordinary errors discard buffered output, prompt included.
    assert!(fake.output().is_empty());
}

#[test]
fn platform_permission_denied_input_error_keeps_wrapper() {
    let fake = FakePlatform::new();
    fake.fail_input(luca_code::PlatformError::new(
        luca_code::PlatformErrorKind::PermissionDenied,
        "denied",
    ));
    let error = run_platform_input("def const name (str) = ask \"Name?\"\nprint(name)\n", &fake)
        .unwrap_err();
    assert_eq!(error.to_string(), "Error at 1:24: Failed to read input: denied");
    // Ordinary errors discard buffered output, prompt included.
    assert!(fake.output().is_empty());
}

#[test]
fn platform_prompt_precedes_result_in_order() {
    let fake = FakePlatform::new();
    fake.push_input("16");
    run_platform_input(
        "def const age (int) = ask \"Age?\"\nprint(age + 1)\n",
        &fake,
    )
    .unwrap();
    assert_eq!(fake.output(), ["Age?", "17"]);
}

#[test]
fn preserves_print_output_order() {
    let (fake, result) = run_with_shared_fake("print(\"one\")\nprint(\"two\")\nprint(\"three\")\n");
    result.unwrap();
    assert_eq!(fake.output(), ["one", "two", "three"]);
    assert!(fake.warnings().is_empty());
}

#[test]
fn preserves_ask_prompt_before_result() {
    let fake = FakePlatform::new();
    let mut interpreter = Interpreter::new()
        .with_input(vec!["Luca".to_owned()])
        .with_platform(Box::new(fake.clone()));
    interpreter.run("def const name (str) = ask \"Name?\"\nprint(\"hi {name}\")\n").unwrap();
    assert_eq!(fake.output(), ["Name?", "hi Luca"]);
}

#[test]
fn warns_unreachable_code_after_return_at_next_statement() {
    let (fake, result) = run_with_shared_fake(
        "def func f() (int):\n    return 1\n    print(\"dead\")\nprint(f())\n",
    );
    result.unwrap();
    assert_eq!(fake.output(), ["1"]);
    assert_eq!(fake.warnings(), [(3, 5, "Unreachable code".to_owned())]);
}

#[test]
fn warns_unreachable_code_after_stop_at_next_statement() {
    let (fake, result) = run_with_shared_fake(
        "for 2 times repeat :\n    stop\n    print(\"dead\")\nprint(\"done\")\n",
    );
    result.unwrap();
    assert_eq!(fake.output(), ["done"]);
    assert_eq!(fake.warnings(), [(3, 5, "Unreachable code".to_owned())]);
}

#[test]
fn warns_on_out_of_range_index_and_returns_none() {
    let (fake, result) =
        run_with_shared_fake("def list l (int) = [1]\nprint(l[5])\nprint(\"after\")\n");
    result.unwrap();
    assert_eq!(fake.output(), ["none", "after"]);
    assert_eq!(fake.warnings(), [(2, 8, "Index out of range".to_owned())]);
}

#[test]
fn warns_on_missing_dictionary_key_and_returns_none() {
    let (fake, result) =
        run_with_shared_fake("def dict d = {\"a\" = 1}\nprint(d[\"b\"])\nprint(\"after\")\n");
    result.unwrap();
    assert_eq!(fake.output(), ["none", "after"]);
    assert_eq!(fake.warnings(), [(2, 8, "Key 'b' not found".to_owned())]);
}

#[test]
fn warns_on_duplicate_set_literal_value() {
    let (fake, result) = run_with_shared_fake("def set s = {1, 1}\nprint(\"ok\")\n");
    result.unwrap();
    assert_eq!(fake.output(), ["ok"]);
    assert_eq!(fake.warnings(), [(1, 13, "Duplicate set value".to_owned())]);
}

#[test]
fn warns_on_duplicate_dictionary_key_at_duplicate_entry() {
    let (fake, result) =
        run_with_shared_fake("def dict d = {\"a\" = 1, \"a\" = 2}\nprint(\"ok\")\n");
    result.unwrap();
    assert_eq!(fake.output(), ["ok"]);
    assert_eq!(
        fake.warnings(),
        [(1, 24, "Duplicate dictionary key 'a'".to_owned())]
    );
}

#[test]
fn warns_on_duplicate_set_add() {
    let (fake, result) = run_with_shared_fake("def set s = {1}\ns.add(1)\nprint(\"ok\")\n");
    result.unwrap();
    assert_eq!(fake.output(), ["ok"]);
    assert_eq!(fake.warnings(), [(2, 1, "Set value already exists".to_owned())]);
}

#[test]
fn warns_on_duplicate_dictionary_add() {
    let (fake, result) =
        run_with_shared_fake("def dict d = {\"a\" = 1}\nd.add(\"a\", 2)\nprint(\"ok\")\n");
    result.unwrap();
    assert_eq!(fake.output(), ["ok"]);
    assert_eq!(
        fake.warnings(),
        [(2, 1, "Duplicate dictionary key 'a'".to_owned())]
    );
}

#[test]
fn warns_on_missing_list_removal() {
    let (fake, result) =
        run_with_shared_fake("def list l (int) = [1]\nl.remove(9)\nprint(\"ok\")\n");
    result.unwrap();
    assert_eq!(fake.output(), ["ok"]);
    assert_eq!(fake.warnings(), [(2, 1, "Value not found; no change".to_owned())]);
}

#[test]
fn warns_on_missing_dictionary_removal() {
    let (fake, result) =
        run_with_shared_fake("def dict d = {\"a\" = 1}\nd.remove(\"b\")\nprint(\"ok\")\n");
    result.unwrap();
    assert_eq!(fake.output(), ["ok"]);
    assert_eq!(
        fake.warnings(),
        [(2, 1, "Key 'b' not found; no change".to_owned())]
    );
}

#[test]
fn run_retains_buffer_across_calls() {
    let fake = FakePlatform::new();
    let mut interpreter = Interpreter::new().with_platform(Box::new(fake.clone()));
    interpreter.run("print(\"a\")\n").unwrap();
    interpreter.run("print(\"b\")\n").unwrap();
    assert_eq!(fake.output(), ["a", "a", "b"]);
}

#[test]
fn run_collect_returns_buffer_without_emitting_while_warnings_capture() {
    let fake = FakePlatform::new();
    let output = Interpreter::new()
        .with_platform(Box::new(fake.clone()))
        .run_collect("def set s = {1, 1}\nprint(\"x\")\n")
        .unwrap();
    assert_eq!(output, ["x"]);
    assert!(fake.output().is_empty());
    assert_eq!(fake.warnings(), [(1, 13, "Duplicate set value".to_owned())]);
}
