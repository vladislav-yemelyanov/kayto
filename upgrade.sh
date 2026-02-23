#!/usr/bin/env sh
set -euo pipefail

cargo set-version --bump patch
sh cross.sh
git add .
git commit -m "upgrade"
git push
cargo package
cargo publish
