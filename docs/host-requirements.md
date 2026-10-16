# Host requirements for embedding Luca Code

This document describes what a host environment must provide to run Luca
Code programs through the `Platform` abstraction (`src/platform.rs`), and
what each host class must additionally own. Language semantics are defined
elsewhere (`docs/locked-language-reference.md`); this document covers only
the embedding contract.

Prefer `Interpreter::run_outcome` for all new embedding-facing code. The
legacy `run`, `run_collect`, and `lib::run`/`run_with_input` APIs can
terminate the host process on valid `System.exit` (transitional behavior,
retained for the CLI and existing callers; see `docs/implementation-status.md`).

## What a host provides through `Platform`

A host implements every `Platform` method. There are no optional methods:
unsupported operations must return an explicit error (never silent
emulation), conventionally `PlatformError::unsupported` or a
`NotFound`-style failure the runtime maps to the frozen diagnostic.

| Capability | Methods | Notes |
|---|---|---|
| Module + program files | `read_file`, `is_file` | `is_file` must distinguish files from directories (`X.lucc` dirs are never module candidates) |
| `File.*` natives | `read_file`, `write_file`, `append_file`, `remove_file`, `path_exists` | Error `detail` strings surface inside frozen diagnostics; keep them stable |
| `System.env` | `env_vars` (unsorted; the runtime sorts) | May be empty; must be deterministic per host |
| `System.os` / `System.arch` | `os`, `arch` | Free-form labels; fakes override them in tests |
| Working directory | `current_dir` | Only used when the host builds default search paths |
| `ask` input | `read_line` | See below |
| Output + warnings | `emit_output`, `emit_warning` | See below |

`System.args` needs no host capability: argument vectors are injected data
(`with_cli_args`), already platform-free. `Math`/`Text` and the language
core need nothing at all.

## Synchronous operation and input

The interpreter is single-threaded and run-to-completion: every platform
call is synchronous and returns before execution continues. `read_line`
returns one line without trailing newlines; end of input is the
contract-exact `Other`/`"No more input available"` error, which the
runtime maps to the frozen `ask` EOF diagnostic. Hosts that cannot block
(e.g. browser event loops) must bridge this themselves — for example with
a pre-filled input queue like `FakePlatform`'s — because the interpreter
never yields mid-`ask`. That bridge is host architecture, not part of this
crate.

## Output and warning delivery

`emit_output` receives one already-formatted line per `print` (plus `ask`
prompts, which are output-buffer lines); `emit_warning` receives
`(line, column, message)` triples. Both are infallible by contract. The
interpreter buffers program output and drains it exactly once on success
or `System.exit`; on ordinary errors the buffer is discarded. Sinks must
preserve call order; they must not add framing the language didn't
produce (no extra prefixes, no line-joining).

## Module and relative-path resolution

The runtime joins each configured search path with `{Name}.lucc` and takes
the first `is_file` hit; builtins (`Math`, `Text`, `File`, `System`) win
before any filesystem probe. Paths are supplied by the host (CLI: program
directory, then working directory). Relative names never escape through
the runtime itself — sandboxing the tree behind `read_file`/`is_file` is
the host's job and is enforced at exactly those two methods.

## `run_outcome` vs legacy APIs

- `run_outcome` returns `Completed(output)` / `Exited { code, output }` /
  `Err(LucaError)` and never terminates or emits. This is the embedding API.
- `run`, `run_collect`, `lib::run`, `lib::run_with_input` may terminate the
  host on valid `System.exit` (flushing first). They exist for the CLI and
  existing callers. `#[deprecated]` was deliberately withheld: this
  repo's CI and Luca UI's CI both deny warnings, so deprecation would
  break builds here and in Luca UI without a coordinated migration
  (recorded as deferred work; Luca UI was not and will not be edited
  from this repo).

## Per-host requirements

- **Native desktop (Windows/macOS/Linux):** `NativePlatform` plus the
  existing `main.rs` CLI, which owns argument parsing, search paths, and
  `process::exit` translation. Fully implemented today.
- **WASM/browser (experimental spike, M3):** implemented as `wasm-host/`
  (crate `luca-wasm-host`, `cdylib`), consuming `luca-code` through its
  public API only — no interpreter or language duplication.
  - Compilation: `luca-code` (rlib) checks clean for
    `wasm32-unknown-unknown` with no source changes; the host crate carries
    the `cdylib` type, so `luca-code`'s own `Cargo.toml` is untouched.
  - Host: in-memory `Platform` (scripted files/dirs with `is_file` vs
    `path_exists` distinction, deterministic input queue plus a deny-input
    policy returning `Unsupported`, captured output/warnings); labels
    `System.os = "web"`, `System.arch = "wasm32"`; empty environment unless
    scripted; virtual search root `/`.
  - Input policy: the spike supports both pre-supplied queues and explicit
    denial (frozen `Failed to read input:` diagnostic). True asynchronous
    browser input remains bridged by the host, never by the runtime.
  - ABI: narrow `extern "C"` (`luca_host_new/free`, `luca_alloc/free`,
    setup calls returning `0`/`-1`/`-2`/`-3`, `luca_run`/`luca_check`
    returning JSON documents with explicit lengths). Handles are opaque
    `Box` pointers (no globals); every allocation is freed exactly once
    with its size; malformed input yields error documents, never traps.
    Responses are JSON parsed by built-in JSON support — no custom parser
    on either side, no new dependencies.
  - Verification: `node --test smoke.mjs` (13 tests) against the real
    artifact; CI runs the wasm check, host build + lint, and the Node
    suite (see `.github/workflows/ci.yml`). No browser UI and no npm
    package exist; this is a spike, not supported WASM product status.
- **Android:** same as WASM plus NDK packaging, signing, and lifecycle
  integration. Future work; no implementation exists.

## Implemented today vs future work

Implemented: the `Platform` trait, `NativePlatform`, deterministic
`FakePlatform` (incl. `add_dir`), runtime routing for output, warnings,
input, filesystem, modules, environment, and OS/architecture; the
`run_outcome` embedding API; file-graph validation with
`check_file_with_platform` (file paths only — unsaved buffers without a
path are syntax-checked via `check_source`; virtual entry identities are
future work); the experimental WASM host spike above. Remaining:
default-construction/search-path ownership details, coordinated
legacy-API deprecation, and every non-desktop production host.
