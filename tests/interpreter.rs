use luca_code::{interpreter::Interpreter, run, run_with_input, PRODUCT_VERSION};
use std::path::PathBuf;

fn run_with_fixtures(source: &str) -> Result<Vec<String>, luca_code::error::LucaError> {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures");
    Interpreter::with_search_paths(vec![fixtures]).run_collect(source)
}

#[test]
fn runs_typed_declarations_arithmetic_and_interpolation() {
    let output = run(
        "def const name (str) = \"Luca\"\ndef var count (int) = 2\ncount (int) = count * 3\nprint(\"{name}: {count}\")\n",
    ).unwrap();
    assert_eq!(output, ["Luca: 6"]);
}

#[test]
fn uses_multiplication_precedence() {
    assert_eq!(run("print(5 + 2 * 3)\n").unwrap(), ["11"]);
}

#[test]
fn rejects_assignment_to_constant() {
    let error = run("def const name (str) = \"Luca\"\nname (str) = \"Other\"\n").unwrap_err();
    assert!(error.message.contains("Cannot change constant"));
}

#[test]
fn rejects_a_mismatched_declared_type() {
    let error = run("def var count (int) = \"two\"\n").unwrap_err();
    assert!(error.message.contains("Expected int, found str"));
}

#[test]
fn identifies_product_release() {
    assert_eq!(PRODUCT_VERSION, "1.0 Beta");
}

#[test]
fn handles_try_capture_finally() {
    // captures a runtime error, finally always runs, execution continues
    assert_eq!(
        run("try to :\n    print(10 / 0)\ncapture error :\n    print(\"caught\")\nfinally :\n    print(\"cleanup\")\nprint(\"after\")\n").unwrap(),
        ["caught", "cleanup", "after"]
    );
    // no error: capture skipped, finally still runs
    assert_eq!(
        run("try to :\n    print(\"try\")\ncapture error :\n    print(\"caught\")\nfinally :\n    print(\"finally\")\n").unwrap(),
        ["try", "finally"]
    );
    // bare try without capture/finally
    assert_eq!(run("try to :\n    print(\"hi\")\nprint(\"after\")\n").unwrap(), ["hi", "after"]);
    // try with only finally (no capture): success path
    assert_eq!(
        run("try to :\n    print(\"a\")\nfinally :\n    print(\"b\")\n").unwrap(),
        ["a", "b"]
    );
    // nested: inner finally runs before outer capture
    assert_eq!(
        run("try to :\n    try to :\n        print(1 / 0)\n    finally :\n        print(\"inner-finally\")\ncapture error :\n    print(\"outer-caught\")\n").unwrap(),
        ["inner-finally", "outer-caught"]
    );
    // try inside loops and functions
    assert_eq!(
        run("for 3 times repeat :\n    try to :\n        print(1 / 0)\n    capture error :\n        print(\"caught\")\nprint(\"done\")\n").unwrap(),
        ["caught", "caught", "caught", "done"]
    );
    assert_eq!(
        run("def func f():\n    try to :\n        return 1 / 0\n    capture error :\n        return -1\nprint(f())\n").unwrap(),
        ["-1"]
    );
    // warnings are not captured
    assert_eq!(
        run("def list l (int) = [1]\ntry to :\n    print(l[5])\n    print(\"after-warn\")\ncapture error :\n    print(\"caught\")\nprint(\"done\")\n").unwrap(),
        ["none", "after-warn", "done"]
    );
    // uncaught error propagates (message preserved)
    let err = run("try to :\n    print(1 / 0)\nfinally :\n    print(\"cleanup\")\n").unwrap_err();
    assert!(err.message.contains("Division by zero"));
    let err = run("print(1 / 0)\n").unwrap_err();
    assert!(err.message.contains("Division by zero"));
}

#[test]
fn handles_try_signals_and_capture_failures() {
    // return inside try runs finally first, then returns
    assert_eq!(
        run("def func foo() (int):\n    try to :\n        return 42\n    finally :\n        print(\"cleanup\")\n    return 0\nprint(foo())\n").unwrap(),
        ["cleanup", "42"]
    );
    // stop inside try runs finally, then exits loop
    assert_eq!(
        run("for 3 times repeat :\n    try to :\n        stop\n    finally :\n        print(\"iter-cleanup\")\nprint(\"done\")\n").unwrap(),
        ["iter-cleanup", "done"]
    );
    // error inside capture block propagates (with its own location)
    let err = run("try to :\n    print(1 / 0)\ncapture error :\n    print(missing)\nfinally :\n    print(\"fin\")\n").unwrap_err();
    assert!(err.message.contains("is not declared"));
    // invalid forms
    let err = run("try missing :\n    print(1)\n").unwrap_err();
    assert!(err.message.contains("Expected 'to'"));
    let err = run("try to :\nprint(1)\n").unwrap_err();
    assert!(err.message.contains("Expected indented block"));
    let err = run("try to :\n    print(1)\ncapture oops :\n    print(2)\n").unwrap_err();
    assert!(err.message.contains("Expected 'error'"));
    let err = run("capture error :\n    print(1)\n").unwrap_err();
    assert!(err.message.contains("Expected a declaration"));
    // stop/skip outside loop are runtime errors, so capture catches them
    assert_eq!(
        run("try to :\n    stop\ncapture error :\n    print(\"caught\")\n").unwrap(),
        ["caught"]
    );
    let err = run("stop\n").unwrap_err();
    assert!(err.message.contains("stop outside loop"));
}

#[test]
fn warns_on_unreachable_code_after_return_or_stop() {
    // Warning (stderr) + normal execution; run() only observes output.
    assert_eq!(
        run("def func f() (int):\n    return 1\n    print(\"dead\")\nprint(f())\n").unwrap(),
        ["1"]
    );
    assert_eq!(
        run("for 3 times repeat :\n    stop\n    print(\"dead\")\nprint(\"done\")\n").unwrap(),
        ["done"]
    );
    // No warning (and no behavior change) when return/stop is last.
    assert_eq!(run("def func f() (int):\n    return 1\nprint(f())\n").unwrap(), ["1"]);
    // Returns nested in branches do not warn about outer code.
    assert_eq!(
        run("def func f(x (int)) (int):\n    if x is == to 1 then :\n        return 10\n    return 20\nprint(f(1))\nprint(f(2))\n").unwrap(),
        ["10", "20"]
    );
}

#[test]
fn rejects_unsupported_operators_and_forms() {
    // No unary `+`, `++`, `--`, `+=`, `-=`; no `let`; keywords are not identifiers.
    let err = run("print(+5)\n").unwrap_err();
    assert!(err.message.contains("Expected an expression"));
    let err = run("def var x = 1\nx++\n").unwrap_err();
    assert!(err.message.contains("Expected '=' in assignment"));
    let err = run("def var x = 1\nx += 1\n").unwrap_err();
    assert!(err.message.contains("Expected '=' in assignment"));
    let err = run("let x = 1\n").unwrap_err();
    assert!(err.message.contains("Expected '=' in assignment"));
    let err = run("def var if = 1\n").unwrap_err();
    assert!(err.message.contains("Expected a declaration name"));
    // print() takes exactly one argument.
    let err = run("print()\n").unwrap_err();
    assert!(err.message.contains("Expected an expression"));
    let err = run("print(1, 2)\n").unwrap_err();
    assert!(err.message.contains("print accepts exactly one argument"));
}

#[test]
fn handles_full_precedence_chain_and_ordering_rules() {
    // not > * / % > + - > comparisons > and > or
    assert_eq!(run("print(not false and true or false)\n").unwrap(), ["true"]);
    assert_eq!(run("print(2 + 3 * 4 is == to 14 and true)\n").unwrap(), ["true"]);
    assert_eq!(run("print(not true is == to false)\n").unwrap(), ["true"]);
    // all six comparison forms
    assert_eq!(run("print(1 is != to 2)\nprint(2 is >= to 2)\nprint(1 is <= to 2)\nprint(1 is < to 2)\n").unwrap(), ["true", "true", "true", "true"]);
    // ordering rejects bools, nones, collections, mismatches
    let err = run("print(true is > to false)\n").unwrap_err();
    assert!(err.message.contains("Cannot order given types"));
    let err = run("print(none is > to 1)\n").unwrap_err();
    assert!(err.message.contains("Cannot order 'none'"));
    let err = run("print(1 is == to \"a\")\n").unwrap_err();
    assert!(err.message.contains("Incompatible types"));
}

#[test]
fn reports_interpolation_locations_and_scales() {
    // Errors point at the string, not 1:1.
    let err = run("def var a = 1\nprint(\"ok\")\nprint(\"x {missing} y\")\n").unwrap_err();
    assert_eq!((err.line, err.column), (3, 7));
    let err = run("print(\"first\")\nprint(\"unclosed {name\")\n").unwrap_err();
    assert_eq!((err.line, err.column), (2, 7));
    assert!(err.message.contains("Unclosed interpolation"));
    // Multiple interpolations and exact decimal rendering.
    assert_eq!(
        run("def var n = \"Luca\"\ndef var v = 2\nprint(\"a {n} b {v}\")\nprint(\"{v} + {v} is {v}\")\n").unwrap(),
        ["a Luca b 2", "2 + 2 is 2"]
    );
    assert_eq!(run("def var p = 20.00\nprint(\"total {p}\")\n").unwrap(), ["total 20.00"]);
}

#[test]
fn shares_collections_through_calls() {
    // Locked: passed collections remain mutable; primitives stay independent.
    assert_eq!(
        run("def list l (int) = [1]\ndef func f(x):\n    x.add(2)\nf(l)\nprint(l)\n").unwrap(),
        ["[1, 2]"]
    );
    assert_eq!(
        run("def list l (int) = [1, 2]\ndef func f(x):\n    change x[1] = 99\nf(l)\nprint(l)\n").unwrap(),
        ["[99, 2]"]
    );
    // Rebinding the parameter stays local.
    assert_eq!(
        run("def list l (int) = [1]\ndef func f(x):\n    x = [9, 9]\nf(l)\nprint(l)\n").unwrap(),
        ["[1]"]
    );
    assert_eq!(
        run("def var n = 5\ndef func f(x):\n    x = x + 1\nf(n)\nprint(n)\n").unwrap(),
        ["5"]
    );
    // var-to-var assignment shares storage (uniform reference values).
    assert_eq!(
        run("def list l (int) = [1]\ndef var m = l\nm.add(2)\nprint(l)\nprint(m)\n").unwrap(),
        ["[1, 2]", "[1, 2]"]
    );
    // const contents can never change, even through an alias or a call.
    assert_eq!(
        run("def const c = [1]\ndef var m = c\nm.add(2)\nprint(m)\nprint(c)\n").unwrap(),
        ["[1, 2]", "[1]"]
    );
    assert_eq!(
        run("def const c = [1]\ndef func f(x):\n    x.add(2)\nf(c)\nprint(c)\n").unwrap(),
        ["[1]"]
    );
    let err = run("def const c = [1]\nc.add(2)\n").unwrap_err();
    assert!(err.message.contains("Cannot change constant"));
}

#[test]
fn handles_cross_feature_integration() {
    // Modules + collections + loops + try/capture + dec conditions + interpolation.
    assert_eq!(
        run_with_fixtures("import Utils\ndef var total = 0\nfor item from Utils.items repeat :\n    try to :\n        def var greeting = Utils.helper(item)\n        total = total + greeting.length\n    capture error :\n        print(\"bad {item}\")\nprint(total)\nprint(\"done {total}\")\n").unwrap(),
        ["8", "done 8"]
    );
    // Functions returning collections + indexing + length in conditions.
    assert_eq!(
        run("def func first_two() (uni):\n    return [10, 20, 30]\ndef var got = first_two()\nif got.length is == to 3 and got[1] is == to 10 then :\n    print(\"ok {got}\")\nelse :\n    print(\"bad\")\n").unwrap(),
        ["ok [10, 20, 30]"]
    );
    // Exact decimals through loops, conditions, and nested collection equality.
    assert_eq!(
        run("def var x = 0.00\nwhile x is < to 1.00 repeat :\n    x = x + 0.25\nprint(x)\nprint([1.0] is == to [1.00])\nprint(1.00 / 4)\n").unwrap(),
        ["1.00", "true", "0.25"]
    );
    // change + forget + redeclare across scopes with collections.
    assert_eq!(
        run("def list l (int) = [1]\nchange l[1] = 2\nif l[1] is == to 2 then :\n    forget var l (int)\n    def list l (str) = [\"back\"]\n    print(l)\n").unwrap(),
        ["[\"back\"]"]
    );
    // Errors deep inside nested constructs keep inner locations.
    let err = run("def var x = 1\nif x is == to 1 then :\n    def var y = \"a\"\n    print(y + 1)\n").unwrap_err();
    assert_eq!((err.line, err.column), (4, 13));
    let err = run("def func f():\n    def var s = \"a\"\n    print(s + 1)\nprint(f())\n").unwrap_err();
    assert_eq!((err.line, err.column), (3, 13));
}

#[test]
fn handles_custom_error_definitions() {
    // fields via bare assignment, including code = none
    assert_eq!(
        run("def error NotFound :\n    code = 404\n    message = \"gone\"\nprint(\"defined\")\n").unwrap(),
        ["defined"]
    );
    assert_eq!(
        run("def error Weird :\n    code = none\n    message = \"odd\"\nprint(\"ok\")\n").unwrap(),
        ["ok"]
    );
    // duplicate and conflicting names are errors
    let err = run("def error E :\n    code = 1\n    message = \"a\"\ndef error E :\n    code = 2\n    message = \"b\"\n").unwrap_err();
    assert!(err.message.contains("already declared"));
    let err = run("def var E = 1\ndef error E :\n    code = 1\n    message = \"m\"\n").unwrap_err();
    assert!(err.message.contains("already declared"));
    // fields use bare assignment (def var code would collide with implicit field)
    let err = run("def error E :\n    def var code = 1\n    message = \"m\"\n").unwrap_err();
    assert!(err.message.contains("already declared"));
    // body errors propagate out of the definition
    let err = run("def error E :\n    code = 1 / 0\n").unwrap_err();
    assert!(err.message.contains("Division by zero"));
    // module error members are definitions, not values or functions
    let err = run_with_fixtures("import ErrMod\nprint(ErrMod.NotFound)\n").unwrap_err();
    assert!(err.message.contains("is an error definition, not a value"));
    let err = run_with_fixtures("import ErrMod\nErrMod.NotFound()\n").unwrap_err();
    assert!(err.message.contains("is an error definition, not a function"));
    // module init errors are capturable
    assert_eq!(
        run_with_fixtures("try to :\n    import Bad\ncapture error :\n    print(\"caught init\")\nfinally :\n    print(\"fin\")\n").unwrap(),
        ["caught init", "fin"]
    );
    // missing modules are capturable too
    assert_eq!(
        run_with_fixtures("try to :\n    import Nope\ncapture error :\n    print(\"caught missing\")\n").unwrap(),
        ["caught missing"]
    );
}

#[test]
fn division_is_numeric_and_returns_dec() {
    assert_eq!(run("print(10 / 4)\n").unwrap(), ["2.5"]);
    assert_eq!(run("print(6 / 2)\n").unwrap(), ["3.0"]);
    assert_eq!(run("print(10 / 2)\n").unwrap(), ["5.0"]);
}

#[test]
fn dec_retains_scale_through_arithmetic() {
    assert_eq!(run("print(20.00 + 1.50)\n").unwrap(), ["21.50"]);
    assert_eq!(run("print(2.50 * 2.00)\n").unwrap(), ["5.00"]);
    assert_eq!(run("print(0.1 + 0.2)\n").unwrap(), ["0.3"]);
    assert_eq!(run("print(0.1 * 0.2)\n").unwrap(), ["0.02"]);
}

#[test]
fn dec_division_and_modulo_mix() {
    assert_eq!(run("print(7.0 / 2)\n").unwrap(), ["3.5"]);
    assert_eq!(run("print(5.5 % 2)\n").unwrap(), ["1.5"]);
    assert_eq!(run("print(5 % 2.0)\n").unwrap(), ["1.0"]);
}

#[test]
fn infers_types_and_handles_none() {
    assert_eq!(run("def var name = \"Luca\"\nprint(name)\n").unwrap(), ["Luca"]);
    assert_eq!(run("def var x (int) = none\nprint(x)\n").unwrap(), ["none"]);
    let err = run("def var y = none\n").unwrap_err();
    assert!(err.message.contains("Cannot infer type from 'none'"));
}

#[test]
fn handles_clear_forget_and_change() {
    assert_eq!(run("def var a (int) = 5\na.clear()\nprint(a)\na (int) = 10\nprint(a)\n").unwrap(), ["none", "10"]);
    assert_eq!(run("def var a (int) = 1\nforget var a (int)\ndef var a (str) = \"hi\"\nprint(a)\n").unwrap(), ["hi"]);
    assert_eq!(run("def var b (int) = 5\nchange b (str) = \"hi\"\nprint(b)\n").unwrap(), ["hi"]);
}

#[test]
fn handles_comparisons_and_logic() {
    assert_eq!(run("print(5 is == to 5)\n").unwrap(), ["true"]);
    assert_eq!(run("print(5 is == to 5.0)\n").unwrap(), ["true"]);
    assert_eq!(run("print(5 is > to 3)\n").unwrap(), ["true"]);
    assert_eq!(run("print(\"b\" is > to \"a\")\n").unwrap(), ["true"]);
    assert_eq!(run("print(not false)\n").unwrap(), ["true"]);
    assert_eq!(run("print(true and false)\n").unwrap(), ["false"]);
    assert_eq!(run("print(true or false)\n").unwrap(), ["true"]);
    assert_eq!(run("print(5 + 2 is == to 7)\n").unwrap(), ["true"]);
    assert_eq!(run("def var x (int) = none\nprint(x is == to none)\n").unwrap(), ["true"]);
}

#[test]
fn handles_multiline_comments() {
    assert_eq!(run("[[ comment ]]\nprint(1)\n").unwrap(), ["1"]);
    assert_eq!(run("print(1) [[ inline ]] \nprint(2)\n").unwrap(), ["1", "2"]);
    assert_eq!(run("[[ multi\nline ]] print(3)\n").unwrap(), ["3"]);
}

#[test]
fn handles_function_type_and_invalid_calls() {
    // type error: wrong param type
    let err = run("def func inc(x (int)) (int):\n    return x + 1\nprint(inc(\"hi\"))\n").unwrap_err();
    assert!(err.message.contains("Expected int"));
    // missing argument
    let err = run("def func greet(name (str)) (str):\n    return name\nprint(greet())\n").unwrap_err();
    assert!(err.message.contains("Missing argument"));
    // too many arguments
    let err = run("def func foo():\n    print(1)\nfoo(1)\n").unwrap_err();
    assert!(err.message.contains("Expected 0 arguments"));
    // return type mismatch
    let err = run("def func bar() (int):\n    return \"not int\"\nprint(bar())\n").unwrap_err();
    assert!(err.message.contains("Expected int"));
    // return outside function
    let err = run("return 5\n").unwrap_err();
    assert!(err.message.contains("return outside function"));
    // wrong arg count for no-param func with args
    let err = run("def func foo(a (int), b (int)) (int):\n    return a + b\nprint(foo(1))\n").unwrap_err();
    assert!(err.message.contains("Missing argument"));
    // duplicate param
    let err = run("def func foo(a (int), a (str)):\n    print(a)\n").unwrap_err();
    assert!(err.message.contains("Duplicate parameter"));
    // duplicate func name in same scope
    let err = run("def func foo():\n    print(1)\ndef func foo():\n    print(2)\n").unwrap_err();
    assert!(err.message.contains("already declared"));
    // call unknown function
    let err = run("print(unknown())\n").unwrap_err();
    assert!(err.message.contains("is not declared"));
    // recursion limit
    let err = run("def func inf():\n    return inf()\nprint(inf())\n").unwrap_err();
    assert!(err.message.contains("Recursion limit exceeded"));
}

#[test]
fn handles_conditional_execution_and_blocks() {
    // valid: simple if
    assert_eq!(run("if true then :\n    print(1)\n").unwrap(), ["1"]);
    assert_eq!(run("if false then :\n    print(1)\nprint(2)\n").unwrap(), ["2"]);
    // valid: if else
    assert_eq!(run("if true then :\n    print(1)\nelse :\n    print(2)\n").unwrap(), ["1"]);
    assert_eq!(run("if false then :\n    print(1)\nelse :\n    print(2)\n").unwrap(), ["2"]);
    // valid: else if chain
    assert_eq!(
        run("def var x = 2\nif x is == to 1 then :\n    print(1)\nelse if x is == to 2 then :\n    print(2)\nelse :\n    print(3)\n").unwrap(),
        ["2"]
    );
    // valid: nested
    assert_eq!(
        run("if true then :\n    if true then :\n        print(1)\n    print(2)\nprint(3)\n").unwrap(),
        ["1", "2", "3"]
    );
    // boundary: condition must be bool
    let err = run("if 5 then :\n    print(1)\n").unwrap_err();
    assert!(err.message.contains("Condition must be bool"));
    let err = run("if none then :\n    print(1)\n").unwrap_err();
    assert!(err.message.contains("Condition must be bool"));
    // invalid: empty block
    let err = run("if true then :\nprint(1)\n").unwrap_err();
    assert!(err.message.contains("Expected indented block"));
    let err = run("if true then :\n    [[ comment ]]\nprint(1)\n").unwrap_err();
    assert!(err.message.contains("Empty block"));
    // invalid: missing then/colon
    let err = run("if true :\n    print(1)\n").unwrap_err();
    assert!(err.message.contains("Expected 'then'"));
    let err = run("if true then print(1)\n").unwrap_err();
    assert!(err.message.contains("Expected ':'"));
    // invalid: inconsistent indentation
    let err = run("if true then :\n  print(1)\n    print(2)\n").unwrap_err();
    assert!(err.message.contains("Inconsistent") || err.message.contains("Unexpected") || err.message.contains("Expected"));
    // valid: shadowing and mutation inside block
    assert_eq!(
        run("def var x = \"outer\"\nif true then :\n    def var x = \"inner\"\n    print(x)\nprint(x)\n").unwrap(),
        ["inner", "outer"]
    );
    assert_eq!(
        run("def var x = 5\nif true then :\n    x = 10\nprint(x)\n").unwrap(),
        ["10"]
    );
}

#[test]
fn handles_loops_iteration_termination_nested_and_boundary() {
    // iteration: for N times
    assert_eq!(run("def var s = 0\nfor 5 times repeat :\n    s = s + 1\nprint(s)\n").unwrap(), ["5"]);
    assert_eq!(run("def var s = 0\nfor 0 times repeat :\n    s = s + 1\nprint(s)\n").unwrap(), ["0"]);
    // iteration: while
    assert_eq!(
        run("def var i = 0\ndef var s = 0\nwhile i is < to 5 repeat :\n    s = s + i\n    i = i + 1\nprint(s)\n").unwrap(),
        ["10"]
    );
    // termination: stop and skip
    assert_eq!(
        run("def var x = 0\nfor 5 times repeat :\n    x = x + 1\n    if x is == to 3 then :\n        stop\nprint(x)\n").unwrap(),
        ["3"]
    );
    assert_eq!(
        run("def var s = 0\ndef var y = 0\nfor 5 times repeat :\n    y = y + 1\n    if y is == to 2 then :\n        skip\n    s = s + y\nprint(s)\n").unwrap(),
        ["13"]
    );
    // while with stop
    assert_eq!(
        run("def var n = 5\nwhile n is > to 0 repeat :\n    print(n)\n    n = n - 1\n    if n is == to 2 then :\n        stop\nprint(\"done\")\n").unwrap(),
        ["5", "4", "3", "done"]
    );
    // nested loops
    assert_eq!(
        run("for 2 times repeat :\n    for 3 times repeat :\n        print(\"x\")\n").unwrap(),
        ["x", "x", "x", "x", "x", "x"]
    );
    // nested: stop only exits inner
    assert_eq!(
        run("for 2 times repeat :\n    print(\"o\")\n    for 2 times repeat :\n        print(\"i\")\n        stop\n    print(\"a\")\n").unwrap(),
        ["o", "i", "a", "o", "i", "a"]
    );
    // boundary: condition must be bool
    let err = run("while 5 repeat :\n    print(1)\n").unwrap_err();
    assert!(err.message.contains("Condition must be bool"));
    let err = run("while none repeat :\n    print(1)\n").unwrap_err();
    assert!(err.message.contains("Condition must be bool"));
    // invalid: for count must be int
    let err = run("for \"hi\" times repeat :\n    print(1)\n").unwrap_err();
    assert!(err.message.contains("for count must be int"));
    let err = run("for -5 times repeat :\n    print(1)\n").unwrap_err();
    assert!(err.message.contains("for count must be non-negative"));
    // invalid: empty block
    let err = run("for 2 times repeat :\nprint(1)\n").unwrap_err();
    assert!(err.message.contains("Expected indented block"));
    // invalid: missing times/repeat/colon
    let err = run("for 2 repeat :\n    print(1)\n").unwrap_err();
    assert!(err.message.contains("Expected 'times'"));
    let err = run("for 2 times :\n    print(1)\n").unwrap_err();
    assert!(err.message.contains("Expected 'repeat'"));
    let err = run("while true :\n    print(1)\n").unwrap_err();
    assert!(err.message.contains("Expected 'repeat'"));
    // invalid: stop/skip outside loop
    let err = run("stop\n").unwrap_err();
    assert!(err.message.contains("stop outside loop"));
    let err = run("skip\n").unwrap_err();
    assert!(err.message.contains("skip outside loop"));
    let err = run("if true then :\n    stop\n").unwrap_err();
    assert!(err.message.contains("stop outside loop"));
    // valid: stop/skip inside if inside loop
    assert_eq!(
        run("for 3 times repeat :\n    if true then :\n        skip\n    print(\"hi\")\nprint(\"done\")\n").unwrap(),
        ["done"]
    );
    // return inside loop exits function
    assert_eq!(
        run("def func foo():\n    for 5 times repeat :\n        if true then :\n            return 42\n    return 0\nprint(foo())\n").unwrap(),
        ["42"]
    );
    // deferred: for ... from collection
    let err = run("for x from c repeat :\n    print(x)\n").unwrap_err();
    assert!(err.message.contains("not implemented") || err.message.contains("is not declared"));
}

#[test]
fn handles_collections_construction_access_and_equality() {
    // construction: list, tuple, set, dict
    assert_eq!(run("def list names (str) = [\"Luca\", \"Alex\"]\nprint(names)\n").unwrap(), ["[\"Luca\", \"Alex\"]"]);
    assert_eq!(run("def tuple data (uni) = (123, true, \"Luca\")\nprint(data)\n").unwrap(), ["(123, true, \"Luca\")"]);
    assert_eq!(run("def set labels (str) = {\"alpha\", \"stable\"}\nprint(labels)\n").unwrap(), ["{\"alpha\", \"stable\"}"]);
    assert_eq!(
        run("def dict person = {\"Name\" = \"Luca\" (str), \"Age\" = 16 (int)}\nprint(person)\n").unwrap(),
        ["{\"Name\" = \"Luca\", \"Age\" = 16}"]
    );
    // empty collections
    assert_eq!(run("def list e (int) = []\nprint(e)\nprint(e.length)\n").unwrap(), ["[]", "0"]);
    assert_eq!(run("def tuple e (int) = ()\nprint(e)\n").unwrap(), ["()"]);
    assert_eq!(run("def dict d = {}\nprint(d)\n").unwrap(), ["{}"]);
    // 1-based access
    assert_eq!(run("def list l (int) = [10, 20]\nprint(l[1])\nprint(l[2])\n").unwrap(), ["10", "20"]);
    assert_eq!(run("def dict d = {\"a\" = 1 (int)}\nprint(d[\"a\"])\n").unwrap(), ["1"]);
    // missing index/key warns and yields none (no error)
    assert_eq!(run("def list l (int) = [1]\nprint(l[5])\n").unwrap(), ["none"]);
    assert_eq!(run("def dict d = {\"a\" = 1 (int)}\nprint(d[\"missing\"])\n").unwrap(), ["none"]);
    // content equality (recursive, set unordered)
    assert_eq!(run("def list a (int) = [1, 2]\ndef list b (int) = [1, 2]\nprint(a is == to b)\n").unwrap(), ["true"]);
    assert_eq!(run("def set a (int) = {1, 2}\ndef set b (int) = {2, 1}\nprint(a is == to b)\n").unwrap(), ["true"]);
    assert_eq!(
        run("def dict a = {\"x\" = 1 (int)}\ndef dict b = {\"x\" = 1 (int)}\nprint(a is == to b)\n").unwrap(),
        ["true"]
    );
    assert_eq!(
        run("def list a (int) = [1, 2]\nprint(a is != to [1, 3])\n").unwrap(),
        ["true"]
    );
    // incompatible comparison is error
    let err = run("print([1] is == to \"hi\")\n").unwrap_err();
    assert!(err.message.contains("Incompatible types"));
    let err = run("print([1] is > to [2])\n").unwrap_err();
    assert!(err.message.contains("Cannot order"));
    // hetero with uni
    assert_eq!(run("def list h (uni) = [1, \"a\", true]\nprint(h.length)\n").unwrap(), ["3"]);
    let err = run("def list h (int) = [1, \"a\"]\n").unwrap_err();
    assert!(err.message.contains("Expected int"));
}

#[test]
fn handles_collections_mutation_length_and_errors() {
    // add/remove/clear
    assert_eq!(
        run("def list l (str) = [\"a\"]\nl.add(\"b\")\nprint(l)\nl.remove(\"a\")\nprint(l)\nl.clear()\nprint(l)\n").unwrap(),
        ["[\"a\", \"b\"]", "[\"b\"]", "[]"]
    );
    assert_eq!(
        run("def dict d = {\"a\" = 1 (int)}\nd.add(\"b\", 2)\nprint(d)\nd.remove(\"a\")\nprint(d)\n").unwrap(),
        ["{\"a\" = 1, \"b\" = 2}", "{\"b\" = 2}"]
    );
    // change preserves type
    assert_eq!(
        run("def list l (int) = [1, 2]\nchange l[1] = 99\nprint(l)\n").unwrap(),
        ["[99, 2]"]
    );
    assert_eq!(
        run("def dict d = {\"a\" = 1 (int)}\nchange d[\"a\"] = 2\nprint(d)\n").unwrap(),
        ["{\"a\" = 2}"]
    );
    // contains and length
    assert_eq!(run("def list l (int) = [1, 2]\nprint(l.contains(2))\nprint(l.contains(9))\n").unwrap(), ["true", "false"]);
    assert_eq!(
        run("def dict d = {\"a\" = 1 (int)}\nprint(d.contains(key = \"a\"))\nprint(d.contains(value = 1))\nprint(d.contains(value = 9))\n").unwrap(),
        ["true", "true", "false"]
    );
    assert_eq!(run("def var s = \"hello\"\nprint(s.length)\n").unwrap(), ["5"]);
    assert_eq!(run("def var n = 12345\nprint(n.length)\n").unwrap(), ["5"]);
    assert_eq!(run("def var d = 20.00\nprint(d.length(decimals))\n").unwrap(), ["2"]);
    assert_eq!(run("def list l (int) = [1, 2]\nprint(l.length(items))\n").unwrap(), ["2"]);
    // invalid: wrong element type on add/change
    let err = run("def list l (str) = [\"a\"]\nl.add(5)\n").unwrap_err();
    assert!(err.message.contains("Expected str"));
    let err = run("def list l (int) = [1]\nchange l[1] = \"hi\"\n").unwrap_err();
    assert!(err.message.contains("Expected int"));
    let err = run("def dict d = {\"a\" = 1 (int)}\nchange d[\"a\"] = \"hi\"\n").unwrap_err();
    assert!(err.message.contains("Expected int"));
    // invalid: change missing key/index
    let err = run("def dict d = {\"a\" = 1 (int)}\nchange d[\"missing\"] = 2\n").unwrap_err();
    assert!(err.message.contains("does not exist"));
    let err = run("def list l (int) = [1]\nchange l[5] = 2\n").unwrap_err();
    assert!(err.message.contains("Index out of range"));
    // invalid: index set, index scalar, length bool
    let err = run("def set s (int) = {1}\nprint(s[1])\n").unwrap_err();
    assert!(err.message.contains("Cannot index set"));
    let err = run("print(true.length)\n").unwrap_err();
    assert!(err.message.contains("length not supported for bool"));
    let err = run("def dict d = {\"a\" = 1 (int)}\nprint(d.contains(\"a\"))\n").unwrap_err();
    assert!(err.message.contains("contains(key"));
    // const cannot be mutated
    let err = run("def const x = [1]\nx.add(2)\n").unwrap_err();
    assert!(err.message.contains("Cannot change constant"));
    // mutating iterated collection is error
    let err = run("def list l (int) = [1, 2]\nfor x from l repeat :\n    l.add(3)\n").unwrap_err();
    assert!(err.message.contains("being iterated"));
    // for-from iterates values and dict keys
    assert_eq!(
        run("def list l (str) = [\"a\", \"b\"]\nfor item from l repeat :\n    print(item)\n").unwrap(),
        ["a", "b"]
    );
    assert_eq!(
        run("def dict d = {\"a\" = 1 (int), \"b\" = 2 (int)}\nfor k from d repeat :\n    print(k)\n").unwrap(),
        ["a", "b"]
    );
}

#[test]
fn handles_functions_normal_nested_and_edge_cases() {
    // normal: simple func with default
    assert_eq!(
        run("def func greet(name (str) = \"Luca\") (str):\n    return \"Hello, {name}!\"\nprint(greet())\nprint(greet(\"Alex\"))\n").unwrap(),
        ["Hello, Luca!", "Hello, Alex!"]
    );
    // normal: typed params and return
    assert_eq!(
        run("def func add(a (int), b (int)) (int):\n    return a + b\nprint(add(2, 3))\n").unwrap(),
        ["5"]
    );
    // nested calls
    assert_eq!(
        run("def func add(a (int), b (int)) (int):\n    return a + b\ndef func double(x (int)) (int):\n    return add(x, x)\nprint(double(5))\nprint(add(double(2), 3))\n").unwrap(),
        ["10", "7"]
    );
    // recursion
    assert_eq!(
        run("def func fact(n (int)) (int):\n    if n is <= to 1 then :\n        return 1\n    else :\n        return n * fact(n - 1)\nprint(fact(5))\n").unwrap(),
        ["120"]
    );
    // closure: read and modify captured var
    assert_eq!(
        run("def var x = 5\ndef func inc():\n    x = x + 1\ninc()\nprint(x)\n").unwrap(),
        ["6"]
    );
    // nested function and shadowing
    assert_eq!(
        run("def var x = \"outer\"\ndef func test():\n    def var x = \"inner\"\n    print(x)\ntest()\nprint(x)\n").unwrap(),
        ["inner", "outer"]
    );
    // bare return and no return (returns none)
    assert_eq!(
        run("def func foo():\n    return\nprint(foo())\n").unwrap(),
        ["none"]
    );
    assert_eq!(
        run("def func bar():\n    print(\"hi\")\nprint(bar())\n").unwrap(),
        ["hi", "none"]
    );
    // untyped params accept any
    assert_eq!(
        run("def func echo(x):\n    return x\nprint(echo(5))\nprint(echo(\"hi\"))\nprint(echo(true))\n").unwrap(),
        ["5", "hi", "true"]
    );
}

#[test]
fn handles_modules_import_call_and_members() {
    assert_eq!(
        run_with_fixtures("import Utils\nprint(Utils.helper())\nprint(Utils.helper(\"Luca\"))\nprint(Utils.version)\nprint(Utils.items[1])\n").unwrap(),
        ["hi World", "hi Luca", "1.0", "a"]
    );
    // member collection queries through namespace
    assert_eq!(
        run_with_fixtures("import Utils\nprint(Utils.items.contains(\"a\"))\nprint(Utils.items.length)\nfor x from Utils.items repeat :\n    print(x)\n").unwrap(),
        ["true", "2", "a", "b"]
    );
    // transitive imports
    assert_eq!(run_with_fixtures("import Mid\nprint(Mid.mid_fn())\n").unwrap(), ["16"]);
    // mutable module state persists across calls
    assert_eq!(
        run_with_fixtures("import Counter\nprint(Counter.bump())\nprint(Counter.bump())\n").unwrap(),
        ["1", "2"]
    );
}

#[test]
fn handles_modules_once_missing_cycle_init_and_visibility() {
    // once-only initialization: second import reuses
    assert_eq!(
        run_with_fixtures("import Once\nimport Once\nprint(Once.get())\n").unwrap(),
        ["once init", "41"]
    );
    // missing module
    let err = run_with_fixtures("import Nope\n").unwrap_err();
    assert!(err.message.contains("missing module"));
    // circular imports
    let err = run_with_fixtures("import A\n").unwrap_err();
    assert!(err.message.contains("Circular import"));
    let err = run_with_fixtures("import Selfish\n").unwrap_err();
    assert!(err.message.contains("Circular import"));
    // initialization errors propagate
    let err = run_with_fixtures("import Bad\nprint(\"unreached\")\n").unwrap_err();
    assert!(err.message.contains("already declared"));
    // module scope isolation: cannot see importer bindings
    let err = run_with_fixtures("def var secret = 1\nimport Needy\nprint(Needy.get())\n").unwrap_err();
    assert!(err.message.contains("is not declared"));
}

#[test]
fn handles_modules_invalid_calls_and_lookups() {
    // unknown member
    let err = run_with_fixtures("import Utils\nprint(Utils.nope)\n").unwrap_err();
    assert!(err.message.contains("has no member"));
    let err = run_with_fixtures("import Utils\nUtils.nope()\n").unwrap_err();
    assert!(err.message.contains("has no member"));
    // function used as value / data used as function
    let err = run_with_fixtures("import Utils\nprint(Utils.helper)\n").unwrap_err();
    assert!(err.message.contains("is a function"));
    let err = run_with_fixtures("import Utils\nprint(Utils.version())\n").unwrap_err();
    assert!(err.message.contains("is not a function"));
    // wrong argument count / missing argument
    let err = run_with_fixtures("import Utils\nprint(Utils.helper(\"a\", \"b\"))\n").unwrap_err();
    assert!(err.message.contains("Expected 1 arguments"));
    // not imported
    let err = run_with_fixtures("print(Utils.helper())\n").unwrap_err();
    assert!(err.message.contains("is not imported"));
    // name conflict with module
    let err = run_with_fixtures("import Utils\ndef var Utils = 5\n").unwrap_err();
    assert!(err.message.contains("already declared"));
}

fn run_with_cli_args(source: &str, args: Vec<&str>) -> Result<Vec<String>, luca_code::error::LucaError> {
    Interpreter::new()
        .with_cli_args(args.into_iter().map(|a| a.to_owned()).collect())
        .run_collect(source)
}

fn temp_file(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("luca_09_{name}"))
}

#[test]
fn handles_ask_conversions_and_errors() {
    // str/int/dec/bool conversion per the binding's declared type.
    assert_eq!(
        run_with_input("def const name (str) = ask \"Name?\"\nprint(\"hi {name}\")\n", vec!["Luca".to_owned()]).unwrap(),
        ["Name?", "hi Luca"]
    );
    assert_eq!(
        run_with_input("def const age (int) = ask \"Age?\"\nprint(age + 1)\n", vec!["16".to_owned()]).unwrap(),
        ["Age?", "17"]
    );
    assert_eq!(
        run_with_input("def const price (dec) = ask \"Price?\"\nprint(price * 2)\n", vec!["2.50".to_owned()]).unwrap(),
        ["Price?", "5.00"]
    );
    assert_eq!(
        run_with_input("def const flag (bool) = ask \"Flag?\"\nprint(flag and true)\n", vec!["true".to_owned()]).unwrap(),
        ["Flag?", "true"]
    );
    // Negative and integer-looking decimals convert.
    assert_eq!(
        run_with_input("def const d (dec) = ask \"D?\"\nprint(d)\n", vec!["-3".to_owned()]).unwrap(),
        ["D?", "-3.0"]
    );
    // Inferred bindings read strings.
    assert_eq!(
        run_with_input("def var answer = ask \"Q?\"\nprint(answer)\n", vec!["yes".to_owned()]).unwrap(),
        ["Q?", "yes"]
    );
    // Assignments and function parameters convert the same way.
    assert_eq!(
        run_with_input(
            "def var age (int) = 0\nage = ask \"Age?\"\nprint(age)\ndef func older(a (int)) (int):\n    return a + 1\nprint(older(ask \"Years?\"))\n",
            vec!["16".to_owned(), "41".to_owned()]
        ).unwrap(),
        ["Age?", "16", "Years?", "42"]
    );
    // Invalid input is an Error with the ask location, and it is capturable.
    let err = run_with_input("def const age (int) = ask \"Age?\"\n", vec!["abc".to_owned()]).unwrap_err();
    assert!(err.message.contains("Invalid input"));
    assert!(err.message.contains("expected int"));
    assert_eq!((err.line, err.column), (1, 23));
    let err = run_with_input("def const d (dec) = ask \"D?\"\n", vec!["1.2.3".to_owned()]).unwrap_err();
    assert!(err.message.contains("expected dec"));
    let err = run_with_input("def const b (bool) = ask \"B?\"\n", vec!["yes".to_owned()]).unwrap_err();
    assert!(err.message.contains("expected bool"));
    assert_eq!(
        run_with_input(
            "try to :\n    def var age (int) = ask \"Age?\"\n    print(age)\ncapture error :\n    print(\"bad input\")\n",
            vec!["abc".to_owned()]
        ).unwrap(),
        ["Age?", "bad input"]
    );
    // Empty input and non-string prompts are Errors.
    let err = run_with_input("def var a = ask \"Q?\"\n", vec![]).unwrap_err();
    assert!(err.message.contains("No more input"));
    let err = run_with_input("print(ask 5)\n", vec!["x".to_owned()]).unwrap_err();
    assert!(err.message.contains("Expected str"));
}

#[test]
fn handles_math_module() {
    assert_eq!(run("import Math\nprint(Math.sqrt(25))\nprint(Math.sqrt(2))\n").unwrap(), ["5.0", "1.4142135623"]);
    assert_eq!(run("import Math\nprint(Math.pow(2, 10))\nprint(Math.pow(2.5, 2))\nprint(Math.pow(2, 0 - 2))\n").unwrap(), ["1024.0", "6.25", "0.25"]);
    assert_eq!(run("import Math\nprint(Math.abs(0 - 5))\nprint(Math.abs(0 - 2.5))\n").unwrap(), ["5", "2.5"]);
    assert_eq!(
        run("import Math\nprint(Math.round(2.5))\nprint(Math.round(2.4))\nprint(Math.round(0 - 2.5))\nprint(Math.floor(2.9))\nprint(Math.floor(0 - 2.1))\nprint(Math.ceil(2.1))\nprint(Math.ceil(0 - 2.9))\n").unwrap(),
        ["3", "2", "-3", "2", "-3", "3", "-2"]
    );
    assert_eq!(run("import Math\nprint(Math.min(3, 1, 2))\nprint(Math.max(1.5, 2))\nprint(Math.max(1.5, 2.5))\n").unwrap(), ["1", "2", "2.5"]);
    // Truncated irrational roots compound exact-decimal error deterministically.
    assert_eq!(run("import Math\nprint(Math.sqrt(2) * Math.sqrt(2))\n").unwrap(), ["1.99999999979325598129"]);
    // Errors: domain, arity, strict types, none rejection.
    let err = run("import Math\nprint(Math.sqrt(0 - 1))\n").unwrap_err();
    assert!(err.message.contains("negative"));
    let err = run("import Math\nprint(Math.pow(2, 2.5))\n").unwrap_err();
    assert!(err.message.contains("exponent must be int"));
    let err = run("import Math\nprint(Math.min(5))\n").unwrap_err();
    assert!(err.message.contains("at least 2 arguments"));
    let err = run("import Math\nprint(Math.sqrt(\"25\"))\n").unwrap_err();
    assert!(err.message.contains("Expected int or dec"));
    let err = run("import Math\nprint(Math.abs(none))\n").unwrap_err();
    assert!(err.message.contains("Cannot use 'none'"));
    let err = run("import Math\nprint(Math.unknown(1))\n").unwrap_err();
    assert!(err.message.contains("has no member"));
    let err = run("import Math\nprint(Math.sqrt)\n").unwrap_err();
    assert!(err.message.contains("is a function"));
}

#[test]
fn handles_text_module() {
    assert_eq!(
        run("import Text\nprint(Text.upper(\"hello\"))\nprint(Text.lower(\"HeLLo\"))\nprint(Text.trim(\"  hi  \"))\nprint(Text.replace(\"aaa\", \"a\", \"b\"))\n").unwrap(),
        ["HELLO", "hello", "hi", "bbb"]
    );
    assert_eq!(
        run("import Text\ndef list parts (str) = Text.split(\"a,b,c\", \",\")\nprint(parts)\nprint(Text.join(parts, \"-\"))\n").unwrap(),
        ["[\"a\", \"b\", \"c\"]", "a-b-c"]
    );
    assert_eq!(run("import Text\nprint(Text.join([], \",\"))\n").unwrap(), [""]);
    // Errors: arity, strict types, none rejection, heterogeneous join.
    let err = run("import Text\nprint(Text.upper(1))\n").unwrap_err();
    assert!(err.message.contains("Expected str"));
    let err = run("import Text\nprint(Text.split(\"a\"))\n").unwrap_err();
    assert!(err.message.contains("Expected 2 arguments"));
    let err = run("import Text\nprint(Text.join([1, 2], \",\"))\n").unwrap_err();
    assert!(err.message.contains("Expected str"));
    let err = run("import Text\nprint(Text.trim(none))\n").unwrap_err();
    assert!(err.message.contains("Cannot use 'none'"));
}

#[test]
fn handles_file_module() {
    let one = temp_file("one.txt");
    let missing = temp_file("missing.txt");
    let _ = std::fs::remove_file(&one);
    let _ = std::fs::remove_file(&missing);
    let one_str = one.to_string_lossy().into_owned();
    let missing_str = missing.to_string_lossy().into_owned();
    let source = format!(
        "import File\nprint(File.exists(\"{one_str}\"))\nFile.write(\"{one_str}\", \"hello\")\nprint(File.read(\"{one_str}\"))\nFile.append(\"{one_str}\", \" world\")\nprint(File.read(\"{one_str}\"))\nprint(File.exists(\"{one_str}\"))\nFile.delete(\"{one_str}\")\nprint(File.exists(\"{one_str}\"))\n"
    );
    assert_eq!(
        run(&source).unwrap(),
        ["false", "hello", "hello world", "true", "false"]
    );
    let _ = std::fs::remove_file(&one);
    // Missing delete warns (no change) instead of erroring.
    let source = format!("import File\nprint(File.delete(\"{missing_str}\"))\nprint(\"after\")\n");
    assert_eq!(run(&source).unwrap(), ["none", "after"]);
    // Failures are Errors and capturable; bad types and arity are Errors too.
    let source = format!("import File\nprint(File.read(\"{missing_str}\"))\n");
    let err = run(&source).unwrap_err();
    assert!(err.message.contains("Could not read file"));
    let source = format!(
        "import File\ntry to :\n    print(File.read(\"{missing_str}\"))\ncapture error :\n    print(\"caught file\")\n"
    );
    assert_eq!(run(&source).unwrap(), ["caught file"]);
    let err = run("import File\nprint(File.read(5))\n").unwrap_err();
    assert!(err.message.contains("Expected str"));
    let err = run("import File\nFile.write(\"only-one-arg\")\n").unwrap_err();
    assert!(err.message.contains("Expected 2 arguments"));
    let err = run("import File\nprint(File.read(none))\n").unwrap_err();
    assert!(err.message.contains("Cannot use 'none'"));
}

#[test]
fn handles_system_module_and_cli_wiring() {
    // os/arch are properties returning non-empty strings.
    assert_eq!(run("import System\nprint(System.os.length is > to 0)\nprint(System.arch.length is > to 0)\n").unwrap(), ["true", "true"]);
    // Injected CLI args are observed; env is a dictionary.
    assert_eq!(
        run_with_cli_args("import System\nprint(System.args())\n", vec!["luca", "prog.lucc", "a"]).unwrap(),
        ["[\"luca\", \"prog.lucc\", \"a\"]"]
    );
    assert_eq!(run("import System\nprint(System.env().length is >= to 0)\n").unwrap(), ["true"]);
    // exit validation (never invoked with valid input in-process).
    let err = run("import System\nSystem.exit(\"x\")\n").unwrap_err();
    assert!(err.message.contains("Expected int"));
    let err = run("import System\nSystem.exit()\n").unwrap_err();
    assert!(err.message.contains("Expected 1 arguments"));
    let err = run("import System\nSystem.exit(99999999999999999999)\n").unwrap_err();
    assert!(err.message.contains("out of range"));
    let err = run("import System\nSystem.exit(none)\n").unwrap_err();
    assert!(err.message.contains("Cannot use 'none'"));
    // System.os() with parens is not a thing; os/arch are bare reads.
    let err = run("import System\nprint(System.os())\n").unwrap_err();
    assert!(err.message.contains("has no member"));
}

#[test]
fn handles_stdlib_cross_feature_integration() {
    // Math + loops + conditions + collections + interpolation.
    assert_eq!(
        run("import Math\ndef var total = 0\nfor 5 times repeat :\n    total = total + 1\nprint(Math.min(total, 3))\ndef var root = Math.sqrt(total)\nprint(\"root {root}\")\n").unwrap(),
        ["3", "root 2.2360679774"]
    );
    // Text + functions + try/capture.
    assert_eq!(
        run("import Text\ndef func shout(name (str)) (str):\n    return Text.upper(name)\ntry to :\n    print(shout(42))\ncapture error :\n    print(Text.join([\"bad\", \"call\"], \"-\"))\n").unwrap(),
        ["bad-call"]
    );
    // File + ask + dict + module namespace combined.
    let path = temp_file("combo.txt");
    let _ = std::fs::remove_file(&path);
    let path_str = path.to_string_lossy().into_owned();
    let source = format!(
        "import File\nimport Text\ndef const name (str) = ask \"Name?\"\nFile.write(\"{path_str}\", Text.lower(name))\nprint(File.read(\"{path_str}\"))\nFile.delete(\"{path_str}\")\n"
    );
    let out = Interpreter::new().with_input(vec!["LUCA".to_owned()]).run_collect(&source).unwrap();
    assert_eq!(out, ["Name?", "luca"]);
    let _ = std::fs::remove_file(&path);
    // Built-in modules take precedence over same-named files.
    assert_eq!(run_with_fixtures("import Math\nprint(Math.sqrt(16))\n").unwrap(), ["4.0"]);
}

#[test]
fn handles_raise_and_capture_inspection() {
    // Valid declaration, raise, and field inspection with raise-site location.
    assert_eq!(
        run("def error NotFound :\n    code = 404\n    message = \"gone\"\ntry to :\n    raise NotFound()\ncapture error :\n    print(error[\"code\"])\n    print(error[\"message\"])\n    print(error[\"line\"])\nprint(\"after\")\n").unwrap(),
        ["404", "gone", "5", "after"]
    );
    // Overrides replace declared values; empty parens keep them.
    assert_eq!(
        run("def error E :\n    code = 1\n    message = \"m\"\ntry to :\n    raise E(code = 2, message = \"n\")\ncapture error :\n    print(error[\"code\"])\n    print(error[\"message\"])\n").unwrap(),
        ["2", "n"]
    );
    // Extra declared fields are exposed alongside the standard entries.
    assert_eq!(
        run("def error F :\n    code = 1\n    message = \"m\"\n    def var detail = \"d\"\ntry to :\n    raise F()\ncapture error :\n    print(error[\"detail\"])\n    print(error)\n").unwrap(),
        ["d", "{\"code\" = 1, \"message\" = \"m\", \"detail\" = \"d\", \"line\" = 6, \"column\" = 5}"]
    );
    // Runtime errors bind code = none plus message/line/column.
    assert_eq!(
        run("try to :\n    print(1 / 0)\ncapture error :\n    print(error[\"code\"])\n    print(error[\"message\"])\n    print(error[\"line\"])\n").unwrap(),
        ["none", "Division by zero", "2"]
    );
    // Uncaught raises propagate with the custom message at the raise site.
    let err = run("def error Oops :\n    code = 7\n    message = \"kaput\"\nraise Oops()\n").unwrap_err();
    assert!(err.message.contains("kaput"));
    assert_eq!((err.line, err.column), (4, 1));
    // Failed raises are ordinary errors, so they are capturable too.
    assert_eq!(
        run("try to :\n    raise Missing()\ncapture error :\n    print(\"caught\")\n").unwrap(),
        ["caught"]
    );
    // The binding lives only inside the capture block: outer names stay
    // visible within it, and `error` does not leak out afterwards.
    assert_eq!(
        run("def var status = \"ok\"\ntry to :\n    print(1 / 0)\ncapture error :\n    print(error[\"message\"])\n    print(status)\nprint(status)\n").unwrap(),
        ["Division by zero", "ok", "ok"]
    );
    let err = run("try to :\n    print(1 / 0)\ncapture error :\n    print(\"in\")\nprint(error)\n").unwrap_err();
    assert!(err.message.contains("is not declared"));
}

#[test]
fn handles_raise_invalid_forms() {
    // Unknown names, non-error names, and bad fields/types.
    let err = run("raise Missing()\n").unwrap_err();
    assert!(err.message.contains("is not declared"));
    let err = run("def var x = 1\nraise x()\n").unwrap_err();
    assert!(err.message.contains("is not an error definition"));
    let err = run("def func f():\n    return 1\nraise f()\n").unwrap_err();
    assert!(err.message.contains("is not an error definition"));
    let err = run("def error E :\n    code = 1\n    message = \"m\"\nraise E(nope = 2)\n").unwrap_err();
    assert!(err.message.contains("has no field"));
    let err = run("def error E :\n    code = 1\n    message = \"m\"\nraise E(code = \"str\")\n").unwrap_err();
    assert!(err.message.contains("for field 'code'"));
    // Parentheses are required; raise is statement-only.
    let err = run("def error E :\n    code = 1\n    message = \"m\"\nraise E\n").unwrap_err();
    assert!(err.message.contains("Expected '(' after error name"));
    let err = run("print(raise E())\n").unwrap_err();
    assert!(err.message.contains("is a statement, not an expression"));
    // Duplicate overrides warn and keep the first value.
    assert_eq!(
        run("def error E :\n    code = 1\n    message = \"m\"\ntry to :\n    raise E(code = 2, code = 3)\ncapture error :\n    print(error[\"code\"])\n").unwrap(),
        ["2"]
    );
    // Raising inside an error body fails the definition.
    let err = run("def error A :\n    code = 1\n    message = \"a\"\ndef error B :\n    raise A()\n").unwrap_err();
    assert!(err.message.contains('a'));
    // A raise makes following sibling statements unreachable (warning only).
    assert_eq!(
        run("def error E :\n    code = 1\n    message = \"m\"\ndef func f():\n    raise E()\n    print(\"dead\")\ntry to :\n    f()\ncapture error :\n    print(\"caught\")\n").unwrap(),
        ["caught"]
    );
}

#[test]
fn handles_raise_nesting_finally_and_modules() {
    // Inner capture sees the inner raise; an inner re-raise reaches the outer.
    assert_eq!(
        run("def error Outer :\n    code = 1\n    message = \"outer\"\ntry to :\n    try to :\n        raise Outer()\n    capture error :\n        print(error[\"message\"])\n        raise Outer(code = 2)\ncapture error :\n    print(error[\"code\"])\n").unwrap(),
        ["outer", "2"]
    );
    // finally runs around raises, then the error continues.
    assert_eq!(
        run("def error E :\n    code = 1\n    message = \"m\"\ntry to :\n    raise E()\ncapture error :\n    print(\"caught\")\nfinally :\n    print(\"fin\")\n").unwrap(),
        ["caught", "fin"]
    );
    // Module functions raise into the importer's capture with values intact.
    assert_eq!(
        run_with_fixtures("import RaiseMod\ntry to :\n    RaiseMod.explode()\ncapture error :\n    print(error[\"code\"])\n    print(error[\"message\"])\nprint(\"after\")\n").unwrap(),
        ["99", "bang", "after"]
    );
    // Uncaught module raises propagate with module context.
    let err = run_with_fixtures("import RaiseMod\nRaiseMod.explode()\n").unwrap_err();
    assert!(err.message.contains("In module 'RaiseMod'"));
    assert!(err.message.contains("bang"));
    // A bad argument to System.exit is an ordinary error, so capture traps
    // it; a valid System.exit cannot run here since it ends the process
    // (the uncapturable path with finally is verified end to end via CLI).
    assert_eq!(
        run("import System\ntry to :\n    System.exit(\"x\")\ncapture error :\n    print(\"caught\")\n").unwrap(),
        ["caught"]
    );
}
