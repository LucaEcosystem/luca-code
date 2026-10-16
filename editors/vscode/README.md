# Luca Code for VS Code (1.0 Beta)

Official language support for [Luca Code](https://github.com/LucaEcosystem/luca-code) 1.0 Beta: `.lucc` syntax highlighting, snippets, and one-key run.

## Setup

1. Install the `luca` CLI (see `docs/install.md` in the Luca Code repository).
2. Install this extension (search "Luca Code" in the Extensions view after it is published, or package it locally with `npx @vscode/vsce package`).
3. Open any `.lucc` file. If the CLI is not on your `PATH`, set the `luca.cliPath` setting to its absolute path.

## Use

- **Luca: New Luca File** — open a starter `.lucc` program.
- **Luca: Run Luca File** — save and run the active file (editor title-bar button when a `.lucc` file is open). Program output appears in **Output > Luca Code**; `Error at line:column:` and `Warning at line:column:` become editor diagnostics.

## Develop

- `npm test` runs the unit tests (no dependencies to install).
- Snippets live in `snippets/luca.json`; the TextMate grammar in `syntaxes/luca.tmLanguage.json`.
