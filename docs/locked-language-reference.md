# Luca Code — locked language reference

This file consolidates the decisions made in the Luca Code design conversation. It is the canonical implementation target for Luca Code 1.0 Beta. The earlier `language-spec.md` describes the implemented language; where it conflicts with this file, **this file wins**.

Do not redesign syntax or silently change semantics. If an item below lacks an exact edge-case rule, implement the smallest consistent behavior, add a test, and record the choice here without adding alternate syntax.

## Product and implementation

- Current release: **Luca Code 1.0 Beta**. `1.0 Beta` is the feature-complete Beta for validation and stabilization of the 1.0 target; the `0.x Alpha` releases built it up from the `0.1` bare-minimum foundation through `0.9`. Fix defects and verify compatibility; do not treat `1.0 Beta` as a renumbered alpha. When accepted as stable, the release becomes `1.0`.
- Luca versioning is generation + release, not SemVer: Gen 0 is `0.x Alpha`; later generations use `N.x Beta` and the same `N.x` when stable. A Beta is a preview of that exact release, e.g. `3.5 Beta` becomes `3.5`. No codenames, patch component, or separate Beta counter.
- Luca Code files use `.lucc`; Luca UI files use `.lucu`.
- Reference implementation: Rust, interpreter first, with lexer → parser → AST → type checking → interpreter. The interpreter defines behavior for a later native backend. Do not start LLVM/native code generation yet.
- Luca Code is general-purpose. Luca UI is intended to be written in Luca Code and to evolve alongside it once Code can express UI; Code must not depend on UI. Code and UI versions share a generation and are designed to work together within it. Luca Build is the integrated environment (Xcode analogy), planned after Code and UI foundations.

## Source structure and lexical syntax

Programs execute top-to-bottom from their first executable statement; there is no required `main()`. Blocks use a colon and indentation, not braces. Empty blocks are Errors.

```luca
import Math

def const name (str) = "Luca"
if name is == to "Luca" then :
    print("Hello, {name}!")
else :
    print("Hello.")
```

Identifiers contain letters, digits, and `_`, cannot start with a digit, are case-sensitive, have no spaces, and cannot be keywords. `--` begins a single-line comment; `[[ ... ]]` delimits a multiline comment. Strings use double quotes. `{name}` interpolation substitutes a variable and works wherever a string value is created, not just in `print`. No expression-interpolation syntax was locked.

## Types, bindings, and absence

Primitive types: `int`, `dec`, `str`, `bool`. `uni` is the universal/mixed type and permits heterogeneous collection values. Type annotations are optional when inference suffices; explicit `(type)` annotations are supported.

```luca
def var username = "Luca"
def var age (int) = 16
def const version (str) = "0.1"
def list names (str) = ["Luca", "Alex"]
def tuple data (uni) = (123, true, "Luca")
def set labels (str) = {"alpha", "stable"}
def dict person = {"Name" = "Luca" (str), "Age" = 16 (int)}
```

- `def var` is mutable; `def const` is immutable. There is no `let`.
- One binding per name per scope. Same-scope duplicates are Errors. `forget var name (type)` / `forget const name (type)` remove a binding; the name may then be declared again.
- `name (type) = value` updates an existing `var` without changing its type. `change var` is the explicit way to change a variable's type. The conversion mechanism is `change`; do not add `int()`, `str()`, `dec()`, implicit coercion, or a competing conversion syntax. Preserve the existing project form for `change var` and document only genuinely unresolved details.
- `const` cannot be reassigned or mutated; attempts to assign, clear, or mutate its contents are Errors.
- `none` means no value, not a separate ordinary primitive type. It may occupy a binding of any declared type. Printing or checking it is valid; using it where an operation needs a value is an Error. `x is == to none` checks it; there is no `is none` syntax.
- `x.clear()` on a mutable variable sets its value to `none` and retains its binding and declared type; it can later be assigned another value of that type. `clear()` on a constant is an Error. On a collection, `clear()` empties the collection but retains the collection binding.

## Expressions and operators

Arithmetic operators: `+`, `-`, `*`, `/`, `%`. Precedence, highest first: `not`; `* / %`; `+ -`; comparisons; `and`; `or`. Parentheses override precedence. Unary `-` accepts `int` and `dec`; unary `+`, `++`, `--`, `+=`, and `-=` are not supported.

`int` and `dec` may mix, producing `dec`; otherwise integer arithmetic remains `int`. **Division is numeric division:** `10 / 4` produces `2.5` (`dec`), not truncated integer `2`. Division and modulo by zero are Errors. Decimal values retain meaningful scale through arithmetic (e.g. `20.00 + 1.50` displays `21.50`; `2.50 * 2.00` displays `5.00`). Use exact decimal arithmetic; binary `f64` must not erase scale or produce unstable decimal output. Integer overflow is an Error.

String `+` accepts only `str + str`; mixed-type concatenation is an Error. Use interpolation to combine types.

Comparisons use the exact worded forms `is == to`, `is != to`, `is > to`, `is >= to`, `is < to`, `is <= to`.

- Equality is content-based for collections, recursively including nested collections; string equality is exact/case-sensitive; Boolean equality compares `true`/`false`. Incompatible-type comparison is an Error.
- Ordering permits `int`/`int`, `dec`/`dec`, mixed `int`/`dec`, and `str`/`str` (lexicographic). Ordering booleans, collections, or incompatible values is an Error.
- Conditions must be `bool`; no truthiness. `none`, numbers, strings, and collections as conditions are Errors.
- `and`/`or` require Boolean operands and short-circuit. `not` accepts only a Boolean expression.

## Input and output

`print()` takes exactly one argument. It prints `none` as `none`. Interpolation automatically renders values without manual conversion.

```luca
def const name (str) = "Luca"
def const age (int) = 16
print("Hello, {name}. You are {age} years old.")
```

`ask "question"` produces input directly for the binding receiving it; the binding's declared type controls conversion. Invalid typed input is an Error (no silent `none`, no automatic retry).

```luca
def const name (str) = ask "What is your name?"
def const age (int) = ask "How old are you?"
```

## Collections

Lists use `[ ... ]`, tuples `( ... )`, sets `{ ... }`, dictionaries `{key = value (type), ...}`. Lists, tuples, and sets use an element type; `uni` allows heterogeneous values. Dictionaries may have different value types. Sets are unique and unordered. **Tuples are mutable**; a later explicit decision superseded the earlier immutable description.

- List/tuple indices are 1-based. Missing indices and dictionary keys yield `none`, emit a Warning, and continue.
- Lists, tuples, and sets support `.add(value)`, `.remove(value)`, `.clear()`, `.contains(value)`. Missing `.remove` items warn and produce `none`. Adding an existing set value warns and makes no change.
- Dictionaries support `.add(key, value)`, `.remove(key)`, `.clear()`, `.contains(key = value)`, and `.contains(value = value)`. Adding a duplicate key or removing a missing key warns and makes no change. `change dict[key] = value` updates an existing entry only; its type cannot change.
- `change list[index] = value` and `change tuple[index] = value` update an item while preserving its type. Invalid type changes are Errors. Mutating the collection being iterated is an Error.
- `.length` gives collection item/entry count, string character count, and digit count for `int`/`dec` (excluding the decimal point). It is invalid for `bool`. `.length(decimals)` gives a decimal's decimal-place count; collection `.length(items)` gives item count.
- `.clear()` on collections produces the corresponding empty collection: `[]`, `()`, or `{}`. On ordinary mutable variables it produces `none` as described above.

## Functions, scope, and control flow

```luca
def func greet(name (str) = "Luca") (str):
    return "Hello, {name}!"

print(greet())
```

- Syntax: `def func name(parameters) (optional_return_type):`; parameters and return annotations are optional; defaults are allowed. Calls use `name(arguments)`. Untyped parameters accept any value.
- No return annotation means return types may vary; reaching the end or bare `return` returns `none`. An explicit return annotation requires every returned value to match. Wrong argument count is an Error. No function overloading.
- Recursion, nested functions, and closures are supported. Closures can read and modify captured `var` bindings subject to mutability. Top-level bindings are global and functions may read/write global `var`s. Locals may shadow globals and otherwise remain local. Primitive arguments are independent values; passed collections remain mutable. No `ref`, `inout`, or pointer parameter syntax.
- `return` outside a function is an Error; inside a loop it exits the current function. `stop` exits a loop and `skip` skips to its next iteration; either outside a loop is an Error. Runtime recursion limit exceeded is an Error; the exact limit is implementation-defined.
- Conditional form: `if condition then :`, `else if condition then :`, `else :`. Loop forms: `for N times repeat :`, `for item from collection repeat :`, `while condition repeat :`.
- An unconditional `return`, `stop`, or `System.exit()` may make following code unreachable; report a Warning, not an Error.

## Errors and warnings

Errors stop execution unless captured. Warnings are nonfatal. Confirmed warning cases include missing/out-of-range collection reads, removing absent collection members/keys, duplicate set additions, duplicate dictionary keys, and deleting a missing file.

```luca
try to :
    File.read("data.txt")
capture error :
    print("Could not read the file.")
finally :
    print("Cleanup")
```

`finally` always runs. Errors may be captured; warnings remain warnings. Custom errors use `def error Name :` and at least `code` and `message` fields (the examples allow `code = none`). Do not create alternate error-handling syntax.

Locked custom-error decision (1.0 Beta): a declared custom error is raised with `raise ErrorName(...)`, where the parentheses hold zero or more `field = value` overrides separated by commas (a trailing comma is allowed); empty `()` raises with the declared field values, and a repeated field warns and keeps its first value, as in dictionary literals. An override field must already exist on the declared error, and its value must match the declared field's type; `none` is accepted for any field. The parentheses are required: `raise` is a statement and never yields a value.

Catching binds the error: in `capture error :`, the name `error` is bound for the duration of the capture block to the caught error value, shadowing any outer `error` binding. The caught value is a dictionary holding the error's declared fields plus the standard `line` and `column` entries, read with the existing dictionary indexing (`error["code"]`); `line` and `column` are the location where the error was raised, or where it occurred for runtime errors (which additionally carry `code = none`). A `raise` with an unknown name, a non-error name, an unknown field, or a mismatched field type is itself an Error. `System.exit` remains uncapturable and `finally` still runs around it.

Other confirmed Errors: invalid types/operations, `none` used in value-requiring operations, invalid `ask` conversion, wrong argument count, missing module, circular import, failed file operations, division/modulo by zero, invalid comparison/condition, and integer overflow.

## Modules

```luca
import Math
import Utils
Math.sqrt(25)
Utils.helper()
```

- Built-in and user modules use the same system, are accessed by their real namespace, and have no aliases.
- `import Utils` resolves `Utils.lucc` from project/library module search paths; the extension/path is normally omitted.
- Each module has its own scope and cannot directly access the importing file's bindings. Every top-level module definition is exported automatically; no `export` keyword.
- Module initialization runs once on first import; later imports reuse it. Circular imports are Errors. Initialization Errors propagate and may be captured by `try to`.

## Core standard library

All library calls use strict argument types, no implicit conversion, and reject `none` when a real value is required.

- `Math.sqrt`, `Math.pow`, `Math.abs`, `Math.round`, `Math.floor`, `Math.ceil`, `Math.min`, `Math.max`. `sqrt`/`pow` return `dec`; `round`/`floor`/`ceil` return `int`; floor/ceil use mathematical rounding. `min`/`max` accept two or more numeric values and follow numeric mixing/precision rules.
- `Text.upper`, `Text.lower`, `Text.trim`, `Text.replace`, `Text.split`, `Text.join`. Transformations return new strings; trim removes edge whitespace; replace replaces all occurrences; split(text, separator) returns `list(str)`; join(list(str), separator) returns `str`.
- `File.read`, `File.write`, `File.append`, `File.exists`, `File.delete`. Paths are strings; relative to current working directory; absolute paths allowed. Write overwrites, append appends, exists checks files and returns bool. Missing delete warns/no change; other failed operations are Errors.
- `System.os` and `System.arch` return `str`; `System.exit(int)` exits with that code; `System.args()` returns `list(str)`; `System.env()` returns a dictionary keyed by environment-variable name.

## Ecosystem compatibility policy

Code and UI share a generation; minor releases in a generation should remain compatible. Only the current generation is actively supported. A new generation starts a one-month migration period; the older generation becomes legacy after that, then leaves normal downloads one month later. Preserve historic releases in an archive. Existing installations keep running indefinitely; Luca does not remotely revoke them.

## Completion requirement

The current interpreter is a starting subset, not completion. The full reference is the target across the 0.x Alpha releases, not all a 0.1 requirement. Keep a checked status list in `docs/implementation-status.md`; continue from one coherent milestone to the next without waiting for per-feature approval. Resolve any previously omitted edge detail consistently and record it here.

### 0.1 Alpha — bare minimum

Deliver a small, real, runnable language foundation: Rust crate; lexer, parser, AST, basic type checking, interpreter; typed and inferred scalar variable/constant declarations; assignment; `print`; core integer/decimal arithmetic; strings and variable interpolation; actionable diagnostics; tests and a few `.lucc` examples. The locked division rule (`10 / 4` → `2.5` as `dec`) and decimal behavior apply from the first arithmetic implementation. Keep the release label at `0.1 Alpha` while completing this foundation.

### 0.2–0.8 Alpha — grow the language

Add the remaining locked capabilities in coherent releases, with each release usable, tested, and documented. The order may follow dependencies, but must not change the specified syntax or semantics. Use `docs/implementation-status.md` to track features for the current release and later alphas. Do not claim the entire language is complete under `0.1`.

### 0.9 Alpha — prepare the Beta candidate

Complete and test every feature in this reference, resolve implementation defects, document the language, and verify real `.lucc` programs. The exit criterion for `0.9 Alpha` is a feature-complete, well-tested candidate ready to become `1.0 Beta`.

### 1.0 Beta — stabilize

Use Beta for validation and stabilization of the feature-complete 1.0 target. Fix defects and verify compatibility; do not treat `1.0 Beta` as a renumbered alpha. When accepted as stable, the release becomes `1.0`.
