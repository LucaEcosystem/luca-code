# Luca Code in VS Code

The official extension gives `.lucc` files syntax highlighting, snippets, one-key run, and syntax/import validation. It is published on the Marketplace by `luca-ecosystem`, and this page mirrors what you see on its Marketplace listing.

## 1. Install the extension

1. Install the `luca` CLI first: [install.md](install.md), then check `luca --version` works in a terminal.
2. In VS Code: Extensions view (`⇧⌘X`) → search **"Luca Code"** → click **Install**.
3. Open any `.lucc` file — keywords should be highlighted.

## 2. Use it

- **Luca: New Luca File** (`⇧⌘P` → type "Luca") opens a starter program.
- **Luca: Run Luca File** (same menu, or the run button in the editor title bar when a `.lucc` file is open) saves and runs the file. By default it runs in the integrated terminal (a shared "Luca" terminal), so output appears where you already look — and programs using `ask` can read your typing. Set **`luca.runInTerminal`** to off to run in the **Output** panel on the **Luca Code** channel instead, which turns `Error at line:column:` and `Warning at line:column:` output into clickable squiggles in the editor.
- **Luca: Check Luca File** (same menu, or the check button in the editor title bar) saves the file and runs `luca check --format json`. Syntax errors and missing or circular imports appear as squiggles **without executing the program**. It never evaluates expressions or touches the filesystem, so type mismatches on computed values, bad runtime arity, division by zero, and `ask` conversion errors still surface only when you run.

| Setting | Default | What it does |
|---|---|---|
| `luca.cliPath` | `luca` (from `PATH`) | Absolute path to the `luca` binary when it is not on your `PATH` |
| `luca.runInTerminal` | `true` | Run in the integrated terminal; when off, run in the Output > Luca Code channel with clickable diagnostics |

- `Error at line:column:` and `Warning at line:column:` output becomes squiggles in the editor; the first error also pops up with its location.
- Snippets: type `try`, `error`, `raise`, `func`, `if`, `for`, `while`, `ask`, `import`, or `print` and press Tab.

### What the extension does not do

- **Check is syntax + import validation only.** `luca check` does not statically type-check expressions, so errors such as assigning a `str` to an `int` variable surface only when you run.
- **The CLI must be installed.** The extension is a front end for the `luca` binary; it does not bundle an interpreter.
- **No language server.** Highlighting is TextMate-based, and diagnostics come from running or checking the file, not from live incremental analysis.
- **Source nesting is capped at 32 levels and function recursion at 64 frames.** Deeper code reports a normal diagnostic rather than crashing; see [known limitations](known-limitations.md).

## 3. If the extension cannot find Luca

You will see: *"the 'luca' CLI was not found."* Fix it one of two ways:

- Put `luca` on your `PATH` (the installer does this; see [troubleshooting.md](troubleshooting.md#luca-command-not-found)), restart VS Code, and run again, **or**
- Set the **`luca.cliPath`** setting to the absolute path of the binary (Settings → search "Luca Code", or run **Preferences: Open User Settings (JSON)** and add `"luca.cliPath": "/Users/you/.cargo/bin/luca"`).

## 4. Upgrade / uninstall

- **Upgrade the extension:** VS Code updates Marketplace extensions automatically, or reinstall from the Marketplace listing to force the newest version. (Packaged by hand? The extension source lives in `editors/vscode/`; current extension version is **1.0.3**.)
- **Upgrade the CLI:** re-run the installer ([install.md](install.md#3-upgrade)).
- **Uninstall the extension:** Extensions view → Luca Code → Uninstall. **Uninstall the CLI:** [install.md](install.md#4-uninstall).

## Next steps

Learning the language: [the guided tour](tour/index.md). Otherwise, back to the [first program](first-program.md).
