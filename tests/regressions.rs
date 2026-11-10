//! Regression tests for defects found in the 1.0.3 Beta audit.
//!
//! Each test pins a behavior that previously aborted, panicked, or silently
//! contradicted the language reference. Keep these tests when changing the
//! lexer, parser, interpreter, or decimal implementation.
use luca_code::run;

#[test]
fn indented_line_comments_do_not_break_blocks() {
    assert_eq!(
        run("def func f():\n    print(\"a\")\n        -- deeply indented comment\n    print(\"b\")\nf()\n").unwrap(),
        ["a", "b"]
    );
    // A comment at the outer indentation level must not split the block.
    assert_eq!(
        run("def func f():\n    print(\"a\")\n-- column-zero comment\n    print(\"b\")\nf()\n")
            .unwrap(),
        ["a", "b"]
    );
}

#[test]
fn indented_block_comments_do_not_break_blocks() {
    assert_eq!(
        run("def func f():\n    print(\"a\")\n        [[ block\n        comment ]]\n    print(\"b\")\nf()\n").unwrap(),
        ["a", "b"]
    );
    assert_eq!(
        run("if true then :\n    [[\n    multi-line block comment\n    ]]\n    print(\"ok\")\n")
            .unwrap(),
        ["ok"]
    );
}

#[test]
fn comment_only_block_is_still_an_empty_block() {
    let error = run("if true then :\n    [[ comment ]]\nprint(1)\n").unwrap_err();
    assert!(error.message.contains("Empty block"), "got: {error}");
}

#[test]
fn int_length_handles_i64_min() {
    // `i64::MIN.unsigned_abs()` has 19 digits; `.abs()` used to panic.
    assert_eq!(
        run("def var x = 0 - 9223372036854775807 - 1\nprint(x.length)\nprint(x)\n").unwrap(),
        ["19", "-9223372036854775808"]
    );
}

#[test]
fn uni_bindings_accept_and_mutate_collections() {
    assert_eq!(
        run("def var x (uni) = [1, 2]\nprint(x.length)\nx.add(3)\nprint(x)\n").unwrap(),
        ["2", "[1, 2, 3]"]
    );
    assert_eq!(
        run("def var x (uni) = {\"a\", \"b\"}\nx.remove(\"a\")\nprint(x)\n").unwrap(),
        ["{\"b\"}"]
    );
    assert_eq!(
        run("def var x (uni) = [1, 2]\nchange x[1] = 9\nprint(x)\n").unwrap(),
        ["[9, 2]"]
    );
    assert_eq!(
        run("def var x (uni) = {\"a\" = 1}\nx.add(\"b\", 2)\nprint(x)\n").unwrap(),
        ["{\"a\" = 1, \"b\" = 2}"]
    );
}

#[test]
fn uni_bindings_can_change_between_collection_and_scalar() {
    assert_eq!(
        run("def var x (uni) = [1]\nprint(x)\nx = \"text\"\nprint(x)\nx = 3.50\nprint(x)\n")
            .unwrap(),
        ["[1]", "text", "3.50"]
    );
    assert_eq!(
        run("def var x (uni) = [1]\nchange var x (uni) = {\"k\" = 1}\nprint(x)\n").unwrap(),
        ["{\"k\" = 1}"]
    );
}

#[test]
fn universal_declarations_reject_typed_collection_bindings() {
    let error = run("def var x (int) = [1]\n").unwrap_err();
    assert!(
        error.message.contains("Expected int, found list"),
        "got: {error}"
    );
    let error = run("def const x (str) = {\"a\"}\n").unwrap_err();
    assert!(
        error.message.contains("Expected str, found set"),
        "got: {error}"
    );
}

#[test]
fn const_collection_reads_are_isolated_from_aliases() {
    assert_eq!(
        run("def const items = [1, 2]\ndef var alias = items\nalias.add(3)\nprint(items)\nprint(alias)\n").unwrap(),
        ["[1, 2]", "[1, 2, 3]"]
    );
    assert_eq!(
        run("def const record = {\"n\" = [1]}\ndef var inner = record[\"n\"]\ninner.add(2)\nprint(record)\nprint(inner)\n").unwrap(),
        ["{\"n\" = [1]}", "[1, 2]"]
    );
}
