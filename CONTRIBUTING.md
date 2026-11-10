# Contributing to Luca Code

Thanks for helping with Luca Code. The project is a feature-complete **1.0 Beta**
(implementation: `lexer → parser → AST → type checking → interpreter`) and the
current work is validation, stabilization, and defect fixes. Small, focused
changes with tests are the easiest to review.

## Ground rules

- **Preserve the language.** Do not redesign syntax or reinterpret documented
  behavior. If an edge case is unspecified, choose the smallest behavior
  consistent with the [language reference](docs/reference/language-reference.md),
  test it, and record the assumption.
- **No new language scope without explicit approval.** Bug fixes, diagnostics,
  tests, examples, docs, and tooling are always welcome.
- **Keep the implementation dependency-free.** The main crate uses only the Rust
  standard library. Do not add runtime dependencies to the VS Code extension
  either.
- **Do not change the license.** The project is free to use, including
  commercially, but modifying or redistributing the Software itself requires
  prior written permission. See [LICENSE](LICENSE).
- **Do not commit generated artifacts.** `target/`, `node_modules/`, and
  `*.vsix` are ignored.

## Prerequisites

- Rust stable (`rustup show` should report a current toolchain).
- Node.js 22+ only if you are changing the VS Code extension (its tests use
  `node --test`).

## Verify before opening a pull request

From the repository root:

```sh
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo doc --no-deps
```

`cargo test` includes the example, documentation-link, and metadata tests, so it
also checks that docs and `examples/` still match behavior. For the extension:

```sh
cd editors/vscode
npm test
```

### Formatting

The repository uses a compact layout that predates a `cargo fmt` gate; a
`cargo fmt --check` run currently reports differences across the whole tree.
Do not mass-reformat files in a functional change. Match the surrounding style
and keep diffs reviewable. Formatting-only work should be its own pull request.

## What to include

- **Behavior change:** a regression test in the relevant `tests/*.rs` file. If
  the change fixes a crash or a wrong result, add the smallest program that
  reproduces it.
- **User-visible change:** an update to the [changelog](CHANGELOG.md), the
  [language reference](docs/reference/language-reference.md) or
  [standard library reference](docs/reference/stdlib.md), and — when useful — an
  example plus its real output in the
  [examples gallery](docs/learn/examples-gallery.md).
- **New limitation or intentional boundary:** document it in
  [known limitations](docs/learn/known-limitations.md), not only in code
  comments.
- **Extension change:** a unit or manifest test under `editors/vscode/test/`.

## Pull requests

1. Keep one concern per pull request.
2. Describe what changed, why, and how you verified it. Paste the commands you
   ran and their result.
3. Note anything intentionally deferred.
4. Link the issue the change closes, if any.
5. Do not bump the package version or publish anything; maintainers cut
   releases separately (see [RELEASING.md](RELEASING.md)).

## Reporting bugs

Use the issue templates. Include the exact `luca --version` output, your OS, the
smallest `.lucc` program that reproduces the problem, the command you ran, and
the actual versus expected output. For anything security-sensitive, follow
[SECURITY.md](SECURITY.md) instead of opening a public issue.

## Code of conduct

Be direct, assume good faith, and keep discussion about the code and its
behavior. Harassment or personal attacks are not acceptable.
