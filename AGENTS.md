# Luca Code implementation instructions

Build Luca Code according to `docs/locked-language-reference.md`. That file consolidates the decisions from the Luca Code design chat and takes precedence over the older implemented-subset note in `docs/language-spec.md`. The current release is **0.1 Alpha**, whose goal is the bare-minimum runnable language foundation. The full language is the goal across the 0.x Alpha releases, not all inside 0.1.

- Continue implementation; do not stop at a plan or the existing subset. Finish the bare-minimum 0.1 foundation first. Then progress through 0.2–0.9 Alpha in coherent releases, keeping each usable, buildable, tested, and documented. By 0.9, implement the full reference and prepare for 1.0 Beta.
- Do not redesign syntax or silently reinterpret locked behavior. Add regression tests for every rule and update `.lucc` examples and implementation status as features land.
- First rerun `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, and the example. Fix any regressions, then continue implementing.
- Correct the known behavior mismatch first: the locked spec says `10 / 4` evaluates to `2.5` (`dec`), while the initial subset used Rust integer-division truncation.
- Implement `dec` with exact decimal arithmetic and retained scale; do not use `f64` where it loses required decimal representation/precision.
- Preserve the lexer → parser → AST → type checking → interpreter architecture. The interpreter is the reference implementation; defer LLVM/native code generation.
- Do not put the whole reference in 0.1. Keep a clear release label for each meaningful alpha milestone and don't advance versions automatically without recording the release scope. Do not ask the user to approve each feature. If an edge case is unspecified, choose the smallest behavior consistent with the reference, test it, and record the assumption.
- Source extensions are `.lucc` for Luca Code and `.lucu` for Luca UI. Do not change them.
- Do not commit changes unless asked.
