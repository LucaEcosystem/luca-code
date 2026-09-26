# Luca Code — implemented language subset (1.0 Beta)

This document describes the 0.1–1.0 Beta language. The full authoritative design is in [locked-language-reference.md](locked-language-reference.md); where these files differ, the locked reference wins. 0.1 delivered the bare-minimum foundation; 0.2 added conditionals/blocks; 0.3 added functions; 0.4 added loops; 0.5 added collections; 0.6 added modules; 0.7 added error handling; 0.8 stabilized the implemented features; 0.9 completed input and the standard library; 1.0 Beta adds custom-error raising and caught-error inspection.

## Source files

Luca Code source files use the `.lucc` extension. Run one with `cargo run -- examples/hello.lucc` (`arithmetic.lucc`, `inference.lucc`, `comparison.lucc`, `conditional.lucc`, `functions.lucc`, `loops.lucc`, `collections.lucc` are 0.1–0.5 examples; `examples/modules/main.lucc` with sibling `helpers.lucc` is the 0.6 example; `errors.lucc` is the error-handling example (extended in 1.0 with `raise`); `integration.lucc` is the 0.8 cross-feature example; `ask.lucc` and `stdlib.lucc` are the 0.9 input/library examples).

Lexical syntax: `--` single-line and `[[ ... ]]` multiline comments, double-quoted strings with escapes, `{name}` interpolation wherever a string value is created.

## Types and declarations

The 0.1 scalar types are `int`, `dec`, `str`, `bool`, and the universal `uni`. `none` is the absence value and may occupy any declared type.

```luca
def const name (str) = "Luca"
def var visits (int) = 1
def var inferred = "hello"
visits (int) = visits + 1
def var x (int) = none
x.clear()
forget var x (int)
change x (str) = "hi"
```

Type annotations are optional when inference suffices (`def var x = 5` infers `int`); explicit `(type)` is supported. `def var` is mutable, `def const` immutable. One binding per name per scope is an Error. `forget var/const name (type)` removes a binding. `x.clear()` on a mutable `var` sets to `none` (Error on `const`). `change var name (type) = value` changes a mutable variable's type and value. A value must match its declared type (or `none`, or any for `uni`). Constants cannot be reassigned; type annotations on references/assignments are checked.

## Expressions

Implements integer and decimal literals (exact decimal, no `f64`; `20.00+1.50→21.50`, `2.50*2.00→5.00`), strings, `true`/`false`/`none`, variable references, parentheses, unary `-` and `not`, `+`, `-`, `*`, `/`, `%`, `and`, `or`, and worded comparisons `is == to`, `is != to`, `is > to`, `is >= to`, `is < to`, `is <= to`.

Precedence, highest first: `not`; `* / %`; `+ -`; comparisons; `and`; `or`. Parentheses override. `int`/`dec` may mix → `dec`, otherwise integer arithmetic stays `int`. **Division is numeric:** `10 / 4` → `2.5` (`dec`). Decimal retains meaningful scale. Division/modulo by zero and integer overflow are Errors. Incompatible-type comparison is an Error; ordering allows `int`/`dec`/`str`; `and`/`or`/`not` require `bool` and short-circuit; `none` may be compared for equality (`is == to none`) but cannot be used in arithmetic/logic.

String `+` accepts two strings only. Interpolation substitutes declared variable names, e.g. `"Hello, {name}"`.

## Output and diagnostics

`print()` takes exactly one argument, prints `none` as `none`, and interpolation renders without manual conversion. Diagnostics are actionable: `Error at line:column: message` and `cargo test`/`clippy` are clean. `cargo run -- examples/hello.lucc` demonstrates the pipeline.

## Control flow — conditionals and blocks (0.2)

Blocks use `:` and indentation (spaces/tabs, consistent per level). Empty blocks are Errors. Conditional form per locked reference: `if condition then :` / `else if condition then :` / `else :`. Conditions must be `bool`; `none`, numbers, strings, collections as conditions are Errors (`Error at line:column: Condition must be bool`). `and`/`or` short-circuit, `not` requires `bool`. Example `examples/conditional.lucc`. Gaps intentionally deferred: `import`/`try to`/`ask` inside blocks, and any unspecified edge (e.g., mixing `else if` with inconsistent indent) is left deferred and documented in `docs/implementation-status.md:31`.

## Functions (0.3)

Syntax per locked reference: `def func name(params) (return_type):` with `:` + indentation block, parameters `name (type) = default` optional, `return`/`return value`, calls `name(args)`. Example `examples/functions.lucc`. Untyped params accept any value; missing return annotation allows varying returns, bare `return` or reaching end returns `none`; explicit `return_type` requires every returned value to match; wrong argument count is Error; no overloading; recursion, nested functions, and closures (read/modify captured `var`, shadowing, global `var` access, primitive args by value) are supported. Diagnostics are source-located: `return outside function`, `Expected X arguments, found Y`, `Missing argument for 'name'`, `Duplicate parameter`, `'name' is already declared`, `Recursion limit exceeded` (64). Gaps deferred: exact default-argument evaluation order with captured `var`, `return` inside `try`/`loop` beyond immediate exit, higher-order function values, and `change`/`forget` of functions.

## Loops (0.4)

Syntax per locked reference: `for N times repeat :` where `N` is `int` expression (non-negative), `while condition repeat :` where `condition` is `bool`, and `for item from collection repeat :` (parsed but deferred in 0.4 because collections are not implemented — `Error at … not implemented in 0.4 (collections deferred)`). Bodies use `:` + indentation; empty blocks Error; `Inconsistent indentation` checked. `stop` exits the innermost loop, `skip` skips to next iteration; both outside loop are `Error at line:column: stop/skip outside loop` and source-located; `return` inside loop correctly exits the current function (`src/interpreter.rs:338`). `for` count must be `int` (`for count must be int; found …`), non-negative; `while` condition must be `bool` (`Condition must be bool`). Example `examples/loops.lucc`. Gaps deferred: `for item from collection` execution (requires collections), `dec` count handling, `stop`/`skip` inside `try`/`finally`, and any unspecified `for`/`while` edge.

## Collections (0.5)

Per locked reference: `def list names (str) = ["Luca", "Alex"]`, `def tuple data (uni) = (123, true, "Luca")`, `def set labels (str) = {"alpha", "stable"}`, `def dict person = {"Name" = "Luca" (str), "Age" = 16 (int)}`. Lists `[ ... ]`, tuples `( ... )` (single-element `(x,)` to distinguish from grouping), sets `{ ... }`, dicts `{key = value (type), ...}` with string keys. Element types use `(type)`; `uni` permits hetero; `def var x = [...]` infers collection kind. Indices 1-based (`l[1]`); missing index/key warns (`Warning at line:column`) and yields `none`. `.add`/`.remove`/`.clear` are statements; `.contains`/`.length`/`[index]` are expressions. Dicts use `.add(k, v)`, `.remove(k)`, `.contains(key = k)`/`.contains(value = v)`; duplicate key/set-add and missing remove warn with no change. `change l[i] = v` / `change d[k] = v` preserves type (missing key/index is Error). `.length` gives collection count, string chars, int/dec digit count (excl. dot/minus); `bool` invalid; `.length(decimals)` gives dec scale; `.length(items)` gives collection count. `clear()` empties collections (`[]`/`()`/`{}`) vs `none` for scalars. Equality is content-based recursive (sets/dicts unordered). `for item from collection repeat :` iterates list/tuple/set values and dict keys (item is `uni`); mutating the iterated collection is Error. Display: `["a"]`, `(1,)`, `{"a"}`, `{"k" = v}` (strings quoted in collections). Assumptions: `[[` always starts a comment, so nested lists starting with inner list need a space (`[ [1]]`); dict keys must be plain strings (no interpolation); `{}` literal is empty set (converts to empty dict for `def dict`); `def list` etc. are mutable (const collections via `def const x = [...]` inference only); `none` allowed as any element/value. Example `examples/collections.lucc`. Deferred: `def const list` syntax, non-string dict keys, set ordering guarantees beyond insertion order, `change` of collection element-type semantics beyond whole-value `change var`.

## Modules (0.6)

Per locked reference: `import Utils` (no alias, no extension/path) resolves `Utils.lucc` from search paths (program directory first, then working directory); `Utils.helper()` / `Math.sqrt(25)` namespace access via real namespace. Each module has its own scope and cannot access the importer's bindings; every top-level definition (var/const/collections/funcs) is exported automatically. Initialization runs once on first import (later imports reuse); circular imports are Errors; init errors propagate. Only `import Name` (single identifier) is supported. `Module.member` reads exported vars/collections; `Module.func(args)` calls exported funcs (arg/default/return-type checks as usual; mutable module state persists across calls). Diagnostics are source-located: `missing module 'X'`, `Circular import 'X'`, `Module 'X' is not imported`, `Module 'X' has no member 'y'`, `In module 'X': …` for init errors. Warnings in modules behave as usual. Example `examples/modules/main.lucc` + `helpers.lucc`. Deferred: built-in `Math`/`Text`/`File`/`System` (require `<Name>.lucc` files for now), `import` with paths/aliases, per-file diagnostics (locations are module-relative), importer-side mutation of module collections (`Utils.names.add` as statement, `change Utils.names[i]`), higher-order module values (`print(Utils)` errors), `import` name shadowing beyond same-scope `already declared`.

## Errors and warnings (0.7)

Per locked reference: errors stop execution unless captured; warnings are nonfatal and never captured. `try to :` runs its block; `capture error :` (optional) runs when the try block errors, binding the name `error` to the caught error for the duration of the block; `finally :` (optional) always runs — on success, after capture, and when errors propagate. A bare `try to :` with neither block is allowed (errors propagate). `return`/`stop`/`skip` inside `try`/`capture`/`finally` still run `finally` first, then propagate. Only runtime errors are capturable (division by zero, missing bindings/modules, type errors, init errors, recursion limit, …); syntax errors fail parsing before execution. The caught value is a dictionary: declared fields plus `line`/`column` for raised custom errors (read with `error["code"]`), or `code = none` plus `message`/`line`/`column` for runtime errors. Example `examples/errors.lucc`.

Custom errors use `def error Name :` with a block establishing at least `code` and `message` fields. The body executes once at definition in a fresh scope with implicit mutable `code`/`message` vars starting as `none`, so fields use bare assignment (`code = 404`, and `code = none` is allowed); `def var code` inside collides (`already declared`). Extra named values in the body become extra readable fields. A declared error is raised with `raise Name(...)`, where the parentheses (required) hold zero or more `field = value` overrides; `raise` is a statement and never yields a value, overrides must name existing fields with matching types (`none` accepted), and repeats warn keeping the first. Unknown names, non-error names, unknown fields, and mismatched types are Errors. Module error members are definitions, not values (`ErrMod.NotFound` as a value or call is an Error); module functions raise into the importer's `capture`. Gaps deferred: `raise Module.Error(...)` qualified paths (bare names only), `forget`/`change` of error definitions, `try` interaction with `import` name conflicts beyond `already declared`.

## Stabilization (0.8)

0.8 adds no new syntax. It fixes integration behavior across implemented features: unconditional `return`/`stop` with following statements in the same block reports `Warning at line:column: Unreachable code` (not an Error; `skip` excluded per reference); interpolation diagnostics point at the string literal; `return` accepts collection literals (`return [..]` / `{..}`); collections are uniform reference values (assignment aliases share storage; function parameters share so passed collections stay mutable; rebinding a name stays local); reads through `const` bindings deep-copy so constant contents can never change via alias. Example `examples/integration.lucc`.

## Input (0.9)

`ask "question"` reads one input line for the binding receiving it. The prompt string is shown as its own output line. The binding's declared type controls conversion: `int` parses (trimmed) integers, `dec` parses decimals (including integer-looking and signed input), `bool` accepts exactly `true`/`false` (trimmed, case-sensitive), `str` (and uninferred bindings) keeps the raw line. Invalid typed input is an `Error` (`Invalid input '…': expected …`, source-located at the `ask`), with no silent `none` and no retry — so it can be captured with `try to`. Running out of input (or stdin EOF/failure) is an Error. `ask` is a primary expression, so it composes (`print(ask "Q?")` reads a string); conversion to `int`/`dec`/`bool` happens only where a binding type is known (declarations, assignments, `change`, typed parameters and typed defaults). Collection targets do not convert (a string mismatches naturally). Example `examples/ask.lucc` (pipe input in, e.g. `printf 'Luca\n16\n' | cargo run -- examples/ask.lucc`).

## Standard library (0.9)

Built-in modules resolve through the same `import Name` system (built-ins take precedence over same-named files) with strict argument types, no implicit conversion, and `none` rejection (`Cannot use 'none' as an argument to 'M.f'`). Wrong argument counts are Errors. Example `examples/stdlib.lucc`.

- `Math`: `sqrt(x)` → `dec` (exact truncated root, up to 10 places, trailing zeros trimmed; negative is an Error); `pow(base, exp)` → `dec` with exact integer exponentiation by squaring (`exp` must be `int`; `0^0` is `1`; negative exponents divide, so `pow(0, negative)` is Division by zero); `abs(x)` preserves the input numeric type; `round(x)` → `int` (half away from zero); `floor`/`ceil(x)` → `int` (mathematical); `min`/`max(a, b, …)` take two or more numerics and return the winner with numeric mixing (all-`int` stays `int`, else `dec`).
- `Text`: `upper`, `lower`, `trim` (edge whitespace), `replace(text, old, new)` (all occurrences), `split(text, separator)` → `list(str)`, `join(list, separator)` → `str`. Empty-separator `split` and empty-pattern `replace` follow Rust `str` semantics (documented edge, unspecified by the reference).
- `File`: `read(path)` → `str` (missing/unreadable is an Error); `write(path, content)` overwrites; `append(path, content)` creates-or-appends; `exists(path)` → `bool` (files or directories); `delete(path)` removes files (missing warns with no change; directories fail as Errors). `write`/`append`/`delete` return `none`. Relative paths resolve against the process working directory; absolute paths allowed. All failures except missing-delete are Errors and work with `try to`.
- `System`: `os`/`arch` are bare property reads returning platform strings (`System.os()` with parens is an Error since they take none); `args()` → `list(str)` of the full process argument vector (executable, program, then script args); `env()` → dictionary of environment variables (sorted by name); `exit(int)` terminates with that code (i32 range; pending output is flushed and `finally` blocks run first, while `capture` never catches it). Reached `System.exit` terminates from any position. Unconditional `System.exit(...)` statements warn `Unreachable code` for followers, like `return`/`stop`.

Deferred (unspecified, not guessed): non-string dict keys; `def const list` form; per-file diagnostic paths (locations stay module-relative with `In module` context); `LUCA_PATH`-style library configuration; `for`-from member-path mutation tracking; `System.args` content beyond “the argument vector” (CLI passes full argv).

## Current boundary for 1.0 Beta

1.0 Beta implements every locked behavior: all types, bindings, expressions, I/O (`print`, `ask`), collections, functions, control flow, error handling with custom-error raising (`raise Name(...)`) and caught-error inspection (`error["code"]`), modules, the `Math`/`Text`/`File`/`System` standard library, and CLI integration (exit codes, `System.args` wiring, `.lucc` enforcement). `System.exit` stays uncapturable with `finally` still running; `raise Module.Error(...)` qualified paths remain unsupported (bare names only).
