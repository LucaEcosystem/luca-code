//! Parser safety limits. The recursive-descent parser must report a normal
//! diagnostic for pathological nesting instead of overflowing the stack.
use luca_code::run;

#[test]
fn deeply_nested_parentheses_are_rejected_without_aborting() {
    let depth = 20_000;
    let source = format!("print({}1{})\n", "(".repeat(depth), ")".repeat(depth));
    let error = run(&source).unwrap_err();
    assert!(
        error.message.contains("nesting is too deep"),
        "got: {error}"
    );
}

#[test]
fn deeply_nested_blocks_are_rejected_without_aborting() {
    let depth = 600;
    let mut source = String::new();
    for level in 0..depth {
        source.push_str(&"    ".repeat(level));
        source.push_str("if true then :\n");
    }
    source.push_str(&"    ".repeat(depth));
    source.push_str("print(1)\n");
    let error = run(&source).unwrap_err();
    assert!(
        error.message.contains("nesting is too deep"),
        "got: {error}"
    );
}

#[test]
fn nesting_below_the_limit_still_parses() {
    let depth = 24;
    let source = format!("print({}1{})\n", "(".repeat(depth), ")".repeat(depth));
    assert_eq!(run(&source).unwrap(), ["1"]);

    let mut blocks = String::new();
    for level in 0..depth {
        blocks.push_str(&"    ".repeat(level));
        blocks.push_str("if true then :\n");
    }
    blocks.push_str(&"    ".repeat(depth));
    blocks.push_str("print(\"ok\")\n");
    assert_eq!(run(&blocks).unwrap(), ["ok"]);
}
