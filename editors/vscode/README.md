# Luca Code for VS Code

Official language support for [Luca Code](https://github.com/LucaEcosystem/luca-code) — `.lucc` syntax highlighting, snippets, one-key run, and clickable diagnostics. This page mirrors the same documentation as the repository, so what you read on the Marketplace matches GitHub.

## Requirements

The extension drives the `luca` CLI. Install it first (a Rust-based build; no account needed):

```sh
git clone --depth 1 https://github.com/LucaEcosystem/luca-code.git
cd luca-code
bash scripts/install.sh            # Windows: .\scripts\install.ps1 in PowerShell
luca --version                     # prints: luca 1.0.3 Beta
```

Full per-OS instructions and PATH fixes: [install guide](https://github.com/LucaEcosystem/luca-code/blob/main/docs/learn/install.md).

## Features

- **Syntax highlighting** for `.lucc` files — keywords, strings, interpolation, comments, numbers.
- **Snippets** — type `try`, `error`, `raise`, `func`, `if`, `for`, `while`, `ask`, `import`, or `print`, then press Tab.
- **Run** — **Luca: Run Luca File** (`⇧⌘P` → "Luca", or the ▷ button in the editor's title bar) saves and runs the active file.
  - Default: integrated terminal (a shared "Luca" terminal) so output lands where you look and `ask` can read typed input.
  - Optional: Output > Luca Code channel (set `luca.runInTerminal` to `off`) — here `Error at line:column:` and `Warning at line:column:` lines turn into clickable squiggles in your editor.
- **New Luca File** — **Luca: New Luca File** opens a starter program.
- **Diagnostics** — error and warning lines from the run output map to inline editors markers, with the first error popping up at its location.

## Settings

| Setting | Default | Purpose |
|---|---|---|
| `luca.cliPath` | `luca` (found on `PATH`) | Absolute path to the `luca` binary when it is not installed on your `PATH` |
| `luca.runInTerminal` | `on` | Integrated terminal run; `off` runs in the Output channel with clickable diagnostics |

If the extension reports *"the 'luca' CLI was not found"*: put `luca` on your `PATH` (restart VS Code afterwards) or set `luca.cliPath` to the binary's absolute path. [Troubleshooting guide](https://github.com/LucaEcosystem/luca-code/blob/main/docs/learn/troubleshooting.md).

## Learning Luca Code

Start with the [first program](https://github.com/LucaEcosystem/luca-code/blob/main/docs/learn/first-program.md) and the [guided tour](https://github.com/LucaEcosystem/luca-code/blob/main/docs/learn/tour/index.md) — twelve short chapters with runnable snippets and their real output; the [language reference](https://github.com/LucaEcosystem/luca-code/blob/main/docs/reference/language-reference.md) and [standard library reference](https://github.com/LucaEcosystem/luca-code/blob/main/docs/reference/stdlib.md) hold the complete rule set.

## About Luca Code

Luca Code is a small, general-purpose language: top-to-bottom execution, exact decimal math, worded comparisons (`if n is > to 3 then :`), indented blocks, and plain-language error messages like `Error at 2:5: 'visits' is not declared`. Source and docs live in the [luca-code repository](https://github.com/LucaEcosystem/luca-code); releases are recorded in its [changelog](https://github.com/LucaEcosystem/luca-code/blob/main/CHANGELOG.md).

## Developing the extension

`npm test` runs the unit tests (no dependencies to install). Snippets live in `snippets/luca.json`, the TextMate grammar in `syntaxes/luca.tmLanguage.json`.
