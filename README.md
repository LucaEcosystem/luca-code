# Luca Code — 1.0 Beta

Luca Code 1.0 Beta is the feature-complete Beta: input (`ask`), the `Math`/`Text`/`File`/`System` standard library, CLI integration, and custom-error raising (`raise`) with caught-error inspection, keeping the Rust reference implementation (lexer → parser → AST → type checking → interpreter) before a native backend.

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
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

The CLI accepts Luca source files with the `.lucc` extension (`cargo run -- <file.lucc>` enforces the extension). Module imports (`import Name` → `Name.lucc`) resolve from the program directory first, then the working directory.

## Project structure

- `src/lexer.rs` — source text to tokens (`--`, `[[ ... ]]`, strings, interpolation, `Indent`/`Dedent` for `:` blocks, `func`/`return`/`for`/`while`/`times`/`repeat`/`from`/`stop`/`skip`, `list`/`tuple`/`set`/`dict`, `import`, `try`/`capture`/`finally`/`error`/`raise`, `ask`, `[`/`]`/`{`/`}`)
- `src/parser.rs` — tokens to syntax tree (declarations incl. `def list/tuple/set/dict` and `def error`, assignments, `print`, `ask`, comparisons/logic, `if`/`else` blocks, `def func`/`return`/`call`, `for`/`while` loops, collection literals/indexing/methods, `change name[i]`, `import Name`, `Module.member`/`Module.func()`, `try to`/`capture error`/`finally`, `raise Name(...)`)
- `src/ast.rs` — language syntax tree (`Block`, `Param`, `DictEntryExpr`, `ContainsMode`, `LengthArg`, `Statement::DeclareCollection`/`ChangeIndex`/`CollectionAdd`/`CollectionRemove`/`Import`/`MemberCall`/`Try`/`ErrorDef`/`Raise` + `RaiseArg`, `Expr::List`/`Tuple`/`Set`/`Dict`/`Index`/`Contains`/`Length`/`Member`/`MemberCall`/`Ask`)
- `src/types.rs` — types (`int`, `dec` exact, `str`, `bool`, `uni`, `none`, `CollectionKind`, `DictEntry`, `values_equal` content equality) and runtime values (shared collections)
- `src/interpreter.rs` — evaluation and output (scoped `scopes`/`func_scopes`/`error_scopes`, `call_depth:64`, `loop_depth`/`iterating`, `BindingType::Scalar`/`Collection`, module cache `modules`/`loading`/`search_paths`, scripted/stdin `input_lines`, CLI `cli_args`, `import_module`/`call_module_function`/`call_native_function`, `execute_try`/`execute_error_def`/`exec_raise` with `error_value` on `LucaError`, warnings via `Warning at line:column`)
- `src/stdlib.rs` — built-in `Math`/`Text`/`File`/`System` (exact `dec` math, strict types, `none` rejection)
- `src/main.rs` — CLI (program-directory + working-directory module search paths, full argv for `System.args`)
- `examples/` — runnable Luca programs (`hello.lucc`, `arithmetic.lucc`, `inference.lucc`, `comparison.lucc`, `conditional.lucc`, `functions.lucc`, `loops.lucc`, `collections.lucc`, `modules/main.lucc` + `modules/helpers.lucc`, `errors.lucc`, `integration.lucc`, `ask.lucc`, `stdlib.lucc`)
- `tests/` — interpreter behavior checks (39 tests incl. `ask`, stdlib, cross-feature integration, `10/4→2.5`, custom-error raising/inspection)
- `docs/language-spec.md` — 0.1–1.0 implemented language and boundary
- `docs/locked-language-reference.md` — full locked reference (authoritative)
- `docs/implementation-status.md` — checklist per release; 1.0 Beta verified 2026-09-26

0.9 completes the locked set: `ask "question"` with declared-type conversion (invalid input is a capturable Error), `Math`/`Text`/`File`/`System` built-ins through `import` (strict types, `none` rejection, exact `dec` math without `f64`), CLI integration (exit codes with flushed output, `System.args` wiring, `System.exit` unreachable warnings), plus all of 0.8's stabilized foundation with actionable `Error at line:column` diagnostics. The one open design item is custom-error raising/inspection, for which the reference defines no syntax.
