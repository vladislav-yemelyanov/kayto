# Kayto

[![Crates.io](https://img.shields.io/crates/v/kayto.svg)](https://crates.io/crates/kayto)

Fast OpenAPI parser and schema generator that turns imperfect specs into stable artifacts with human-readable diagnostics.

From imperfect OpenAPI to production-ready schema with stable output and clear diagnostics.

## Features

- ⚡ Fast best-effort parsing for real-world OpenAPI specs.
- 🧠 Clear, actionable diagnostics that help you fix spec issues faster.
- 🧩 Stable IR layer between parser and language generators.
- ✅ TypeScript and Dart generators with endpoint metadata output.
- 🛡️ Generation keeps working even when parts of the spec are unsupported.

## Release Binaries (Multi-OS)

`kayto` CLI release builds are available for 3 OS and 4 targets. ✅

Support matrix:

| OS                  | Target                     | Status   |
| ------------------- | -------------------------- | -------- |
| Linux (x64)         | `x86_64-unknown-linux-gnu` | ✅ Ready |
| macOS Intel         | `x86_64-apple-darwin`      | ✅ Ready |
| macOS Apple Silicon | `aarch64-apple-darwin`     | ✅ Ready |
| Windows (x64)       | `x86_64-pc-windows-gnu`    | ✅ Ready |

## Install From Releases

Linux + macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/vladislav-yemelyanov/kayto/main/install.sh | bash
```

Windows (PowerShell):

```powershell
irm https://raw.githubusercontent.com/vladislav-yemelyanov/kayto/main/install.ps1 | iex
```

## What It Does

`kayto` parses OpenAPI, builds an internal IR, and passes that IR to language generators.

Core concept:

```text
   OpenAPI Spec (v2/v3, partial) ✅
                 |
                 v
             Parser ✅
                 |
                 v
               IR ✅
                 |
                 v
             Generators
        (ts ✅, dart ✅, ...)
                 |
                 v
         Output Artifacts ✅
      (generated/schema.ts, ...)
                 |
                 v
         API Client Libraries
      (kayto_ts ✅, kayto_dart 🚧)
```

## Why Kayto

- Works even when OpenAPI is incomplete or inconsistent.
- Keeps one stable schema contract (`Schemas` + `Endpoints`) across projects.
- Lets teams integrate API clients quickly without custom parser glue.

## Current Status

- ✅ TypeScript generator (`schema.ts`)
- ✅ Dart generator (`schema.dart`)
- ✅ Best-effort parsing with graceful fallback to `unknown`
- ✅ Structured diagnostics with stable issue codes
- ✅ Production-ready TypeScript client:
  [`kayto_ts`](https://github.com/vladislav-yemelyanov/kayto_ts)
- 🚧 Dart client is in progress

Note: Kayto targets practical API coverage, not full OpenAPI v2/v3 spec compliance.

## Quick Start

CLI arguments are required: `--lang`, `--input`, `--output`.

TypeScript:

```bash
kayto --lang ts --input "https://raw.githubusercontent.com/github/rest-api-description/main/descriptions/api.github.com/api.github.com.json" --output "generated/schema.ts"
```

Dart:

```bash
kayto --lang dart --input "https://raw.githubusercontent.com/github/rest-api-description/main/descriptions/api.github.com/api.github.com.json" --output "generated/schema.dart"
```

Helper scripts:

```bash
./scripts/generate-ts.sh
./scripts/generate-dart.sh
```

Scripts support optional args:

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

The Dart output is built for simple client usage:

- quick endpoint access (`Endpoints.get.user` or `Endpoints.get['/user']`),
- one consistent schema contract across projects,
- easy integration into API clients with minimal glue code.

Example:

```dart
import 'schema.dart';

final endpoint = Endpoints.get.user;
```

## Kayto Schema Standard

Kayto output is a stable contract designed for long-term maintenance:

- predictable structure (`Schemas` + `Endpoints`),
- easy client integration without per-project parser logic,
- stable generated shape across releases to reduce integration churn.

In short: generated schemas are reusable infrastructure, not throwaway code.

## Diagnostics

Diagnostics are built for fast debugging:

- show exactly which endpoint has a problem,
- explain what went wrong in plain text,
- keep stable issue codes for automation/CI,
- provide a short summary of unsupported schema parts.

You can quickly see what to fix now and what can stay as-is for later.

## Roadmap

Planned generator targets:

- Swift
- Kotlin
- C#
- Go
- Rust

Client SDKs as separate projects:

- ✅ [`kayto_ts`](https://github.com/vladislav-yemelyanov/kayto_ts) — production-ready
- 🚧 `kayto_dart` — in progress

Distribution and developer experience:

- Improve docs for working with many different Swagger/OpenAPI services in real projects.
- Add support for local spec files (`.json` / `.yaml`) in addition to HTTPS input.
