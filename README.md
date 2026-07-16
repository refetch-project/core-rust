# Refetch Core Rust

Foundation v0.1 Rust reference implementation for the language-neutral contract maintained in [`refetch-project/concept`](https://github.com/refetch-project/concept).

The **concept** repository is the specification source of truth. This repository provides the Rust reference implementation and keeps a read-only snapshot of the locked Concept contract for offline conformance tests.

## Locked spec

- Spec version: `v0.1`
- Concept repository: `https://github.com/refetch-project/concept`
- Locked Concept commit: `a49e51bbfd04462398bbb7ea613f003b2c417544`
- Tag: `null` (no nonexistent tag is recorded)
- Lock file: `SPEC_LOCK.json`
- Snapshot: `tests/spec/v0.1/`

## Workspace

```text
crates/refetch-contract/  serde bindings for the locked schemas/v0.1
crates/refetch-core/      deterministic ranking and slate selection
crates/refetch-cli/       refetch rank --input request.json --output slate.json
tests/spec/v0.1/          read-only Concept snapshot and SHA-256 manifest
scripts/                  offline snapshot verification/update helpers
```

## Verify the spec snapshot

```bash
python3 scripts/verify-spec-snapshot.py
```

Updating the snapshot must be explicit and must use a local Concept checkout already at the locked commit:

```bash
scripts/sync-spec-snapshot.sh /path/to/concept
```

CI does not need network access to verify the snapshot manifest.

## Run the CLI

```bash
cargo run -p refetch-cli -- rank --input tests/spec/v0.1/fixtures/v0.1/valid/production.rank-request.json --output slate.json
```

The CLI only reads JSON, deserializes and validates it, calls `refetch-core`, and writes a `FeedSlate`. It does not perform network, model, database, or UI work.

## Run checks

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --release
```

## Not included in Foundation v0.1.2

Network adapters, AI analyzers, WASM, Flutter/Dart FFI, Feed Lab UI, Bilibili/PiliPlus integration, databases, cloud services, Tokio runtime, App Semantic Contract, MCP, AG-UI, and A2UI are out of scope.
