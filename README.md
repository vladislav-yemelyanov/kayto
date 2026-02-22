# Kayto

[![Crates.io](https://img.shields.io/crates/v/kayto.svg)](https://crates.io/crates/kayto)

Fast OpenAPI v2/v3 parser with structured diagnostics.

## What It Does

- Parses OpenAPI paths and methods.
- Extracts:
  - request parameters
  - request body schema
  - response schemas
- Returns parsed requests plus parse issues (non-fatal problems with context: `path`, `method`, `status`).

## Current CLI Behavior

The CLI currently reads `./api_example.json`, parses it, and prints:

1. Number of parsed requests
2. Parsed request details (`Debug` view)
3. Grouped parse issues in a readable format:

```text
issue GET /pets:
    problem: ...
```

## Run

```bash
cargo run
```

## Notes

- Parsing is best-effort: valid parts are still returned even if some parts fail.
- Issues are grouped by `METHOD + PATH` for easier debugging.
- OpenAPI `default` response is currently reported as an issue in status parsing because it is not a numeric HTTP status code.
