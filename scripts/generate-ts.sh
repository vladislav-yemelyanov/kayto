#!/usr/bin/env sh
set -eu

OUTPUT_PATH="${1:-generated/schema.ts}"
INPUT_URL="${2:-https://raw.githubusercontent.com/github/rest-api-description/main/descriptions/api.github.com/api.github.com.json}"

cargo run -- --lang ts --input "$INPUT_URL" --output "$OUTPUT_PATH"
