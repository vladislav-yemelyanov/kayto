#!/usr/bin/env sh
set -eu

OUTPUT_PATH="${1:-generated/schema.dart}"
INPUT_URL="${2:-https://raw.githubusercontent.com/github/rest-api-description/main/descriptions/api.github.com/api.github.com.json}"

cargo run -- --lang dart --input "$INPUT_URL" --output "$OUTPUT_PATH"
