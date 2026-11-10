# Luca Code

Luca Code is a small, friendly, general-purpose programming language: plain sentences for rules, exact decimal math (`10 / 4` is `2.5`), and programs that run from top to bottom with no ceremony. This repository contains the language's Rust reference implementation (lexer → parser → AST → type checking → interpreter) and its tooling.

Current release: **Luca Code 1.0 Beta** — a feature-complete Beta of every planned behavior, undergoing validation toward 1.0. `luca --version` reports the package version: **`luca 1.0.3 Beta`**.

## Get started in one minute

Install the `luca` command ([full instructions](docs/learn/install.md)):

```sh
git clone --depth 1 https://github.com/LucaEcosystem/luca-code.git
cd luca-code
bash scripts/install.sh            # Windows: .\scripts\install.ps1 in PowerShell
luca --version                     # prints: luca 1.0.3 Beta
```

Write `hello.lucc`:

```luca
def const name (str) = "Luca"
print("Hello, {name}!")
```

Run it:

```sh
luca hello.lucc
# Hello, Luca!
```

## Learn and use

**Learning** — read in this order, nothing presupposed beyond a terminal:

1. [Install](docs/learn/install.md) — get `luca` running on macOS, Linux, or Windows
2. [First program](docs/learn/first-program.md) — two minutes, start to first output
3. [The guided tour](docs/learn/tour/index.md) — twelve short chapters from values to custom errors, every snippet runnable
4. [Examples gallery](docs/learn/examples-gallery.md) — one runnable program per feature, with its real output

**Reference** — for looking things up:

- [Language reference](docs/reference/language-reference.md) — the complete, verified rule set
- [Standard library](docs/reference/stdlib.md) — every `Math`, `Text`, `File`, `System` function
- [CLI reference](docs/reference/cli.md) — `luca`/`luca check`/`--help`/`--version`, exit codes
- [Glossary](docs/reference/glossary.md) — plain-language definitions
- [Changelog](CHANGELOG.md) — what shipped in each release

**In your editor**: the official [VS Code extension](docs/learn/vscode.md) (Marketplace: search **Luca Code**) gives `.lucc` highlighting, snippets, one-key run, and syntax/import validation. First-run problems are collected in [troubleshooting](docs/learn/troubleshooting.md).

## A taste of the language

```luca
def list names (str) = ["Luca", "Alex"]
names.add("Sam")
for name from names repeat :
    print("Hi, {name}")

def func total(prices (int)) (int):
    return prices * 2 + 1

print(total(10))
```

```text
Hi, Luca
Hi, Alex
Hi, Sam
21
```

Error messages read like sentences, with the exact spot: `Error at 2:5: 'visits' is not declared`.

## Try an example

The repository ships runnable programs per feature — arithmetic, conditionals, functions, loops, collections, modules, errors, input, and the standard library:

```sh
luca examples/errors.lucc
printf 'Luca\n16\n' | luca examples/ask.lucc
```

Full list and expected output: [examples gallery](docs/learn/examples-gallery.md).

## Status

Luca Code 1.0 Beta implements the complete 1.0 language: types and exact decimals, variables and constants, control flow, functions and closures, the four collection kinds, modules, error handling with custom `raise`-able errors, typed console input (`ask`), and the `Math`/`Text`/`File`/`System` library. Everything is verified by 179 automated tests across the language, CLI, examples, and embedding layers. Validation and stabilization continue toward the stable 1.0; see the [changelog](CHANGELOG.md) for what shipped when and [known limitations](docs/learn/known-limitations.md) for what is intentionally out of scope.

## Developing this repository

From a checkout: `cargo test` runs the suite, `cargo run -- examples/hello.lucc` runs an example, and `cargo clippy --all-targets --all-features -- -D warnings` is the lint gate. The VS Code extension lives in [`editors/vscode/`](editors/vscode/README.md) (`npm test` there). See [CONTRIBUTING.md](CONTRIBUTING.md) for the full workflow and [SECURITY.md](SECURITY.md) for reporting vulnerabilities. LICENSE: the interpreter, CLI, extension, examples, and docs are free to **use**, including commercially; modifying or redistributing the Software itself requires prior written permission — see [LICENSE](LICENSE). Programs you write in Luca Code are yours.
