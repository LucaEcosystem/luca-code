# Luca Code implementation instructions

Build Luca Code according to `docs/locked-language-reference.md`. That file consolidates the decisions from the Luca Code design chat and takes precedence over the older implemented-subset note in `docs/language-spec.md`. The current release is **1.0 Beta**, a feature-complete Beta implementing the full locked reference. Work from here is validation, stabilization, and defect fixes toward 1.0 — no new language scope without explicit approval.

- Preserve implemented behavior; do not regress the usable, buildable, tested, and documented bar. Keep a clear release label for each meaningful milestone and don't advance versions without recording the release scope. Do not ask the user to approve each defect fix. If an edge case is unspecified, choose the smallest behavior consistent with the reference, test it, and record the assumption.
- Do not redesign syntax or silently reinterpret locked behavior. Add regression tests for every rule and update `.lucc` examples and implementation status as features land.
- First rerun `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, and the example. Fix any regressions, then continue implementing.
- Preserve the locked behavior: `10 / 4` evaluates to `2.5` (`dec`), not Rust integer-division truncation — this is pinned by regression tests; do not reintroduce truncation.
- Implement `dec` with exact decimal arithmetic and retained scale; do not use `f64` where it loses required decimal representation/precision.
- Preserve the lexer → parser → AST → type checking → interpreter architecture. The interpreter is the reference implementation; defer LLVM/native code generation.
- Do not expand language scope in a stabilization milestone. Keep a clear release label for each meaningful milestone and don't advance versions automatically without recording the release scope. Do not ask the user to approve each feature. If an edge case is unspecified, choose the smallest behavior consistent with the reference, test it, and record the assumption.
- Source extensions are `.lucc` for Luca Code and `.lucu` for Luca UI. Do not change them.
- Do not commit changes unless asked.
