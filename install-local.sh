#!/usr/bin/env bash
cargo set-version --bump patch;

set -euo pipefail
IFS=$'\n\t'

BINARY="kayto"
INSTALL_DIR="${KAYTO_INSTALL_DIR:-/usr/local/bin}"
BUILD_TARGET="${KAYTO_BUILD_TARGET:-}"

echo "Building local ${BINARY} release..."
if [ -n "$BUILD_TARGET" ]; then
  cargo build --release --target "$BUILD_TARGET"
  BINARY_PATH="target/$BUILD_TARGET/release/$BINARY"
else
  cargo build --release
  BINARY_PATH="target/release/$BINARY"
fi

if [ ! -f "$BINARY_PATH" ]; then
  echo "Build completed but binary not found: $BINARY_PATH" >&2
  exit 1
fi

chmod +x "$BINARY_PATH"

echo "Installing to $INSTALL_DIR..."
if [ -w "$INSTALL_DIR" ]; then
  cp "$BINARY_PATH" "$INSTALL_DIR/$BINARY"
else
  sudo cp "$BINARY_PATH" "$INSTALL_DIR/$BINARY"
fi

if command -v "$BINARY" >/dev/null 2>&1; then
  echo "Installed: $($BINARY --version 2>/dev/null || echo "$BINARY")"
  echo "Run: $BINARY --help"
else
  echo "Installed to $INSTALL_DIR/$BINARY"
  echo "Add to PATH if needed: export PATH=\"\$PATH:$INSTALL_DIR\""
fi
