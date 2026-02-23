#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

REPO="vladislav-yemelyanov/kayto"
BINARY="kayto"
INSTALL_DIR="${KAYTO_INSTALL_DIR:-/usr/local/bin}"
VERSION="${KAYTO_VERSION:-}"

if ! command -v curl >/dev/null 2>&1; then
  echo "curl is required" >&2
  exit 1
fi

if ! command -v tar >/dev/null 2>&1; then
  echo "tar is required" >&2
  exit 1
fi

if [ -z "$VERSION" ]; then
  VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | sed -nE 's/.*"tag_name"[[:space:]]*:[[:space:]]*"([^"]+)".*/\1/p' | head -n1)"
fi

if [ -z "$VERSION" ]; then
  echo "Failed to resolve release version. Set KAYTO_VERSION manually, e.g. KAYTO_VERSION=v0.1.14" >&2
  exit 1
fi

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Darwin) TARGET_OS="apple-darwin" ;;
  Linux) TARGET_OS="unknown-linux-gnu" ;;
  *)
    echo "Unsupported OS: $OS" >&2
    echo "For Windows use install.ps1" >&2
    exit 1
    ;;
esac

case "$ARCH" in
  aarch64|arm64) TARGET_ARCH="aarch64" ;;
  x86_64|amd64) TARGET_ARCH="x86_64" ;;
  *)
    echo "Unsupported architecture: $ARCH" >&2
    exit 1
    ;;
esac

ARCHIVE="${BINARY}-${VERSION}-${TARGET_ARCH}-${TARGET_OS}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE}"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

echo "Downloading ${BINARY} ${VERSION} for ${TARGET_ARCH}-${TARGET_OS}..."
curl --fail --location --progress-bar --output "$TMP_DIR/$ARCHIVE" "$URL"

echo "Extracting..."
tar -xzf "$TMP_DIR/$ARCHIVE" -C "$TMP_DIR"

BINARY_PATH=""
if [ -f "$TMP_DIR/$BINARY" ]; then
  BINARY_PATH="$TMP_DIR/$BINARY"
elif [ -f "$TMP_DIR/tmp/$BINARY" ]; then
  BINARY_PATH="$TMP_DIR/tmp/$BINARY"
else
  BINARY_PATH="$(find "$TMP_DIR" -type f -name "$BINARY" | head -n 1)"
fi

if [ -z "$BINARY_PATH" ] || [ ! -f "$BINARY_PATH" ]; then
  echo "Binary not found in archive" >&2
  exit 1
fi

chmod +x "$BINARY_PATH"

echo "Installing to $INSTALL_DIR..."
if [ -w "$INSTALL_DIR" ]; then
  mv "$BINARY_PATH" "$INSTALL_DIR/$BINARY"
else
  sudo mv "$BINARY_PATH" "$INSTALL_DIR/$BINARY"
fi

if command -v "$BINARY" >/dev/null 2>&1; then
  echo "Installed: $($BINARY --version 2>/dev/null || echo "$BINARY")"
  echo "Run: $BINARY --help"
else
  echo "Installed to $INSTALL_DIR/$BINARY"
  echo "Add to PATH if needed: export PATH=\"\$PATH:$INSTALL_DIR\""
fi
