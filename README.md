# Refetch Core Rust

Foundation v0.1 Rust reference implementation for the language-neutral contract maintained in [`refetch-project/concept`](https://github.com/refetch-project/concept).

This repository proves that the v0.1 schemas and fixtures can be executed deterministically. The schemas and fixtures remain the specification source of truth; Rust structs are implementation bindings.

## Workspace

```text
crates/refetch-contract/  serde models for schemas/v0.1
crates/refetch-core/      deterministic ranking and slate selection
crates/refetch-cli/       refetch rank --input request.json --output slate.json
tests/spec/v0.1/          copied concept fixture snapshot
SPEC_VERSION              concept commit used by tests
```

## Supported spec

- Spec version: v0.1
- Concept snapshot: see `SPEC_VERSION`
- Status: Foundation v0.1

## Run the CLI

```bash
cargo run -p refetch-cli -- rank --input request.json --output slate.json
```

The CLI only reads JSON, calls `refetch-core`, and writes JSON.

## Run checks

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --release
```

## Not included in v0.1

Network adapters, model calls, databases, Tokio, WASM, Dart bindings, Flutter integration, user profiles, and cloud sync are later phases.
