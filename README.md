# Kayto

[![Crates.io](https://img.shields.io/crates/v/kayto.svg)](https://crates.io/crates/kayto)

Fast OpenAPI parser and schema generator that turns imperfect specs into stable artifacts with human-readable diagnostics.

## Features

- ⚡ Fast best-effort parsing for real-world OpenAPI specs.
- 🧠 Actionable diagnostics with context (`method`, `path`, `status`, `stage`, `code`).
- 🧩 Stable IR layer between parser and language generators.
- ✅ TypeScript and Dart generators with endpoint metadata output.
- 🛡️ Graceful fallback to `unknown` instead of hard-failing generation.
- 🧪 Snapshot and unit tests for parser and generators.

## What It Does

`kayto` parses OpenAPI, builds an internal IR, and passes that IR to language generators.

Core concept:

```text
OpenAPI Spec (v2/v3, partial)
        |
        v
      Parser
        |
        v
        IR
        |
        v
   Generators
   (ts ✅, dart ✅, ...)
        |
        v
  Output Artifacts
 (generated/schema.ts, ...)
        |
        v
  API Client Libraries
   (separate projects)
```

## Current Status

- TypeScript generator is implemented and produces `schema.ts`.
- Dart generator is implemented and produces `schema.dart`.
- Parsing is best-effort: unsupported parts are downgraded to `unknown` where possible.
- Diagnostics are still emitted with context and stable codes.

Important: this is not full OpenAPI v2/v3 compliance yet. It is practical coverage for many real APIs, not spec-complete coverage.

## CLI

```bash
cargo run -- --lang <ts|dart> --input <OPENAPI_URL> --output <PATH>
```

Arguments:

- `--lang` (required): target generator (`ts` or `dart`).
- `--input` (required): OpenAPI URL (HTTPS).
- `--output` (required): output file path.

## Quick Start

Generate TypeScript schema:

```bash
cargo run -- --lang ts --input "https://raw.githubusercontent.com/github/rest-api-description/main/descriptions/api.github.com/api.github.com.json" --output "generated/schema.ts"
```

Or via helper script:

```bash
./scripts/generate-ts.sh
```

Generate Dart schema:

```bash
./scripts/generate-dart.sh
```

Script arguments:

```bash
./scripts/generate-ts.sh [output_path] [input_url]
./scripts/generate-dart.sh [output_path] [input_url]
```

Defaults are provided by scripts:

- TS: `generated/schema.ts` + GitHub public OpenAPI JSON
- Dart: `generated/schema.dart` + GitHub public OpenAPI JSON

## Generated TypeScript Model

The TS generator writes `schema.ts` with:

- `Schemas` interface: map of model names (`Schemas["UserModel"]`).
- `Endpoints` interface: endpoint metadata by method/path (`Endpoints["get"]["/path"]`).
- Reuse of `Schemas[...]` in endpoint params/body/responses.

Example:

```ts
import type { Endpoints } from "./schema";

type GetUser = Endpoints["get"]["/user"];
```

## Generated Dart Model

The Dart generator writes `schema.dart` with:

- model typedefs (`typedef UserModel = ...`),
- `Schemas.types` registry,
- `EndpointMeta` model,
- method groups with both alias and path access:
  - `Endpoints.get.user`
  - `Endpoints.get['/user']`

Example:

```dart
import 'schema.dart';

final endpoint = Endpoints.get.user;
```

## Diagnostics

Diagnostics are grouped by endpoint and include:

- kind: `unsupported`, `invalid_spec`, `incomplete_data`
- stage: parser stage (`schema`, `response.ref`, `parameters.ref`, ...)
- code: stable machine-readable code (for example, `unknown_schema_missing_type_and_ref`)
- path/method/status context

When schemas are downgraded to `unknown`, CLI also prints an aggregated summary by `unknown_*` code.

## Tests

Run:

```bash
cargo test
```

The test suite currently covers parser behavior, CLI argument parsing, TS and Dart naming/formatting, and end-to-end rendering snapshots (including `anyOf` / `oneOf` / `allOf`).

Coverage snapshot (`cargo llvm-cov --workspace --summary-only`):

- Regions: `81.68%`
- Lines: `86.39%`
- Functions: `91.11%`

## Test Coverage Status

What is covered well:

- Parser core flows and many degradation paths (`unknown_*` mapping + diagnostics).
- TS and Dart render snapshots, including combinator scenarios (`anyOf` / `oneOf` / `allOf`).
- Naming normalization/collision behavior.
- Generator file-write entrypoints.
- CLI argument parsing rules.

What is not fully covered yet:

- `main.rs` runtime flow (network fetch, end-to-end CLI execution with real HTTP).
- Some conversion branches in `generators/ts/convert.rs` and `generators/dart/convert.rs`.
- Some parser helper branches (`schema_mapping`, `request_parameters`) still have uncovered edge paths.

Coverage is intentionally described as practical/high for core paths, but not 100%.
