## Summary

<!-- What changed, and why? One concern per pull request. -->

## Related issue

<!-- Closes #... (or "none") -->

## Changes

<!-- Bullet the user-visible and internal changes. -->

-

## Verification

<!-- Paste the commands you ran and their results. -->

- [ ] `cargo test`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo doc --no-deps`
- [ ] `cd editors/vscode && npm test` (extension changes only)

## Checklist

- [ ] Behavior changes include a regression test.
- [ ] User-visible changes update the changelog and the relevant reference doc.
- [ ] New examples (if any) are listed in `examples/README.md` and the gallery with real output.
- [ ] No new dependencies.
- [ ] No version bump, tag, or publish step.
- [ ] `LICENSE` is unchanged.
- [ ] I matched the surrounding code style and did not mass-reformat files.

## Deferred / follow-up

<!-- Anything intentionally left out. -->
