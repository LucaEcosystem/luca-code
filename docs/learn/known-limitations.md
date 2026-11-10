# Known limitations

Luca Code 1.0 Beta implements the complete 1.0 language described in the
[language reference](../reference/language-reference.md). This page lists what
is deliberately **not** implemented, or is bounded by an explicit safety limit,
so the documentation never promises more than the binary does.
If something here blocks you, open an issue with the smallest program that
shows the problem. “Missing feature” and “defect” are different: this page is
about the former.

## Implementation shape

- **Interpreter only.** Programs run in the Rust reference interpreter
  (`lexer → parser → AST → checker/interpreter`). There is no LLVM backend, JIT,
  or native compiler, and no plan to add one in the 1.0 stabilization line.
- **Zero external dependencies.** The crate builds with the Rust standard
  library alone; there is no package manager, dependency resolver, or lockfile
  beyond the toolchain's own `Cargo.lock`.
- **Built-ins are compiled in.** `Math`, `Text`, `File`, and `System` live in
  the interpreter. The `std/` and `compiler/` folders in a local checkout are
  empty placeholders and are not part of the build.

## Language boundaries

- **No casts or conversion functions.** `change var name (type) = value` is the
  only way to change a stored type; there are no `int()`, `str()`, or `dec()`
  calls and no implicit coercion.
- **No overloading or keyword arguments.** One function per name per scope;
  calls are positional. The `field = value` form belongs to `raise`.
- **No `let`, no ranges, no exponent operator.** Use `for N times`,
  `for item from collection`, and `Math.pow`.
- **Single-line string literals.** `\n` is an escape inside a quoted string;
  raw newlines are rejected. Interpolation accepts a variable name only
  (`{name}`), not arbitrary expressions.
- **ASCII identifiers.** Names use ASCII letters, digits, and `_` (not starting
  with a digit). Unicode is fully supported inside string values; string
  `.length` counts Unicode scalar values, not grapheme clusters.
- **Fixed 28-digit decimal scale.** Literals with more than 28 fractional
  digits are rejected. A non-terminating division is truncated (not rounded)
  at 28 fractional digits: `1 / 3` prints
  `0.3333333333333333333333333333`. This is exact decimal arithmetic, not
  arbitrary-precision rationals.
- **No truthiness.** Conditions must be `bool`; there is no implicit conversion
  from numbers or strings.

## Safety limits (not language rules)

- **Parser nesting: 32 levels.** Deeper blocks or grouped expressions report
  `Block nesting is too deep` / `Expression nesting is too deep` instead of
  exhausting the interpreter's stack.
- **Function recursion: 64 frames.** Deeper recursion reports
  `Recursion limit exceeded`; the error is capturable with `try to`.
- **`luca check` import walk: 100 modules, 32 levels.** Pathological import
  graphs report `Too many modules` / `Import depth exceeded`.

These limits are deliberately far above ordinary programs; they exist so
pathological input produces a diagnostic instead of aborting the process.

## Tooling boundaries

- **`luca check` is syntax + import-graph validation.** It does not statically
  type-check expressions, so errors such as `def var age (int) = "hi"` are
  reported only when the program runs. Dynamic errors (arity on runtime values,
  division by zero on computed values, invalid `ask` input) likewise need a run.
- **Module resolution is local.** `import Name` looks for `Name.lucc` in the
  program's own directory, then the working directory. There are no package
  registries, search-path flags, aliases, or remote imports.
- **No REPL, debugger, formatter, profiler, or language server.** The CLI runs
  and checks programs; the VS Code extension adds highlighting, snippets, run,
  check, and clickable diagnostics.
- **No async, threads, or networking.** The standard library is synchronous and
  local. Host processes and sockets are not exposed.
- **`File.*` is unsandboxed.** A program has the filesystem permissions of the
  user running it. There is no path allowlist or size cap; only run scripts you
  trust. `luca check` never touches the filesystem.
- **`System.exit` ends the host process.** Embedders should use
  `Interpreter::run_outcome`, which returns the exit request as data instead of
  terminating.

## Platforms and packaging

- CI runs the test suite on Linux, macOS, and Windows. The install scripts are
  smoke-tested there; untested shells or unusual permissions setups may still
  need the manual PATH steps in [troubleshooting](troubleshooting.md).
- The VS Code extension and the CLI share the `1.0.3` package version while the
  product identity stays **Luca Code 1.0 Beta**. A version bump does not imply
  a language change.

See also: [language reference](../reference/language-reference.md),
[CLI reference](../reference/cli.md), [troubleshooting](troubleshooting.md).
