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
