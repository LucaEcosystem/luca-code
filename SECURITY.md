# Security policy

## Supported versions

Luca Code 1.0 Beta is the supported line. The package version is currently
**1.0.3**; security fixes land in the next package release on that line.

## Reporting a vulnerability

Please do **not** open a public issue for a vulnerability.

1. Use GitHub's private vulnerability reporting: open the repository's
   **Security** tab → **Advisories** → **Report a vulnerability**.
2. If that channel is unavailable, open a minimal public issue saying only that
   you have a security report and asking for a private channel. Do not include
   exploit details, payloads, or affected paths in the public issue.

Include: affected version (`luca --version`), OS, a minimal reproducer, the
impact you believe is possible, and any suggested fix.

## What is in scope

- Interpreter panics, aborts, or stack overflows reachable from source input
  (the documented nesting/recursion limits should produce diagnostics, not
  crashes).
- Memory-safety or integer-overflow defects in the Rust implementation.
- Command or path injection in the VS Code extension.
- Escapes from documented resource limits in `luca check` (module count/depth)
  that can hang or crash validation.
- Supply-chain issues in the repository's build or packaging scripts.

## What is not in scope

- **`File.*` is intentionally unsandboxed.** A Luca program can read and write
  whatever the running user can. Treat unknown `.lucc` files like shell scripts.
- Programs terminating via `System.exit`, consuming CPU in non-terminating
  loops, or exhausting memory through ordinary data structures are expected
  behavior. There is no wall-clock or memory quota.
- Source nesting above 32 levels or recursion above 64 frames is rejected by
  design; a clean diagnostic is the expected outcome.
- Vulnerabilities in third-party tools used only by maintainers (for example,
  the Marketplace packaging CLI) should be reported upstream.

## Disclosure

We will acknowledge a report as soon as practical, investigate, and coordinate a
fix and disclosure timeline with you. Please give us a reasonable window before
publishing details. Credit is offered unless you prefer to remain anonymous.
