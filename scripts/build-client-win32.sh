#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET="i686-pc-windows-msvc"
CEF_LIB_DIR="${CEF_PATH:-$ROOT_DIR/third_party/cef}"
CEF_LIB_PATH="${CEF_LIB_DIR}/libcef.lib"
CEF_LIB_URL="https://github.com/ZOTTCE/samp-cef/releases/download/v1.1-beta/libcef.lib"
DX_SDK_LIB="${DX_SDK:-}"
DX_SDK_URL="https://download.microsoft.com/download/a/e/7/ae743f1f-632b-4809-87a9-aa1bb3458e31/DXSDK_Jun10.exe"
DX_SDK_DIR="${DX_SDK_DIR:-$ROOT_DIR/third_party/dxsdk}"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required. Install Rust from https://rustup.rs/." >&2
  exit 1
fi

if ! command -v rustup >/dev/null 2>&1; then
  echo "rustup is required to install the Windows target toolchain." >&2
  exit 1
fi

if ! cargo xwin --version >/dev/null 2>&1; then
  echo "cargo-xwin is required for cross-compiling MSVC targets." >&2
  echo "Install with: cargo install cargo-xwin --locked" >&2
  exit 1
fi

if ! command -v curl >/dev/null 2>&1; then
  echo "curl is required to download libcef.lib." >&2
  exit 1
fi

if ! command -v 7z >/dev/null 2>&1; then
  echo "7z is required to extract the DirectX SDK archive." >&2
  exit 1
fi

if ! command -v nasm >/dev/null 2>&1; then
  echo "nasm is required for building native dependencies." >&2
  echo "Install with: brew install nasm (macOS) or apt-get install nasm (Debian/Ubuntu)." >&2
  exit 1
fi

if ! command -v llvm-lib >/dev/null 2>&1; then
  if command -v brew >/dev/null 2>&1; then
    llvm_prefix="$(brew --prefix llvm 2>/dev/null || true)"
    if [ -n "$llvm_prefix" ] && [ -x "$llvm_prefix/bin/llvm-lib" ]; then
      export PATH="$llvm_prefix/bin:$PATH"
    else
      echo "llvm-lib not found. Install LLVM with: brew install llvm" >&2
      exit 1
    fi
  else
    echo "llvm-lib not found. Install LLVM and ensure llvm-lib is on PATH." >&2
    exit 1
  fi
fi

if ! rustup target list --installed | grep -q "^${TARGET}$"; then
  rustup target add "$TARGET"
fi

if [ -z "$DX_SDK_LIB" ]; then
  DX_SDK_LIB="$DX_SDK_DIR/Lib/x86"
  if [ ! -d "$DX_SDK_LIB" ]; then
    mkdir -p "$DX_SDK_DIR/Lib"
    echo "Downloading DirectX SDK (June 2010) to $DX_SDK_DIR"
    tmp_dir="$(mktemp -d)"
    curl -L "$DX_SDK_URL" -o "$tmp_dir/DXSDK_Jun10.exe"
    7z x "$tmp_dir/DXSDK_Jun10.exe" DXSDK/Lib/x86 -o"$tmp_dir/dx"
    mv "$tmp_dir/dx/DXSDK/Lib/x86" "$DX_SDK_DIR/Lib/x86"
    rm -rf "$tmp_dir"
  fi
elif [ ! -d "$DX_SDK_LIB" ]; then
  echo "DX_SDK is set but $DX_SDK_LIB does not exist." >&2
  exit 1
fi

if [ ! -f "$CEF_LIB_PATH" ]; then
  mkdir -p "$CEF_LIB_DIR"
  echo "Downloading libcef.lib to $CEF_LIB_PATH"
  curl -L "$CEF_LIB_URL" -o "$CEF_LIB_PATH"
fi

export CEF_PATH="$CEF_LIB_DIR"
export XWIN_ACCEPT_LICENSE="${XWIN_ACCEPT_LICENSE:-1}"
export LIB="${DX_SDK_LIB}${LIB:+;$LIB}"
# export CFLAGS_i686_pc_windows_msvc="${CFLAGS_i686_pc_windows_msvc:-} /FIstring.h /DHAVE_STRING_H=1"

echo "Building Windows client artifacts for $TARGET"
cargo xwin build --release --target "$TARGET" \
  -p client -p renderer -p loader \
  --xwin-arch x86,x86_64 \
  --cross-compiler clang-cl \
  --xwin-cache-dir .xwin/
