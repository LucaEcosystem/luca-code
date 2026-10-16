# Luca Code in VS Code (1.0 Beta)

The official extension gives `.lucc` files syntax highlighting, snippets, and one-key run.

> The extension is not on the Marketplace yet. Until it is published under `luca-ecosystem`, install it from the packaged file below.

## 1. Install the extension

1. Install the `luca` CLI first: [install.md](install.md), then check `luca --version` works in a terminal.
2. Package the extension (needs Node.js 20+; no dependencies to install):
   ```sh
   cd editors/vscode
   npx @vscode/vsce package
   ```
   This creates `luca-code-1.0.0.vsix`.
3. In VS Code: Extensions view (`⇧⌘X`) → `...` menu → **Install from VSIX…** → pick the `.vsix` file.
4. Open any `.lucc` file — keywords should be highlighted. (After Marketplace publication, replace steps 2–3 with: search "Luca Code" in the Extensions view and click Install.)

## 2. Use it

- **Luca: New Luca File** (`⇧⌘P` → type "Luca") opens a starter program.
- **Luca: Run Luca File** (same menu, or the run button in the editor title bar when a `.lucc` file is open) saves and runs the file. By default it runs in the integrated terminal (a shared "Luca" terminal), so output appears where you already look — and programs using `ask` can read your typing. Set **`luca.runInTerminal`** to off to run in the **Output** panel on the **Luca Code** channel instead, which turns `Error at line:column:` and `Warning at line:column:` output into clickable squiggles in the editor.
- `Error at line:column:` and `Warning at line:column:` output becomes squiggles in the editor; the first error also pops up with its location.
- Snippets: type `try`, `error`, `raise`, `func`, `if`, `for`, `while`, `ask`, `import`, or `print` and press Tab.

## 3. If the extension cannot find Luca

You will see: *"the 'luca' CLI was not found."* Fix it one of two ways:

- Put `luca` on your `PATH` (the installer does this; see [troubleshooting.md](troubleshooting.md#luca-command-not-found)), restart VS Code, and run again, **or**
- Set the **`luca.cliPath`** setting to the absolute path of the binary (Settings → search "Luca Code", or run **Preferences: Open User Settings (JSON)** and add `"luca.cliPath": "/Users/you/.cargo/bin/luca"`).

## 4. Upgrade / uninstall

- **Upgrade the extension:** install the newer `.vsix` the same way (VS Code replaces it), or update from the Marketplace once published.
- **Upgrade the CLI:** re-run the installer ([install.md](install.md#3-upgrade)).
- **Uninstall the extension:** Extensions view → Luca Code → Uninstall. **Uninstall the CLI:** [install.md](install.md#4-uninstall).
