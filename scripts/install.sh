#!/usr/bin/env bash
# Luca Code installer for macOS and Linux.
#
# Installs the `luca` CLI by building from source with Cargo:
#   bash scripts/install.sh            # from inside a Luca Code checkout
#   LUCA_REPO_URL=<url> bash scripts/install.sh   # clone then install
#
# Override with LUCA_REPO_URL to install from a fork or mirror.
# Re-running this script upgrades an existing install.
set -euo pipefail

LUCA_REPO_URL="${LUCA_REPO_URL:-https://github.com/LucaEcosystem/luca-code.git}"
LUCA_REF="${LUCA_REF:-main}"

log() { printf 'luca installer: %s\n' "$*"; }
die() { printf 'luca installer: error: %b\n' "$*" >&2; exit 1; }

need_cmd() { command -v "$1" >/dev/null 2>&1; }

ensure_rust() {
    if need_cmd cargo; then
        return 0
    fi
    log "Rust (cargo) not found; installing via rustup..."
    need_cmd curl || die "curl is required to install Rust. Install curl, then re-run."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
    # shellcheck disable=SC1090
    source "${HOME}/.cargo/env"
    need_cmd cargo || die "cargo is still missing after rustup. Open a new terminal and re-run."
}

find_source_dir() {
    if [ -f "./Cargo.toml" ] && grep -q 'name = "luca-code"' ./Cargo.toml; then
        pwd
        return 0
    fi
    return 1
}

SOURCE_DIR=""
if SOURCE_DIR="$(find_source_dir)"; then
    log "using local checkout at ${SOURCE_DIR}"
else
    need_cmd git || die "git is required to download Luca Code. Install git, then re-run."
    WORKDIR="$(mktemp -d)"
    trap 'rm -rf "${WORKDIR}"' EXIT
    log "cloning ${LUCA_REPO_URL} (${LUCA_REF})..."
    git clone --depth 1 --branch "${LUCA_REF}" "${LUCA_REPO_URL}" "${WORKDIR}/luca-code" \
        || die "clone failed. If the repository is still private, clone it yourself and re-run from inside the checkout."
    SOURCE_DIR="${WORKDIR}/luca-code"
fi

ensure_rust

log "building and installing luca (this takes a minute)..."
# Always --force so re-running this script upgrades an existing install.
cargo install --force --path "${SOURCE_DIR}"

if ! need_cmd luca; then
    CARGO_BIN="${CARGO_HOME:-${HOME}/.cargo}/bin"
    die "'luca' installed to ${CARGO_BIN} but it is not on your PATH. Add this line to ~/.zshrc or ~/.bashrc and restart your terminal:\n  export PATH=\"${CARGO_BIN}:\$PATH\""
fi

log "installed: $(luca --version)"
log "try it: printf 'print(\"Hello, Luca!\")\n' > hello.lucc && luca hello.lucc"
