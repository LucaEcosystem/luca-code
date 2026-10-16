# Implementation status

The current release is **Luca Code 1.0 Beta** — the feature-complete Beta: everything in 0.9 plus custom-error raising (`raise`) and caught-error inspection. The full design in `locked-language-reference.md` is implemented; 0.1 delivered the bare-minimum foundation, 0.2 delivered conditionals, 0.3 delivered functions, 0.4 delivered loops, 0.5 delivered collections, 0.6 delivered modules, 0.7 delivered error handling, 0.8 stabilized the implementation, 0.9 delivered input and the standard library, and 1.0 Beta closes the last gap (custom-error raising/inspection) for validation and stabilization toward 1.0.

Track this checklist against `locked-language-reference.md`.

- [x] Initial Rust crate and interpreter pipeline — `Cargo.toml:1`, `src/lib.rs:1` (0.1)
- [x] Basic typed declarations, reassignment, scalar arithmetic, string interpolation, and `print` — `src/ast.rs:8`, `src/parser.rs:20`, `src/interpreter.rs:34`, `tests/interpreter.rs:4` (0.1)
- [x] Correct numeric division: `10 / 4` must return `2.5` as `dec` — `src/types.rs:34` `Dec`, `src/interpreter.rs:88` numeric division, `tests/interpreter.rs:28` (0.1)
- [x] Exact `dec` arithmetic with scale preservation — `unscaled i128 + scale≤28`, `add`/`sub` max scale, `mul` sum trimmed to `max_scale`, `div` minimal scale `≤28`, `0.1+0.2=0.3` (`src/types.rs:96`, `tests/interpreter.rs:32`), no `f64` (0.1)
- [x] Confirm the 0.1 foundation has actionable diagnostics, README/example coverage, and passing tests/Clippy — `src/error.rs:16` `Error at line:column`, `README.md:1`, `examples/hello.lucc:1` + `arithmetic.lucc` + `inference.lucc` + `comparison.lucc`, `tests/interpreter.rs:1` 12 tests (0.1 verified 2026-09-24)
- [x] Full bindings: optional inference, `none`, `clear`, `forget`, and `change` — `src/types.rs:227` `Value::None`, `src/lexer.rs:97` `none`/`forget`/`change`, `src/ast.rs:9` `Declare{Option<Type>}` + `Forget`/`Clear`/`Change`, `src/parser.rs:20`, `src/interpreter.rs:40` (0.1 early, hardened in 0.2)
- [x] Multiline comments and full lexical error handling — `src/lexer.rs:30` `[[ ... ]]`, `--` (0.1 early)
- [x] Comparisons and logical operators — `src/lexer.rs:4` `Is`/`To`/`And`/`Or`/`Not` + `==`/`!=`/`>=`/`<=`, `src/ast.rs:28`, `src/parser.rs:88` precedence, `src/interpreter.rs:118` short-circuit (0.1 early)
- [x] Conditional execution and blocks — `if`/`else if`/`else` with `:` + indentation (`src/lexer.rs:4` `If`/`Then`/`Else`/`Indent`/`Dedent`/`Colon`, `src/ast.rs:9` `Block` + `Statement::If`, `src/parser.rs:25` `if_statement`/`block`, `src/interpreter.rs:34` scoped `scopes: Vec<HashMap>` + `execute_if`, `tests/interpreter.rs:91` 13 tests, `examples/conditional.lucc:1`, verified 2026-09-24)
- [x] Functions — `def func name(params) (return_type):` with `:` + indentation, parameters `name (type) = default` optional, `return`/`return value`, calls `name(args)` (`src/lexer.rs:4` `Func`/`Return`, `src/ast.rs:9` `Param` + `FuncDef`/`Return`/`Call`, `src/parser.rs:30` `func_def`/`primary` Call vs type assertion, `src/interpreter.rs:11` `Function` + `func_scopes` + `call_depth:64` + `ExecSignal::Return`, `tests/interpreter.rs:91` 15 tests, `examples/functions.lucc:1`, verified 2026-09-24)
- [x] Loops and iteration control — `for N times repeat :`, `while condition repeat :` with `:` + indentation, `stop`/`skip` (`src/lexer.rs:4` `For`/`While`/`Times`/`Repeat`/`Stop`/`Skip`, `src/ast.rs:9` `ForTimes`/`While`/`Stop`/`Skip`, `src/parser.rs:25` `for_statement`/`while_statement`, `src/interpreter.rs:34` `loop_depth` + `ExecSignal::Stop`/`Skip` + `execute_for_times`/`execute_while`, `tests/interpreter.rs:91` 16 tests, `examples/loops.lucc:1`, verified 2026-09-24)
- [x] Collections with warnings and type checks — `def list/tuple/set/dict`, literals `[ ]`/`( )`/`{ }`/`{k = v (type)}`, 1-based indexing, `.add`/`.remove`/`.clear`, `.contains`/`.length`, `change name[i]`, content equality, `for item from collection` (`src/lexer.rs:4` `List`/`Tuple`/`Set`/`Dict` + brackets/braces, `src/ast.rs:9` `DeclareCollection`/`ChangeIndex`/`CollectionAdd`/`Remove` + `List`/`Tuple`/`Set`/`Dict`/`Index`/`Contains`/`Length`, `src/types.rs:1` `CollectionKind`/`DictEntry`/`values_equal`, `src/parser.rs:25` collection declarations/literals/postfix, `src/interpreter.rs:34` `BindingType` + `execute_for_each`/`change_index`/`collection_add`/`remove` + warnings, `tests/interpreter.rs:91` 18 tests, `examples/collections.lucc:1`, verified 2026-09-25)
- [x] Modules and imports — `import Name` → `Name.lucc` (program-dir-first search), `Module.member`/`Module.func(args)`, own scope, auto-exported top-levels, once-only init, circular-import Errors, init-error propagation (`src/lexer.rs:4` `Import`, `src/ast.rs:9` `Import`/`Member`/`MemberCall`, `src/parser.rs:25` import/member parsing, `src/interpreter.rs:34` `modules`/`loading`/`search_paths` + `import_module`/`call_module_function`, `src/main.rs:1` program-dir paths, `tests/interpreter.rs:91` 21 tests with `tests/fixtures/`, `examples/modules/main.lucc` + `helpers.lucc`, verified 2026-09-25)
- [x] Error handling — `try to :` / `capture error :` / `finally :` (optional capture/finally, finally always runs, signals propagate after finally, runtime errors only), `def error Name :` with `code`/`message` fields (`src/lexer.rs:4` `Try`/`Capture`/`Finally`/`Error`, `src/ast.rs:9` `Try`/`ErrorDef`, `src/parser.rs:25` `try_statement` + `def error`, `src/interpreter.rs:34` `error_scopes` + `execute_try`/`run_finally`/`execute_error_def` + module error export, `tests/interpreter.rs:91` 24 tests, `examples/errors.lucc:1`, verified 2026-09-25)
- [x] 0.8 stabilization — unreachable-code warnings for unconditional `return`/`stop`, interpolation source locations, uniform shared collections (mutable through calls/aliases, `const` deep-copy on read), `return` with collection literals, full operator/arity/keyword rejection coverage, cross-feature integration tests/examples (`src/interpreter.rs:34` `warn_unreachable_after`/`statement_location`, `src/ast.rs:9` `StringTemplate{value,line,column}`, `src/types.rs:1` `Rc<RefCell>` collection storage + `new_list/tuple/set/dict` + `deep_copy_collection`, `src/parser.rs:25` return expression-start set, `tests/interpreter.rs:91` 30 tests, `examples/integration.lucc:1`, verified 2026-09-25)
- [x] Input (`ask`) — `ask "question"` prompt line plus stdin/scripted input, declared-type conversion (`int`/`dec`/`bool`/`str`), invalid-input and EOF Errors, capturable via `try to` (`src/lexer.rs:4` `Ask`, `src/ast.rs:9` `Expr::Ask`, `src/parser.rs:25` ask primary, `src/interpreter.rs:34` `input_lines` + `read_input_line` + `apply_ask_conversion` + `convert_input_to_type`, `src/lib.rs:10` `run_with_input`, `tests/interpreter.rs:91` ask tests, `examples/ask.lucc:1`, verified 2026-09-26)
- [x] Standard library — `Math` (exact `sqrt`/`pow`/`abs`/`round`/`floor`/`ceil`/`min`/`max`), `Text` (`upper`/`lower`/`trim`/`replace`/`split`/`join`), `File` (`read`/`write`/`append`/`exists`/`delete` with missing-delete warning), `System` (`os`/`arch` properties, `args()`/`env()`, `exit(int)`) with strict types, `none` rejection, arity checks, builtin-first `import` precedence (`src/stdlib.rs:1`, `src/interpreter.rs:34` `loaded_builtins` + `call_native_function` + exit marker, `tests/interpreter.rs:91` stdlib tests incl. `tests/fixtures/Math.lucc` precedence decoy, `examples/stdlib.lucc:1`, verified 2026-09-26)
- [x] CLI integration — full argv for `System.args()`, exit codes with flushed output, `System.exit` unreachable warnings, `.lucc` enforcement (`src/main.rs:1`, `src/interpreter.rs:34` exit-marker flush, manually verified exit paths, verified 2026-09-26)
- [x] Custom-error raising and inspection — `raise Name(...)` statement with `field = value` overrides (required parens, trailing comma allowed, duplicate override warns first-wins, unknown/non-error names and unknown/mismatched fields are Errors, source-located at the name/field), `capture error` binds the caught error dict (declared fields plus `line`/`column`; runtime errors carry `code = none`), `error` usable as an expression name, `System.exit` still uncapturable with `finally` running (`src/lexer.rs:8` `Raise`, `src/ast.rs:32` `RaiseArg` + `Statement::Raise`, `src/parser.rs:262` `raise_statement` + expression-position rejection, `src/error.rs:15` `error_value` + `raised`, `src/interpreter.rs:24` `CustomError.fields` + `exec_raise:891` + capture binding `:843` + value-preserving module wrap `:1108`/`:2600`, `tests/interpreter.rs:939` 3 tests with `tests/fixtures/RaiseMod.lucc`, `examples/errors.lucc:35`, verified 2026-09-26)

## Verified 0.2 Alpha (2026-09-24)

```
cargo test                          — 13 passed, 0 failed (0.2)
cargo clippy --all-targets --all-features -- -D warnings — clean
cargo run -- examples/hello.lucc    — Hello, Luca! / Visit 2 / 3.0
cargo run -- examples/arithmetic.lucc — 2.5 / 14 / 40 / 6 / 2 / 21.50 / 5.00 / 0.3
cargo run -- examples/inference.lucc — Hello, Luca! Visit 2 / 3.14 / none / 5
cargo run -- examples/comparison.lucc — true ×9
cargo run -- examples/conditional.lucc — Hello, Luca! / B / nested / after inner / and true / not false / inner / outer
```

## Verified 0.3 Alpha (2026-09-24)

```
cargo test                          — 15 passed, 0 failed (0.3)
cargo clippy --all-targets --all-features -- -D warnings — clean
cargo run -- examples/hello.lucc    — Hello, Luca! / Visit 2 / 3.0
cargo run -- examples/functions.lucc — Hello, Luca! / Hello, Alex! / 5 / 30 / 120 / 2 / 15 / 5 / hi
```

## Verified 0.4 Alpha (2026-09-24)

```
cargo test                          — 16 passed, 0 failed (0.4)
cargo clippy --all-targets --all-features -- -D warnings — clean
cargo run -- examples/hello.lucc    — Hello, Luca! / Visit 2 / 3.0
cargo run -- examples/loops.lucc    — 5 / 10 / hi×3 / 3 / 13 / nested×6 / 5/4/3/done / 3/2/1
```

## Verified 0.5 Alpha (2026-09-25)

```
cargo test                          — 18 passed, 0 failed (0.5)
cargo clippy --all-targets --all-features -- -D warnings — clean
cargo run -- examples/hello.lucc    — Hello, Luca! / Visit 2 / 3.0
cargo run -- examples/collections.lucc — ["Luca", "Alex"] / Lucca…(see examples/collections.lucc) / 2 / 2
```

## Verified 0.6 Alpha (2026-09-25)

```
cargo test                          — 21 passed, 0 failed (0.6)
cargo clippy --all-targets --all-features -- -D warnings — clean
cargo run -- examples/hello.lucc    — Hello, Luca! / Visit 2 / 3.0
cargo run -- examples/modules/main.lucc — hi World / hi Luca / 1.0 / demo / true / 1 / 2 / demo / modules
```

## Verified 0.7 Alpha (2026-09-25)

```
cargo test                          — 24 passed, 0 failed (0.7)
cargo clippy --all-targets --all-features -- -D warnings — clean
cargo run -- examples/hello.lucc    — Hello, Luca! / Visit 2 / 3.0
cargo run -- examples/errors.lucc   — caught division / cleanup / no trouble / always / inner finally / outer caught / 5.0 / 0
```

## Verified 0.8 Alpha (2026-09-25)

```
cargo test (from clean build)       — 30 passed, 0 failed (0.8)
cargo clippy --all-targets --all-features -- -D warnings — clean
cargo run -- examples/hello.lucc    — Hello, Luca! / Visit 2 / 3.0
cargo run -- examples/integration.lucc — skipped oops / 35.50 / skipped oops / 1 over 10.00 / {"retries" = 5} / true / no crash / done
```

Diagnostics verified for 0.8: `Warning at line:column: Unreachable code` after unconditional `return`/`stop` (Warning, execution continues; `skip` excluded per reference); interpolation errors at the string literal (`3:7`, not `1:1`); `Expected an expression` for unary `+`; `Expected '=' in assignment` for `++`/`+=`/`let`; `print accepts exactly one argument`; `Expected a declaration name` for keyword identifiers; `Cannot order given types` / `Cannot order 'none'`; `Cannot change constant` through direct and alias paths; shared-collection updates visible to callers; plus all 0.1–0.7 diagnostics.

Deferred gaps for 0.8 stabilization (locked reference silent or explicitly deferred; recorded rather than guessed):

- `def var m = l` shares storage (uniform reference values; only argument sharing is locked — assignment aliasing is the smallest consistent extension)
- Reads through `const` collection bindings deep-copy (keeps `const cannot be mutated` total; direct `c.add` still Errors)
- Nested-collection mutation through aliases/`for` items propagates via sharing (unspecified; falls out of the model)
- Dict keys remain plain static strings (no interpolation in keys; values interpolate everywhere)
- `change`/`forget` of functions and error definitions (existing gaps, unchanged)
- Raising custom errors / inspecting caught errors (no locked syntax; unchanged from 0.7)

Diagnostics verified for 0.7: `Error at line:column` for `Expected 'to' after 'try'`, `Expected ':' after 'try to'`, `Expected 'error' after 'capture'`, `Expected ':' after 'capture error'`, `Expected ':' after 'finally'`, `Expected indented block`/`Empty block` (try/capture/finally bodies), `return outside function`-style propagation intact, `'X' is already declared` (error/var/func/module conflicts), `Custom error`-body failures propagate, `is a function`-style member diagnostics extended to `'X' is an error definition; raising custom errors is not supported in 0.7`, plus all 0.1–0.6 diagnostics. Also fixed as required groundwork: hot-path dispatch factored into helpers (`exec_declare`, `bind_call_params`, literal builders) so default-thread stacks hold 64-deep recursion in unoptimized builds (depth-63 verified at 2MB; was overflowing at ~39 before), and failed local calls no longer leak partial param scopes (matters once errors are capturable).

Deferred gaps for error handling (locked reference defines the forms above; following are not specified and intentionally left deferred rather than guessed):

- Raising custom errors (no `raise`/`throw` syntax locked) and inspecting caught error details (`error.code` in `capture` blocks; `capture error` binds nothing in 0.7)
- `def error` payload details beyond `code`/`message` presence (extra body statements allowed and discarded; field types unchecked; unassigned fields default to `none`)
- `forget`/`change` of error definitions (deferred; `forget var E` on an error reports not-declared)
- `stop`/`skip`/`return` inside `capture`/`finally` beyond natural signal propagation (implemented via propagation; `try`-specific interaction rules unspecified)
- Uncaught-error output buffering (pre-existing: buffered `print` output is discarded when an error propagates uncaught; `finally` still runs)
- Parse-time (syntax/lex) errors are not capturable (parsing precedes execution)
- `error` as a general identifier (now a keyword; use another name)
- Unreachable-code warnings for `return`/`stop`/`System.exit` (locked line 109; deferred with CLI hardening)

Diagnostics verified for 0.6: `Error at line:column` for `missing module 'X'`, `Circular import 'X'`, `Module 'X' is not imported`, `Module 'X' has no member 'y'`, `'f' is not a function in module 'X'`, `'f' is a function; use 'M.f(...)'`, `In module 'X': …` (init-error propagation), `'X' is already declared` (module/var/func name conflict), plus all 0.1–0.5 diagnostics.

Diagnostics verified for 0.5: `Error at line:column` for `Expected list, found …`, `Expected str, found int` (element), `Index must be int`, `Index out of range` (change), `Dictionary keys must be strings`, `Key 'x' does not exist`, `Cannot change type of 'k'`, `Cannot index set`, `contains(key/value…)` misuse, `length not supported for bool`, `Cannot change constant`, `Cannot mutate collection being iterated`, `Incompatible types for comparison`, `Cannot order given types`, plus `Warning at line:column` (nonfatal, continue) for `Index out of range` (read), `Key 'x' not found`, `Value not found; no change`, `Set value already exists`, `Duplicate dictionary key`, `Duplicate set value`, plus all 0.1–0.4 diagnostics.

Deferred gaps for collections (locked reference defines forms above; following are not specified and intentionally left deferred rather than guessed):

- `def const list/tuple/set/dict` syntax (not shown; const collections currently only via `def const x = [...]` inference, which are immutable)
- Non-string dictionary keys (current requires string keys; `.add`/`change`/index all require `str`)
- Set display/iteration order beyond insertion order (spec says unordered; current preserves insertion order deterministically)
- `for item from` over strings/ints (current requires collections; dict iterates keys)
- `.length` on `none`, `.length(x)` with non-`decimals`/`items` args, `.contains` single-arg on dicts (current errors; unspecified)
- `change` of a collection's element-type annotation semantics beyond whole-value `change var` (current updates stored element type)
- `forget list/tuple/set/dict` vs `forget var` equivalence (both accepted; `forget const` on collections errors since mutable)
- Exact `Warning` text/stderr channel (current `Warning at line:column` via stderr; `run()` output excludes warnings)

Diagnostics verified for 0.4: `Error at line:column` for `for count must be int; found …`, `for count must be non-negative`, `Condition must be bool; found …` (while), `stop outside loop`, `skip outside loop`, `Expected indented block after ':'`, `Expected 'times'`, `Expected 'repeat'`, `Empty block is not allowed`, plus all 0.1–0.3 diagnostics.

Deferred gaps for loops (locked reference specifies `for N times repeat :` and `while condition repeat :` with `:`+indentation and `stop`/`skip`, but following are not specified and intentionally left deferred):

- `for item from collection repeat :` — implemented in 0.5 (`execute_for_each`: list/tuple/set values, dict keys); the stale 0.4-deferred note is superseded
- Exact handling of `for` with non-integer `dec` count (deferred; current requires `int`)
- `stop`/`skip` inside `try to`/`finally` — implemented in 0.7 via signal propagation (signals run `finally` first, then propagate)
- `return` inside loop already correctly exits function, but `stop`/`skip` inside function without loop is already Error

Diagnostics verified for 0.3: `Error at line:column` for `return outside function`, `Expected X arguments, found Y`, `Missing argument for 'name'`, `Duplicate parameter`, `'name' is already declared`, `'name' is not declared`, `Expected int, found str` (param/return type), `Recursion limit exceeded` (limit 64), plus all 0.1–0.2 diagnostics.

Deferred gaps for functions (locked reference specifies `def func name(params) (return_type):` with `:` + indentation, calls `name(args)`, defaults, no overloading, recursion/nested/closures, but following are not specified and intentionally left deferred):

- Exact evaluation order of default arguments when capturing outer `var` (deferred; current evaluates defaults at call time in caller's scope)
- `return` inside `try to`/`loop` interaction beyond immediate function exit (deferred until those features)
- `change` of function binding vs `forget` of function (deferred; current treats like var)
- Returning functions as values / higher-order calls (not specified, deferred)

Deferred gaps for modules (locked reference defines `import Name`, real-namespace access, no aliases, `Name.lucc` resolution, own scope, auto-export, once-only init, circular-import Errors, init-error propagation; following are not specified and intentionally left deferred rather than guessed):

- Built-in `Math`/`Text`/`File`/`System` (implemented in 0.9 as natives with builtin-first `import` precedence; same-named `.lucc` files are ignored)
- `import` with paths (`a/b`), extensions (`import Utils.lucc`), or aliases (parser accepts single identifier only)
- Per-file diagnostics (error locations are module-relative `line:column` with `In module 'X':` prefix; no file-path tracking yet)
- Importer-side mutation of module collections (`Utils.names.add(...)` as statement, `change Utils.names[i] = ...` — parsed as errors; only module-owned funcs mutate module state for now)
- Higher-order module values (`print(Utils)` / returning module funcs as values — errors; not specified)
- `import` name shadowing beyond same-scope `already declared` (module names reserved globally once imported)
- Module search beyond program-dir + working-directory (no `LUCA_PATH`/library-path config yet)
- `for x from Utils.items` mutation tracking (only simple variable names tracked in `iterating`; member paths not tracked)

## Verified 0.9 Alpha (2026-09-26)

```
cargo test (from clean build)       — 36 passed, 0 failed (0.9)
cargo clippy --all-targets --all-features -- -D warnings — clean
cargo run -- examples/hello.lucc    — Hello, Luca! / Visit 2 / 3.0
cargo run -- examples/ask.lucc (piped input) — prompts + greeting
cargo run -- examples/stdlib.lucc   — exact Math/Text/File/System outputs
cargo run -- examples/modules/main.lucc — unchanged 0.6 outputs
System.exit paths (manual CLI)      — exit 3 with flushed output + Warning; exit 7 with finally; exit 5 from function
```

All previously verified outputs (0.1–0.8) re-verified unchanged during 0.9 development.

Diagnostics verified for 0.9: `Invalid input '…': expected …` at the `ask`, `No more input available`, prompt-must-be-`str`, `Math.sqrt of negative number`, `Math.pow exponent must be int`, `Expected int or dec, found …`, `Expected at least 2 arguments`, `Could not read/write/append/delete file '…'`, `System.exit code out of range`, `missing module`/`Circular import` unchanged, `Module 'M' has no member` for unknown natives, `is a function` guidance (including `System.os()` vs `System.os`), exit-marker flow (`try` never captures exit; `finally` runs; output flushes), plus all 0.1–0.8 diagnostics.

Deferred gaps for 0.9 (unspecified in the reference; recorded rather than guessed):

- Raising custom errors and inspecting caught error details (no `raise`/`throw` syntax locked; `def error` validates/stores only; `capture error` binds nothing)
- `def error` payload details beyond `code`/`message` (extra body statements allowed; field types unchecked; defaults `none`)
- `forget`/`change` of error definitions and functions (existing gaps, unchanged)
- Non-string dict keys; `def const list` form; per-file diagnostic paths (locations stay module-relative with `In module` context on calls too); `LUCA_PATH` configuration
- `for`-from member-path mutation tracking; nested-alias mutation propagation (falls out of sharing)
- `System.args` content beyond the full argument vector (CLI passes full argv: executable, program, script args)
- Empty-separator `split` / empty-pattern `replace` follow Rust `str` semantics (unspecified edge)
- `System.exit` in expression position terminates identically (no value semantics; only direct statements warn unreachable)
- Uncaught-error output buffering (pre-existing: buffered prints flush only on success or exit, not on uncaught errors)

## Remaining work and 1.0 Beta readiness verdict

Every locked behavior is implemented and verified above, including the former open item **custom-error raising/inspection** (locked custom-error decision in `locked-language-reference.md`, implemented in 1.0 Beta).

Verdict: **1.0 Beta is declared** — the locked feature set is fully implemented and verified: clean-build suite green, strict Clippy clean, all examples runnable, diagnostics source-located. Beta work from here is validation and stabilization toward 1.0 (defect fixes and compatibility verification only).

## Verified 1.0 Beta (2026-09-26)

```
cargo test (from clean build)       — 39 passed, 0 failed (1.0 Beta)
cargo clippy --all-targets --all-features -- -D warnings — clean
cargo run -- examples/hello.lucc    — Hello, Luca! / Visit 2 / 3.0
cargo run -- examples/errors.lucc   — caught division / cleanup / no trouble / always / inner finally / outer caught / 5.0 / 0 / 503 / unavailable / 41 / 500 / boom
cargo run -- examples/ask.lucc (piped input) — prompts + greeting (unchanged)
cargo run -- examples/stdlib.lucc   — exact Math/Text/File/System outputs (unchanged)
cargo run -- examples/modules/main.lucc — unchanged 0.6 outputs
System.exit paths (manual CLI)      — exit 9 with finally + capture skipped (unchanged); bad-arg exit is an ordinary capturable error
Module raise (manual CLI)           — RaiseMod.explode() caught with code 99 / message bang / execution continues; uncaught carries `In module 'RaiseMod'` context
```

All previously verified outputs (0.1–0.9) re-verified unchanged during 1.0 Beta development.

Diagnostics verified for 1.0 Beta: `'X' is not declared` (unknown raise name) at the raise name, `'X' is not an error definition` (var/func names) at the raise name, `'E' has no field 'f'` at the override field, `Expected int, found str for field 'code'` / `Cannot change type of field 'f'` at the raise statement, `Expected '(' after error name`, `'raise' is a statement, not an expression`, `Warning at line:column: Duplicate field 'f'` (first wins), `Warning at line:column: Unreachable code` after `raise`, `'M' is an error definition, not a value/function` for member reads/calls, plus all 0.1–0.9 diagnostics.

Deferred gaps for 1.0 Beta (unspecified in the reference; recorded rather than guessed):

- `raise Module.Error(...)` qualified paths (bare error names only; member reads/calls report not-a-value/not-a-function)
- `forget`/`change` of error definitions and functions (existing gaps, unchanged)
- Non-string dict keys; `def const list` form; per-file diagnostic paths (locations stay module-relative with `In module` context on calls too); `LUCA_PATH` configuration
- `for`-from member-path mutation tracking; nested-alias mutation propagation (falls out of sharing)
- `System.args` content beyond the full argument vector (CLI passes full argv: executable, program, script args)
- Empty-separator `split` / empty-pattern `replace` follow Rust `str` semantics (unspecified edge)
- `System.exit` in expression position terminates identically (no value semantics; only direct statements warn unreachable)
- Uncaught-error output buffering (pre-existing: buffered prints flush only on success or exit, not on uncaught errors)

## Distribution readiness (1.0 Beta, 2026-09-26)

First-run experience work (CLI install, VS Code extension, docs, CI). The verified language implementation is untouched except a CLI-only `--help`/`--version` addition (`src/main.rs`, no language-syntax change); 39-test suite and strict Clippy still green.

What was built:

- Terminal install: `scripts/install.sh` (macOS/Linux) and `scripts/install.ps1` (Windows) — Rust auto-install via rustup, local-checkout or clone mode, always-`--force` so re-running upgrades, `luca --version` self-check, copy-paste `PATH` fix on failure. `luca --version` prints `luca 1.0 Beta`; `luca --help` prints usage (exit 0); usage/extension errors stay exit 2.
- VS Code extension (`editors/vscode/`, zero runtime dependencies): `.lucc` TextMate grammar, 10 snippets (`try`, `error`, `raise`, `func`, `if`, `for`, `while`, `ask`, `import`, `print`), `Luca: New Luca File` / `Luca: Run Luca File` commands with title-bar run button, `luca.cliPath` setting with `PATH` discovery, two run modes — integrated terminal by default (`luca.runInTerminal`, supports `ask` input) or **Output > Luca Code** channel with `Error/Warning at line:column` → editor diagnostics. 10 `npm test` unit tests (manifest wiring, grammar regexes, diagnostics parsing, CLI discovery, run-command quoting).
- Docs: `docs/install.md` (install/upgrade/uninstall per OS), `docs/first-program.md` (verified outputs), `docs/vscode.md` (setup/usage), `docs/troubleshooting.md`, README entry section.
- CI (`.github/workflows/ci.yml`, runs on push to `main` + PRs, publishes nothing): Rust test+Clippy+example smoke on ubuntu/macos/windows; install-script syntax + full checkout install on all three OSs; extension `npm test` + `vsce package` with `.vsix` artifact upload.

Local vs CI verification ledger:

| Check | Local (this Mac, arm64) | CI (on push) |
|---|---|---|
| `cargo test` 39 passed, Clippy clean | verified | ubuntu + macOS + Windows runners |
| `install.sh`: syntax, fresh install, upgrade re-run, PATH-hint (exit 1), uninstall round-trip | verified (sandboxed `CARGO_HOME`) | ubuntu + macOS runners |
| `install.ps1` | syntax reviewed only (no `pwsh` locally) | Windows runner (parse + full install) |
| Extension `npm test` 9 passed, `vsce package`, real VS Code 1.139.1 sandbox install/uninstall | verified | ubuntu runner (`npm test` + package) |
| In-editor UX (highlight colors, run button, diagnostics display) | not verified headless | not covered — manual launch checklist |
| `cargo install` release build | verified (macOS arm64) | ubuntu + macOS + Windows via scripts job |

Placeholders (swept 2026-09-30; repository visibility still to flip for anonymous clone):

- All `<OWNER>` URLs now point at `https://github.com/LucaEcosystem/luca-code` (`scripts/install.sh`, `scripts/install.ps1`, `docs/install.md`, `editors/vscode/*`, `editors/vscode/package.json` repository URL).
- Extension publisher is set (`luca-ecosystem` in `editors/vscode/package.json`).
- License decided and added: custom proprietary free-use `LICENSE` at the repo root (free use incl. commercial; no modification, redistribution, or resale; programs users write are theirs). Extension manifest points at it.

Packaging-version constraint (reported, not changed): the product identity stays `1.0 Beta` (`Cargo.toml` 1.0.0 + `PRODUCT_VERSION = "1.0 Beta"`, CLI prints `luca 1.0 Beta`). The VS Code package version is semver `1.0.x` because vsce/Marketplace require numeric `major.minor.patch` and reject re-uploading an existing version — hence `1.0.1` for the placeholder-sweep republish with zero behavior change; the Beta identity is carried in the extension `displayName`/`description` and produced artifact `luca-code-1.0.1.vsix`.

Manual launch checklist (before public release/tag or any publish):

1. Flip repository visibility (anonymous clone still needs this), choose-license done, placeholders swept.
2. Push; confirm all three CI jobs green (first real Windows + Linux evidence).
3. In VS Code with the packaged `.vsix`: confirm highlighting on `examples/errors.lucc`, run-button flow, an error diagnostic jump, and the missing-CLI guidance.
4. Publish the extension to the Marketplace and tag the release only after 1–3.

## M1a validation API + `luca check` (approved SDK step, no semantics change)

Read-only validation without executing application code. `run` behavior, language semantics, and diagnostic wording are unchanged.

What was built:

- Library API (`src/check.rs`, re-exported from `src/lib.rs`): `check_source(&str) -> Vec<CheckDiagnostic>` (lex + parse only), `check_file(&Path, &[PathBuf]) -> Result<Vec<CheckDiagnostic>, CheckFailure>` (plus recursive `import` resolve/read/lex/parse with builtin precedence, once-only, `Circular import` guard, depth/file caps), `CheckDiagnostic{message,line,column}` with `Error at line:column` Display, `diagnostics_to_json` stable `[{"message","line","column"}]` with hand-rolled escaping (zero new dependencies).
- CLI (`src/main.rs`): `luca check <program.lucc> [--format human|json]` (`--format=json` also accepted). Exit 0 valid (JSON prints `[]`), 1 validation errors, 2 usage/internal failure (`luca check:` stderr prefix, never JSON). `search_paths` match `luca run` (program-dir + CWD). `run` path untouched.
- Guarantees: check never calls `execute`, `stdlib::call`, `stdin().read_line`, `println` (application output), or `process::exit` on behalf of source code. Imports are resolved and parsed only; module top-levels never run.
- Tests (`tests/check.rs`, 13 tests + 2 `check.rs` unit tests): valid source/file, syntax and lexer errors with locations, missing/nested/circular imports, `In module` wrapping, builtin pass-through, side-effect pass-through (`ask`/`print`/`File.write`/`System.exit` validate clean and create nothing), internal-vs-validation distinction, JSON stability/escaping.
- Limits: dynamic checks still require execution (call arity on runtime values, division by zero on computed values, `ask` conversion, declaration type checks on runtime values, import-name scope collisions). `check` covers syntax plus the import graph only.

## M1b-1 Platform interface (additive foundation, no behavior change)

The `Platform` interface and its implementations have been introduced. Runtime call sites have not yet migrated. This milestone makes no runtime behavior changes.

What was built:

- `src/platform.rs` (new, re-exported from `src/lib.rs`): `PlatformErrorKind` (`NotFound`/`PermissionDenied`/`InvalidInput`/`Unsupported`/`Other`), `PlatformError{kind,detail}` with `from_io` (kind mapping + complete `io::Error` Display preserved in `detail`, including OS error information) and `unsupported` constructors, the `Platform` trait (file read/write/append/delete/existence, runtime output/warnings sinks, input line, environment variables, OS/architecture, working directory), `NativePlatform` (delegates to the exact `std::` calls the interpreter uses today), and `FakePlatform` (per-instance scripted files/errors, environment, input queue, captured output/warnings, OS/arch and working-directory overrides; no global state).
- Every trait method documents the historical call it mirrors and the compatibility contract later migrations must preserve (frozen diagnostic text via `detail`, lossy `false`-on-error existence probes, infallible output/warning sinks, unsorted environment passthrough, unchanged silent working-directory fallbacks).
- Tests (7 unit tests in `src/platform.rs`): `from_io` kind mapping and detail preservation (including `os error 2`), native OS/architecture/working-directory parity, native missing-file `NotFound`, fake scripted files/failures/input/environment/output/identity, and two-instance isolation.
- Limits: the interface is unused — `interpreter.rs`, `stdlib.rs`, `main.rs`, and `check.rs` are untouched. Legacy terminating APIs (`run`, `run_collect`, `run_with_input`) are unchanged for now; the planned safe outcome API and their migration remain future work (M1b-3). Platform independence and safe embedding are not complete.

## M1b-2 Output/warning migration (no semantics change)

Interpreter runtime output and warnings now flow through the `Platform` abstraction. Existing public execution and `System.exit` behavior remain unchanged.

What was built:

- `FakePlatform` (`src/platform.rs`): state moved into a private shared owner so `Clone` shares observations between clones while independently created instances stay isolated (`Rc`-based, standard library only). Public method signatures unchanged; existing unit tests unmodified, plus a clone-sharing test.
- `src/interpreter.rs`: new `platform: Box<dyn Platform>` field (no generics), defaulting to `NativePlatform` in `new()` so every existing constructor chain (`run`, `run_with_input`, CLI, tests, `Default`) behaves exactly as before, plus a `with_platform` constructor for injection. `warn()` forwards to `emit_warning`; the three output-flush loops in `run`/`run_collect` forward to `emit_output`. `run_collect()` successful execution still returns buffered output without emitting it. Every `System.exit` check and `process::exit` call is untouched, as is `warn_at` with its remaining `stdlib.rs` `File.delete` caller.
- Tests (`tests/output.rs`, 13 tests): print/`ask` ordering, unreachable-code locations, out-of-range/missing-key reads with `none` results, literal and `.add` duplicate warnings, remove-miss warnings, the `run_collect` buffer-without-emit contract, all with exact line/column/message expectations verified against the implementation.
- Limits: input, filesystem, module resolution, environment, OS/architecture, and working-directory handling still call `std::` directly. The `File.delete` warning migrates with the filesystem natives (M1b-5). Luca Code is not yet platform-independent or safe for embedding; the outcome API remains future work (M1b-3).

## M1b-3 Non-terminating outcome API + CLI ownership (no semantics change)

Library consumers can now execute programs containing `System.exit(N)` without the library terminating the host process. Language semantics, diagnostic wording, and the exit marker mechanism are unchanged.

What was built:

- `RunOutcome` (`src/interpreter.rs`, re-exported from `src/lib.rs`): `Completed(Vec<String>)` for successful runs, `Exited { code, output }` for `System.exit` requests. `run_outcome(&mut self, source)` shares the existing lex/parse/execute path (all propagation sites untouched), drains the output buffer into the owned outcome, and never calls `process::exit` or emits captured output. Ordinary errors return `Err` with buffered output discarded, exactly as before.
- Legacy `run`, `run_collect`, and `lib::run`/`run_with_input` keep their signatures and observable behavior, reimplemented as thin wrappers over the outcome core, with doc comments marking them transitional (host-terminating on valid exit; new embedding code must use `run_outcome`). No deprecation attributes, so all callers stay warning-free.
- CLI (`src/main.rs`, run path only): translates the outcome itself — prints captured output, exits with the requested code, reports ordinary errors to stderr with exit 1. `luca check` untouched.
- Tests (`tests/exit.rs`, 6 tests): in-process `Exited` with code and pre-exit buffer, runner alive, no direct emission; `Completed`; ordinary-error discard; exit in functions, module init (identity preserved), and `try`/`finally` (finally runs, capture skipped). (`tests/cli.rs`, 10 tests): subprocess runs via `CARGO_BIN_EXE_luca` in RAII-cleaned temp dirs asserting exact status/stdout/stderr — success, exit-after-output, exit in function/module/finally, ordinary-error discard with exact stderr bytes, usage errors, and `check` valid/invalid/missing coverage (closes M1a F1).
- Limits: input, filesystem, modules, environment, OS/architecture, and working-directory handling still call `std::` directly (M1b-4/M1b-5). Embedding remains transitional until callers migrate off the legacy APIs.

## M1b-4 Input migration (no semantics change)

Interpreter `ask` input now flows through the `Platform` abstraction. Prompt buffering, output ordering, source locations, conversion behavior, and all diagnostics are unchanged.

What was built:

- `read_input_line` (`src/interpreter.rs`): the direct stdin branch now delegates to `Platform::read_line()`. The scripted `with_input` path is untouched and still takes precedence. End of input maps to the frozen `No more input available` diagnostic (anchored on the documented platform detail string); every other input failure maps to the frozen `Failed to read input: {detail}` wrapper, with `detail` rendering byte-identically to the former `io::Error` text. The now-unused `BufRead` import was removed.
- Tests (`tests/output.rs`, 5 new): platform-provided normal line with prompt ordering, empty-input EOF with exact `1:24` diagnostic, generic and `PermissionDenied` input errors with exact `1:24` wrapper diagnostics (proving EOF is never conflated), and prompt-before-result ordering with type conversion. Error paths additionally pin the pre-existing discard rule (buffered prompt not emitted).
- Limits: filesystem, module resolution, environment, OS/architecture, and working-directory handling still call `std::` directly (M1b-5). The `Platform` public contract is unchanged.

## M1b-5 Filesystem + module loading (no semantics change)

Runtime file operations and module loading now flow through the `Platform` abstraction, enabling fully deterministic tests. Language semantics, diagnostic wording, and warning behavior are unchanged.

What was built:

- **B1 — file detection + module loading.** `Platform::is_file()` added (native `Path::is_file` semantics; fakes report in-memory files only), plus `FakePlatform::add_dir()` so scripted directories satisfy `path_exists` without ever satisfying `is_file`. `resolve_module_file` probes candidates via `is_file` (a directory named `X.lucc` still resolves to `missing module`), and `import_module` reads sources via `read_file` with identical `Could not read module` mapping. Tests (`tests/modules.rs`, 12 tests): successful/repeated imports, missing/unreadable modules with locations, directory-`.lucc` rejection, circular/nested imports, search-path ordering, `In module` wrapping, builtin precedence — all against scripted files, no host I/O.
- **B2 — standard-library operations.** `File.read/write/append/exists/delete`, `System.env`, and `System.os/arch` now use the injected platform, threaded through `call_native_function → stdlib::call` and `builtin_member_value` (each with exactly one caller). Frozen messages render byte-identically via error `detail`; environment sorting stays in the caller; `File.delete`-missing still warns at the statement location and returns `none`, now via `emit_warning`. The last `warn_at` caller is gone, so the helper was removed. Tests (`tests/files.rs`, 8 tests): round-trips, append-create, existence, delete + missing warning capture, scripted error mapping for all four mutating reads, sorted env display, OS/arch overrides, pure-dispatch sanity, outcome carriage.
- Compatibility note: one M1b-3 test mixed an injected (empty) fake with real temp-dir module files; it now scripts its module into the fake instead — same exit-identity assertion, fully hermetic. All pre-existing temp-file tests stay green unmodified as native-parity evidence.
- Limits: constructor/CLI working-directory handling, `check.rs` validation I/O, and `System.args` (already injected data) intentionally still use `std::` directly. No new dependencies; Luca UI untouched.

## M2 Embedding-safe SDK surface

Pluggable validation, a legacy-API compatibility decision, and a host contract — no language-semantic changes.

What was built:

- **B1 — pluggable validation filesystem.** Additive `check_file_with_platform(entry, search_paths, platform)`; `check_file` delegates with `NativePlatform`, behavior unchanged. Import traversal (`resolve_module_file`, source reads) routes through the injected filesystem. Unsaved buffers without a path remain syntax-only via `check_source` (documented limitation, not built). Tests (`tests/check_platform.rs`, 11 tests): scripted lookup, missing/nested/circular imports, `.lucc` directories rejected, unreadable-module detail, `In module` wrapping, entry-failure distinction, builtin precedence, native-parity and JSON shape.
- **B2 — legacy termination compatibility (assessment only, no code change).** `#[deprecated]` is deliberately withheld: this repo's CI and Luca UI's CI both deny warnings, so deprecation would break builds here (dozens of test call sites) and in Luca UI (`seam.rs` uses `run`/`run_with_input`), which must not be edited from this repo. Legacy APIs keep exact behavior with transitional-debt docs; `run_outcome` is the documented embedding API (also referenced from `docs/host-requirements.md`). Recorded as deferred work pending coordinated migration.
- **B3 — host contract + target spike.** New `docs/host-requirements.md`: per-capability host obligations, synchronous/input/output/module-resolution contracts, `run_outcome`-vs-legacy distinction, per-host (desktop/WASM/Android) requirements, and implemented-vs-future inventory. WASM experiment 2026-10-16: `cargo check --target wasm32-unknown-unknown` fails with `E0463` (target std not installed; only `aarch64-apple-darwin` present); installing it requires downloads and was out of scope. No WASM support claimed.
- Limits: no language, diagnostic, CLI-behavior, or public-signature changes; no dependencies; nothing committed.

## M3 WASM host spike (experimental, no semantics change)

First non-native host: `luca-code` compiles for `wasm32-unknown-unknown`
with zero source changes, and a minimal host executes scripted programs
in Node through a narrow C ABI. Language semantics and frozen diagnostics
are unchanged.

What was built:

- **B1 — compile gate.** `cargo check --target wasm32-unknown-unknown --lib`
  passes with no code changes (local toolchain extended with the target
  std; no project configuration changed). `luca-code` keeps its `rlib`
  shape; no `cdylib` was added to it.
- **B2 — host + boundary (`wasm-host/`, crate `luca-wasm-host`,
  `cdylib`, zero new dependencies).** Consumes `luca-code` public API
  only. Opaque-handle hosts (`Box` pointer as `usize`, no globals) with
  an in-memory `Platform` (scripted files/dirs, denied-vs-queued input
  via `Unsupported`, captured output/warnings, `web`/`wasm32` labels,
  virtual root `/`). Narrow ABI: alloc/free with exact-size pairing,
  setup calls returning `0`/`-1`/`-2`/`-3`, `luca_run`/`luca_check`
  returning length-explicit JSON documents parsed by built-in JSON
  support (hand-rolled output escaping only; no custom parser, no
  request JSON). All argument validation is explicit; panics cannot
  unwind across the boundary by construction.
- **B3 — Node integration (`wasm-host/smoke.mjs`, 13 tests)** against the
  real artifact: execution ordering, exit-as-data with a live Node,
  virtual imports + missing/dir rejection, queued and denied input,
  per-call output/warning isolation, invalid-handle/UTF-8 robustness,
  alloc/run/free cycles, and `check` over scripted graphs.
- **B4 — CI + docs.** New `wasm` CI job (target install, lib check, host
  build + deny-warnings lint, Node smoke on ubuntu); `host-requirements.md`
  updated with the actual contract. No browser UI, no npm package.
- Limits: experimental spike only — no supported WASM product status, no
  async-input bridge beyond queue-or-deny, no Android/LSP work.

## Runtime host-behavior notes (specified, not deferred)

- **Working directory.** `Interpreter::new()` (`src/interpreter.rs:121`)
  seeds default module search paths with the process working directory,
  falling back to `"."` when the directory cannot be read; the fallback
  is silent by design. The CLI (`src/main.rs`) builds search paths from
  the program's own directory first, then the working directory (skipped
  silently when unreadable, deduplicated). Library embedders that inject
  explicit search paths via `with_search_paths` bypass both behaviors.
- **Iteration and fuel.** Function recursion is bounded (`recursion_limit`
  64, `src/interpreter.rs:128`; exceeding it is a capturable
  `Recursion limit exceeded` error). Loop steps are not bounded: `while`
  loops and large `for N times` counts run until completion, and deep
  non-cyclic import chains have no file-count cap in the interpreter
  (`luca check` caps its own traversal at 100 files / depth 32,
  `src/check.rs:31-35`, because validation must terminate on adversarial
  input). Numeric overflow is always a proper error (`Integer overflow`,
  `Decimal result is out of range`), never a wrap or panic. Hosts that
  need execution bounds (timeouts, step budgets) must enforce them
  outside the interpreter.
