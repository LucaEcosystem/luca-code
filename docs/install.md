# Install Luca Code (1.0 Beta)

The `luca` command runs Luca programs: `luca hello.lucc`. Installing builds it from source, which takes a few minutes but needs no account and costs nothing.

> If the repository is still private, anonymous cloning does not work: clone it with an account that has access (or download the source) and install from inside the checkout — step 1 covers this.

## 1. macOS or Linux

You need `git` and Rust (`cargo`). The installer fetches Rust for you if it is missing.

**From a local checkout** (works today, no network beyond Rust itself):

```sh
cd luca-code
bash scripts/install.sh
```

**From the public repository** (after launch):

```sh
git clone --depth 1 https://github.com/LucaEcosystem/luca-code.git
cd luca-code
bash scripts/install.sh
```

Or one line without cloning first:

```sh
LUCA_REPO_URL=https://github.com/LucaEcosystem/luca-code.git bash <(curl -sSf https://raw.githubusercontent.com/LucaEcosystem/luca-code/main/scripts/install.sh)
```

Then verify:

```sh
luca --version   # prints: luca 1.0 Beta
```

If the installer says `luca` is not on your `PATH`, add the line it prints (usually `export PATH="$HOME/.cargo/bin:$PATH"`) to `~/.zshrc` (macOS default) or `~/.bashrc` (most Linux), restart the terminal, and verify again.

## 2. Windows

You need `git` and Rust. The installer fetches Rust for you if it is missing. Open **PowerShell** (not CMD):

**From a local checkout:**

```powershell
cd luca-code
.\scripts\install.ps1
```

**From the public repository** (after launch):

```powershell
git clone --depth 1 https://github.com/LucaEcosystem/luca-code.git
cd luca-code
.\scripts\install.ps1
```

If Windows blocks the script, run this once, then re-run the installer:

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

Then verify:

```powershell
luca --version   # prints: luca 1.0 Beta
```

If `luca` is not recognized, add `%USERPROFILE%\.cargo\bin` to your user `PATH` (search "environment variables" in Settings), **restart the terminal**, and verify again.

## 3. Upgrade

Re-run the same installer. It always rebuilds and replaces the old binary:

```sh
bash scripts/install.sh          # macOS / Linux
.\scripts\install.ps1            # Windows
```

Then `luca --version` to confirm the new version.

## 4. Uninstall

```sh
cargo uninstall luca-code        # removes the luca binary
```

That leaves Rust and your `.lucc` files untouched. To also remove a manually added `PATH` line, delete it from `~/.zshrc` / `~/.bashrc` (macOS/Linux) or from the user `PATH` (Windows).

## Next step

Write your first program: [first-program.md](first-program.md). Stuck? See [troubleshooting.md](troubleshooting.md).
