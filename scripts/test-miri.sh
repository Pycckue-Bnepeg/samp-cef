#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOOLCHAIN="nightly"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required. Install Rust from https://rustup.rs/." >&2
  exit 1
fi

if ! command -v rustup >/dev/null 2>&1; then
  echo "rustup is required to install the nightly toolchain and miri." >&2
  exit 1
fi

if ! rustup toolchain list | grep -q "^${TOOLCHAIN}"; then
  rustup toolchain install "$TOOLCHAIN"
fi

if ! rustup component list --toolchain "$TOOLCHAIN" | grep -q "^miri"; then
  rustup component add miri --toolchain "$TOOLCHAIN"
fi

cd "$ROOT_DIR"

echo "Running miri tests for cef"
export CEF_SYS_SKIP_LINK=1
cargo +"$TOOLCHAIN" miri test -p cef "$@"
