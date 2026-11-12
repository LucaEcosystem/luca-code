# Changelog

All notable user-visible changes to Luca Code. The product identity **Luca Code 1.0 Beta** is the current feature-complete Beta; machine-facing "package versions" are semantic `1.0.x` numbers and appear in `luca --version` and in packaged artifacts.

Starting with the 1.0.3 package, package versions follow this scheme: within the 1.0 generation, documentation/publishing/defect releases bump the package version while the language stays at its Beta content.

## Unreleased

No changes yet.

## 1.0.3 Beta (2026-11-12) — documentation, publishing, and audit

- New documentation set: a [guided tour](docs/learn/tour/index.md) from "Hello" to custom errors, a complete [language reference](docs/reference/language-reference.md), [standard-library reference](docs/reference/stdlib.md), [CLI reference](docs/reference/cli.md), and a [glossary](docs/reference/glossary.md).
- Guided install-and-first-run path refreshed for public use: [install](docs/learn/install.md), [first program](docs/learn/first-program.md), [VS Code setup](docs/learn/vscode.md), [troubleshooting](docs/learn/troubleshooting.md), and a runnable [examples gallery](docs/learn/examples-gallery.md).
- `luca --version` now prints `luca 1.0.3 Beta`; the VS Code extension is packaged as `1.0.3`.

### Correctness fixes

- **Comment-only lines no longer affect indentation.** `--` and `[[ … ]]`
  comments may now be indented to any depth inside a block without producing
  spurious `Indent`/`Dedent` errors. A block containing only comments is still
  reported as empty.
- **`int.length` is safe for every 64-bit value**, including
  `-9223372036854775808` (19 digits); it previously panicked on `i64::MIN`.
- **Module constant collections are isolated from aliases.** Reading a
  collection-valued `def const` from a module now returns a deep copy, matching
  the const semantics of ordinary bindings.
- **`uni` is genuinely universal again.** A `uni` binding may hold a list,
  tuple, set, or dictionary; `.add`, `.remove`, `.clear()`, `change x[i] = v`,
  and `.length` dispatch on the runtime value, and the binding may later switch
  back to a scalar.
- **Decimal extremes no longer panic.** Display, division, and remainder handle
  `i128::MIN`/`i128::MAX` using checked/unsigned arithmetic. Non-terminating
  division is documented and tested as truncated at 28 fractional digits.
- **Pathological nesting reports a diagnostic.** Groups and blocks nest at most
  32 levels (`Block/Expression nesting is too deep`) instead of overflowing the
  parser stack on thousands of nested parentheses.

### Tests and tooling

- Regression suites for the fixes above, plus lexer, parser, decimal, module,
  documentation-link, metadata, and end-to-end example tests. The suite now
  runs **179 tests** across **17 test targets**.
- New examples: `text.lucc`, `math.lucc`, `files.lucc`, `utility.lucc`, with
  `examples/README.md` and expected output in the gallery.
- VS Code extension: new **Luca: Check Luca File** command (`luca check
  --format json` diagnostics without executing), unit tests for the JSON
  contract, `bugs`/`homepage` metadata, a packaged `CHANGELOG.md`, and a copy of
  the repository license.
- Documentation: the language reference now documents comment indentation, the
  28-digit decimal cap, `uni` collections, typed-return behavior, and nesting
  limits; `docs/learn/known-limitations.md` records intentional boundaries;
  troubleshooting no longer describes the repository as private or mentions a
  nonexistent `luca run` subcommand.
- CI additionally builds documentation (`cargo doc --no-deps`) and smoke-runs
  the new deterministic examples.


## 1.0 Beta (verified 2026-09-26) — the feature-complete Beta

Everything listed in 0.9 below plus:

- **Custom errors end to end**: `raise Name(field = value, …)` raises a declared error with per-raise field overrides (required parentheses, optional overrides, duplicate overrides warn and keep the first, mismatched types reject) and **`capture error :` binds the caught error as a dictionary** — your declared fields plus the `line`/`column` where the error occurred; runtime errors carry `code = none` and a `message`.
- The Beta is declared complete: the whole locked feature set is implemented and verified (129 automated tests across the language, standard library, CLI, check mode, and the experimentally supported WASM host spike).

## 0.9 Alpha (2026-09-26) — input and the standard library

- **`ask "question"`** reads one line of typed console input: `str` (raw line), `int`/`dec` (parsed numbers), `bool` (exactly `true`/`false`). Invalid conversions and exhausted input are ordinary errors that `try to` can capture.
- **Standard library**: `Math` (exact `sqrt`/`pow`/`abs`/`round`/`floor`/`ceil`/`min`/`max`), `Text` (`upper`/`lower`/`trim`/`replace`/`split`/`join`), `File` (`read`/`write`/`append`/`exists`/`delete`), `System` (`os`/`arch` properties, `args()`, `env()`, `exit(code)`) — strict argument types, no implicit conversion, `none` rejected.
- **CLI integration**: full command-line visibility via `System.args()`, `.lucc` extension enforcement, `System.exit(code)` termination with flushed output and running `finally` blocks (uncapturable by `capture error`).
- Distribution groundwork: `scripts/install.sh` (macOS/Linux) and `scripts/install.ps1` (Windows), a packaged VS Code extension (syntax highlighting, snippets, one-key run, editor diagnostics), CI on all three platforms, and the first beginner documentation set.

## 0.8 Alpha (2026-09-25) — stabilization

- **Unreachable code warnings** after unconditional `return`/`stop` in a block — a `Warning`, execution continues.
- **Uniform collection sharing**: collections are reference values everywhere (assignments, parameters, returned values); `const` bindings deep-copy on read so constant contents can never change.
- **Interpolation diagnostics** point at the string literal; **`return` accepts collection literals**; warnings and diagnostics hardened across all features.

## 0.7 Alpha (2026-09-25) — error handling

- **`try to :` / `capture error :` / `finally :`**: capture runtime failures; `finally` always runs; signals (`return`/`stop`/`skip`) propagate through both blocks orderly.
- **Custom error definitions**: `def error Name :` declaring at least `code` and `message` fields (raising and inspection arrived in the 1.0 Beta above).

## 0.6 Alpha (2026-09-25) — modules

- **`import Name`** reads `Name.lucc` from the program directory (then the working directory); module members and functions are reachable as `Module.member` / `Module.func(...)`; each module has its own scope, initializes once, exports every top-level definition, and rejects circular imports.

## 0.5 Alpha (2026-09-25) — collections

- **Lists, tuples, sets, dictionaries** with 1-based indexing, `.add`/`.remove`/`.clear` statements, `.contains`/`.length` expressions, element reassignment via `change name[i] = v`, content-based equality, and kind-aware warnings (missing reads yield `none` and warn, never crash).

## 0.4 Alpha (2026-09-24) — loops

- `for N times repeat :`, `while condition repeat :`, and — arriving in 0.5 — `for item from collection repeat :`; `stop` exits the innermost loop, `skip` advances the iteration.

## 0.3 Alpha (2026-09-24) — functions

- `def func name(params) (return_type):` with optional parameter types, defaults, and return annotations; `return`/`return value`; recursion (limit 64) and closures that may read and modify captured variables.

## 0.2 Alpha (2026-09-24) — conditionals and blocks

- `if condition then :` / `else if` / `else :` with `:` + indentation blocks; booleans required as conditions (no truthiness); short-circuit `and`/`or` and strict `not`.

## 0.1 Alpha (2026-09-24) — the foundation

- Lexer → parser → AST → type checking → interpreter pipeline with actionable `Error at line:column: message` diagnostics.
- Typed and inferred variable/constant declarations, assignment, `print` with `{name}` interpolation, exact decimal arithmetic with the from-the-start rule **`10 / 4` is `2.5` (a `dec`)**, strings, comparisons, `and`/`or`/`not`, multiline comments, plus the first runnable examples and test suite.
