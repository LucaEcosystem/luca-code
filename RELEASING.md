# Releasing Luca Code

Releases are explicit, maintainer-only operations. Nothing in CI publishes to
crates.io or the VS Code Marketplace, and no tag is created automatically.

## Versioning

- **Product identity:** Luca Code 1.0 Beta.
- **Package version:** currently `1.0.3`. Within the 1.0 generation, defect,
  documentation, and publishing releases bump the package version; the language
  stays at its Beta content.
- **Single source of truth:** `Cargo.toml` (`version`), `src/lib.rs`
  (`PRODUCT_VERSION`, e.g. `"1.0.3 Beta"`), and `editors/vscode/package.json`
  must agree. The metadata test enforces this.
- Do not bump the version as part of an ordinary pull request.

## Checklist

1. **Confirm scope.** The release contains only reviewed changes; no
   unreviewed language-scope changes and no dependency additions.
2. **Update versions** in all three places if this is a release:
   - `Cargo.toml`
   - `src/lib.rs` → `PRODUCT_VERSION`
   - `editors/vscode/package.json`
3. **Move changelog entries.** Promote the relevant `Unreleased` section in
   `CHANGELOG.md` to a dated version heading, and add the matching entry to
   `editors/vscode/CHANGELOG.md` when the extension changed.
4. **Verify from a clean tree:**

   ```sh
   cargo test
   cargo clippy --all-targets --all-features -- -D warnings
   cargo doc --no-deps
   cargo run --quiet -- examples/hello.lucc
   ```

   Extension:

   ```sh
   cd editors/vscode
   npm test
   npx -y @vscode/vsce package
   ```

5. **Check packaging contents** in the produced `.vsix`: `extension.js`,
   `package.json`, grammar, snippets, language configuration, `README.md`,
   `CHANGELOG.md`, and `LICENSE`. Test files and `.vscode/` are excluded by
   `.vscodeignore`.
6. **License unchanged.** Do not alter `LICENSE` or its terms in a release.

## Publishing (requires explicit authorization)

Only after the checklist passes and the maintainer has explicitly authorized
the release:

- Commit the version/changelog changes and create an annotated tag.
- Publish the extension with `vsce publish` from `editors/vscode/`.
- Publish the crate only if that decision has been made separately; the current
  CI does not publish.

Never publish, tag, or push on someone else's behalf without written
authorization.

## After release

- Announce the release with the package version and product identity together:
  “Luca Code 1.0 Beta (package 1.0.x)”.
- Open a fresh `Unreleased` section in `CHANGELOG.md`.
- Record any known issue in [known limitations](docs/learn/known-limitations.md).
