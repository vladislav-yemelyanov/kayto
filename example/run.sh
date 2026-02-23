#!/usr/bin/env sh
set -eu

echo "==> Install JS deps"
bun i

echo "==> Install Dart deps"
dart pub get

echo "==> Generate TS + Dart code"
bun run codegen.ts

echo "==> Done"
