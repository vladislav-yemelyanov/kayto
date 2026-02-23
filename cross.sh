#!/usr/bin/env bash
set -euo pipefail

BIN="kayto"
VERSION="v$(sed -nE 's/^version = "([^"]+)"/\1/p' Cargo.toml | head -n1)"
RELEASE_DIR="releases/$VERSION"
TARGETS=(
  "x86_64-unknown-linux-gnu"
  "x86_64-apple-darwin"
  "aarch64-apple-darwin"
  "x86_64-pc-windows-gnu"
)

if [ -z "$VERSION" ] || [ "$VERSION" = "v" ]; then
  echo "Failed to detect version from Cargo.toml"
  exit 1
fi

mkdir -p "$RELEASE_DIR"

for TARGET in "${TARGETS[@]}"; do
  if [ "$TARGET" = "x86_64-unknown-linux-gnu" ] || [ "$TARGET" = "x86_64-pc-windows-gnu" ]; then
    cross build --release --target "$TARGET"
  else
    cargo build --release --target "$TARGET"
  fi

  TMP_DIR="$RELEASE_DIR/tmp"
  mkdir -p "$TMP_DIR"

  if [ "$TARGET" = "x86_64-pc-windows-gnu" ]; then
    cp "target/$TARGET/release/$BIN.exe" "$TMP_DIR/"
  else
    cp "target/$TARGET/release/$BIN" "$TMP_DIR/"
  fi

  cp README.md LICENSE "$TMP_DIR/" || true

  if [ "$TARGET" = "x86_64-pc-windows-gnu" ]; then
    (
      cd "$TMP_DIR"
      zip -qr "../$BIN-$VERSION-$TARGET.zip" .
    )
  else
    tar -czf "$RELEASE_DIR/$BIN-$VERSION-$TARGET.tar.gz" -C "$RELEASE_DIR" tmp
  fi

  rm -rf "$TMP_DIR"
done

echo "Release artifacts are ready in: $RELEASE_DIR"
