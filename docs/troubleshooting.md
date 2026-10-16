# Troubleshooting

## `luca: command not found`

The binary installed, but your shell cannot find it.

- **macOS/Linux:** run `export PATH="$HOME/.cargo/bin:$PATH"`, then `luca --version`. If that works, make it permanent by adding that line to `~/.zshrc` (macOS default shell) or `~/.bashrc`, then **restart the terminal**.
- **Windows:** `%USERPROFILE%\.cargo\bin` must be in your **user** `PATH` (Settings → search "environment variables" → Environment Variables). **Restart the terminal** (or Windows Terminal) afterwards — open terminals do not pick up `PATH` changes.
- **VS Code terminals** inherit `PATH` from how VS Code was launched. If `luca` works in Terminal but not in VS Code, restart VS Code (or set the `luca.cliPath` setting — see [vscode.md](vscode.md#3-if-the-extension-cannot-find-luca)).

## Installer says Rust/cargo is missing

The installer installs Rust via rustup automatically. If that fails:

- Check network access to `sh.rustup.rs` (macOS/Linux) or `win.rustup.rs` (Windows).
- Install Rust manually from [rustup.rs](https://rustup.rs), restart the terminal, confirm `cargo --version`, then re-run the installer.

## `git clone` asks for a login / fails

While the repository is private, anonymous cloning does not work. Either clone with an account that has access, or skip cloning: download/copy the source and run `bash scripts/install.sh` (or `.\scripts\install.ps1`) from inside the checkout. After the public launch this goes away.

## Build (`cargo install`) fails

- Make sure you are on a recent stable Rust: `rustup update`, then re-run.
- On Linux, a C linker is required (`build-essential` on Debian/Ubuntu, "Development Tools" on Fedora). Install it and re-run.
- Paste the full error output when asking for help — the first `error:` line matters most.

## `Luca Code source files must use the .lucc extension`

You ran `luca` on a file without the `.lucc` suffix (e.g. `luca hello.txt`). Rename it to `hello.lucc` and re-run.

## `Could not read <file>: ...`

The path is wrong or the file is unreadable. Check spelling, and remember paths are relative to where you run `luca`. Module imports (`import Name` → `Name.lucc`) resolve from the program's own directory first, then the working directory.

## `luca check` (validate without running)

`luca check <file.lucc> [--format human|json]` parses the file plus its transitive `import` graph and reports `Error at line:column: ...` (human, one per line) or `[{"message":"...","line":N,"column":M}]` (JSON, `[]` when valid). It never executes application code: `print` produces no output, `ask` never prompts, `File.write/append/delete` never touch the filesystem, `System.exit` never terminates the checker.

Exit codes: 0 valid, 1 validation errors, 2 usage or internal failure. Internal failures print `luca check: ...` to stderr (e.g. entry file unreadable) and never appear as JSON diagnostics. Dynamic errors (call arity on runtime values, division by zero on computed values, `ask` conversion of real input) only appear when you `luca run`.

## `Error at line:column: ...`

This is a Luca program error, not an install problem. Read it as **what** + **where** (line:column). Common first ones:

- `'x' is not declared` — typo, or the binding is declared later (programs run top to bottom).
- `print accepts exactly one argument` — use interpolation: `print("a and {b}")`.
- `Expected ...` — a syntax slip near the reported spot (often a missing `:` or wrong indentation; blocks use `:` + indentation).

See [first-program.md](first-program.md#when-something-goes-wrong) and [language-spec.md](language-spec.md).

## Extension shows no highlighting / commands missing

- Confirm the file ends in `.lucc` and VS Code shows "Luca Code" in the bottom-right language picker. If not, reinstall the `.vsix` ([vscode.md](vscode.md#1-install-the-extension)).
- Confirm the extension is enabled for your workspace (Extensions view → Luca Code → Enable).

## Still stuck

Re-run with the exact failing command and keep the full output (including the `luca installer:` lines). That output is what a helper needs first.
