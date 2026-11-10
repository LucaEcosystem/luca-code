# Changelog — Luca Code for VS Code

The extension version follows the CLI package version; the product identity is
**Luca Code 1.0 Beta**.

## 1.0.3

- New command **Luca: Check Luca File**: saves the active `.lucc` file and runs
  `luca check --format json`, showing syntax and import-graph errors as inline
  diagnostics without executing the program.
- Runs now share one CLI-resolution path with check, so a missing `luca`
  binary produces the same install/settings prompt in both commands.
- `parseCheckJson()` and `buildCheckArgs()` are exported and unit-tested.
- Package metadata now includes `bugs`, `homepage`, and the license file;
  extension limitations are documented in the README.

## 1.0.2

- Syntax highlighting, snippets, **Luca: Run Luca File**, **Luca: New Luca File**,
  clickable run diagnostics, and `luca.cliPath` / `luca.runInTerminal` settings.
