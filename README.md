# Kayto

[![Crates.io](https://img.shields.io/crates/v/kayto.svg)](https://crates.io/crates/kayto)

Fast OpenAPI parser that turns imperfect specs into a stable output schema with actionable diagnostics.

## About

`kayto` is focused on transforming OpenAPI specs into a stable output schema through a parser + IR pipeline.

Its primary goal is the output schema itself. That schema can then be consumed by separate language-specific libraries (e.g., `kayto-ts`, `kayto-dart`, and future libraries for any other language).

## Core Concept

```text
Input OpenAPI Spec ✅
        |
        v
      Parser ✅
        |
        v
        IR ✅
        |
        v
Language Codegen Module 🛠️
(any programming language, planned)
        |
        v
   Output Schema 🛠️
        |
        v
API Client Libs 🚧
(separate libraries, future)
```

In short: `kayto` is the parser/IR core that produces a stable output schema; separate libraries can use that schema to build type-safe API clients for TypeScript, Dart, and any other language.

## Why It Is Useful

- Real-world OpenAPI files are often incomplete, inconsistent, or legacy-heavy.
- Strict all-or-nothing parsers are hard to use in production pipelines.
- `kayto` helps teams extract value now and improve specs incrementally.

## Key Characteristics

- Best-effort parsing: returns valid parsed parts even when some parts fail.
- Structured diagnostics with context (`path`, `method`, `status`, `stage`).
- IR layer designed to support downstream generators and integrations.
- Covers core practical OpenAPI v3 scenarios and part of OpenAPI v2 patterns.

## Honest Project Status

`kayto` does **not** claim full OpenAPI specification coverage yet (for either v2 or v3).

The current implementation covers the main practical path (paths, methods, parameters, request body, responses, and core schema shapes), but some OpenAPI areas are still partial or not implemented.

## Roadmap

- [ ] Client code generation for **TypeScript** (priority #1)
- [ ] Client code generation for **Dart** (priority #2)
- [ ] Broader OpenAPI v2 coverage (including legacy edge cases)
- [ ] Broader OpenAPI v3 coverage (more schema constructs and media-type handling)
- [ ] Better diagnostics with clearer root-cause hints
- [ ] Regression suite based on real public API specs
- [ ] Stabilize IR as a reusable contract for integrations

## Maintainer Note

This project is currently maintained by a single author in spare time outside a full-time job.

Small, focused PRs are very welcome and appreciated. Please keep changes scoped and incremental rather than large rewrites.

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
