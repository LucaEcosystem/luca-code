# Luca Code — 1.0 Beta

Luca Code 1.0 Beta is the feature-complete Beta: input (`ask`), the `Math`/`Text`/`File`/`System` standard library, CLI integration, and custom-error raising (`raise`) with caught-error inspection, keeping the Rust reference implementation (lexer → parser → AST → type checking → interpreter) before a native backend.

## Install and first run

```sh
bash scripts/install.sh   # macOS/Linux (.\scripts\install.ps1 on Windows)
luca --version            # prints: luca 1.0 Beta
```

Beginner guides: [install / upgrade / uninstall](docs/install.md), [your first program](docs/first-program.md), [VS Code setup](docs/vscode.md), [troubleshooting](docs/troubleshooting.md). The VS Code extension lives in `editors/vscode/`.

## Run an example

```sh
cargo run -- examples/hello.lucc
cargo run -- examples/arithmetic.lucc
cargo run -- examples/inference.lucc
cargo run -- examples/comparison.lucc
cargo run -- examples/conditional.lucc
cargo run -- examples/functions.lucc
cargo run -- examples/loops.lucc
cargo run -- examples/collections.lucc
cargo run -- examples/modules/main.lucc
cargo run -- examples/errors.lucc
cargo run -- examples/integration.lucc
printf 'Luca\n16\n' | cargo run -- examples/ask.lucc
cargo run -- examples/stdlib.lucc
cargo run -- examples/stdlib.lucc extra args here
cargo run -- check examples/hello.lucc
cargo run -- check examples/hello.lucc --format json
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

`luca check <file.lucc> [--format human|json]` validates a program without executing it: syntax plus the transitive `import` graph (missing/circular/unreadable modules, syntax errors inside imports). It never runs application code — no prompts, no `print` output, no `File` mutations, no `System.exit` termination. Exit codes: 0 valid, 1 validation errors, 2 usage/internal failure (`luca check:` prefix, never JSON).

The CLI accepts Luca source files with the `.lucc` extension (`cargo run -- <file.lucc>` enforces the extension). Module imports (`import Name` → `Name.lucc`) resolve from the program directory first, then the working directory.

## Project structure

- `src/lexer.rs` — source text to tokens (`--`, `[[ ... ]]`, strings, interpolation, `Indent`/`Dedent` for `:` blocks, `func`/`return`/`for`/`while`/`times`/`repeat`/`from`/`stop`/`skip`, `list`/`tuple`/`set`/`dict`, `import`, `try`/`capture`/`finally`/`error`/`raise`, `ask`, `[`/`]`/`{`/`}`)
- `src/parser.rs` — tokens to syntax tree (declarations incl. `def list/tuple/set/dict` and `def error`, assignments, `print`, `ask`, comparisons/logic, `if`/`else` blocks, `def func`/`return`/`call`, `for`/`while` loops, collection literals/indexing/methods, `change name[i]`, `import Name`, `Module.member`/`Module.func()`, `try to`/`capture error`/`finally`, `raise Name(...)`)
- `src/ast.rs` — language syntax tree (`Block`, `Param`, `DictEntryExpr`, `ContainsMode`, `LengthArg`, `Statement::DeclareCollection`/`ChangeIndex`/`CollectionAdd`/`CollectionRemove`/`Import`/`MemberCall`/`Try`/`ErrorDef`/`Raise` + `RaiseArg`, `Expr::List`/`Tuple`/`Set`/`Dict`/`Index`/`Contains`/`Length`/`Member`/`MemberCall`/`Ask`)
- `src/types.rs` — types (`int`, `dec` exact, `str`, `bool`, `uni`, `none`, `CollectionKind`, `DictEntry`, `values_equal` content equality) and runtime values (shared collections)
- `src/interpreter.rs` — evaluation and output (scoped `scopes`/`func_scopes`/`error_scopes`, `call_depth:64`, `loop_depth`/`iterating`, `BindingType::Scalar`/`Collection`, module cache `modules`/`loading`/`search_paths`, scripted/stdin `input_lines`, CLI `cli_args`, `import_module`/`call_module_function`/`call_native_function`, `execute_try`/`execute_error_def`/`exec_raise` with `error_value` on `LucaError`, warnings via `Warning at line:column`)
- `src/stdlib.rs` — built-in `Math`/`Text`/`File`/`System` (exact `dec` math, strict types, `none` rejection)
- `src/check.rs` — read-only validation (`check_source`/`check_file`, `CheckDiagnostic` with `Error at line:column` Display, stable `diagnostics_to_json`); resolves imports but never executes, prompts, prints, mutates files, or exits
- `src/main.rs` — CLI (program-directory + working-directory module search paths, full argv for `System.args`, `check` subcommand for execution-free validation with human/JSON diagnostics)
- `examples/` — runnable Luca programs (`hello.lucc`, `arithmetic.lucc`, `inference.lucc`, `comparison.lucc`, `conditional.lucc`, `functions.lucc`, `loops.lucc`, `collections.lucc`, `modules/main.lucc` + `modules/helpers.lucc`, `errors.lucc`, `integration.lucc`, `ask.lucc`, `stdlib.lucc`)
- `tests/` — interpreter behavior checks (39 tests incl. `ask`, stdlib, cross-feature integration, `10/4→2.5`, custom-error raising/inspection), `check` validation tests, platform/output/exit/module/file CLI tests, and Node integration tests for the experimental WASM host (129 Rust tests + 13 Node tests total)
- `docs/language-spec.md` — 0.1–1.0 implemented language and boundary
- `docs/locked-language-reference.md` — full locked reference (authoritative)
- `docs/implementation-status.md` — checklist per release; 1.0 Beta verified 2026-09-26

1.0 Beta closes the locked set: custom-error raising (`raise Name(...)` with required parentheses, field overrides, and caught-error inspection including `line`/`column`) completes the 0.9 foundation above. See `docs/implementation-status.md` for the per-release checklist (verified 2026-09-26).
