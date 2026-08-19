# Refetch Core integration guide

> Status: implementation guide, not a language-neutral specification
>
> Foundation revision: `v0.1.3`
>
> Contract version: `v0.1`

This document describes the integration boundary currently implemented by the
Rust reference Core. When it conflicts with the locked Concept snapshot, the
Concept RFC, Schema, fixtures, and expected outputs take precedence.

## 1. What can integrate today

The supported input is a complete `RankRequest`. It already contains:

- normalized `FeedCandidate` records;
- one `AnalysisRecord` for every candidate;
- explicit Signal values and Evidence references;
- the user-selected `LensProfile`;
- deterministic request context, including `generatedAt`.

Core validates that request and returns a deterministic `FeedSlate`. It does
not perform network access, source collection, model calls, persistence, user
profiling, or UI work.

Two invocation boundaries are available:

1. Rust hosts call `refetch_core::rank(&request)` in process.
2. Other hosts invoke `refetch rank` with JSON input and output files.

The CLI is suitable for frozen-sample experiments and early Feed Lab work. It
is not a stable FFI, WASM, Flutter, streaming, or network protocol.

## 2. Version checks

Every v0.1 `RankRequest`, nested candidate, analysis record, and Lens must use
the exact `specVersion` required by the locked Schema. Normal unknown fields are
rejected; additive application data belongs under explicit `extensions`.

The following version axes are independent:

| Axis | Current value | Meaning |
| --- | --- | --- |
| JSON contract | `v0.1` | Cross-language object and behavior contract |
| Foundation revision | `v0.1.3` | Locked Concept snapshot revision |
| Rust workspace | `0.1.0` | Rust implementation release version |
| Algorithm | `refetch.rank.baseline.v0.1` | Observable ranking algorithm identity |

A Host must not infer one axis from another. Persist the request, selected Lens,
FeedSlate, and implementation version together when replay or audit is needed.

## 3. Rust API

The in-process entry point is:

```rust
use refetch_contract::RankRequest;
use refetch_core::rank;

fn build_slate(json: &str) -> Result<refetch_contract::FeedSlate, Box<dyn std::error::Error>> {
    let request: RankRequest = serde_json::from_str(json)?;
    let slate = rank(&request)?;
    Ok(slate)
}
```

JSON deserialization errors and `RankError` are separate boundaries. Callers
should preserve both instead of converting every failure into an empty slate.

## 4. CLI boundary

```bash
refetch rank --input request.json --output slate.json
```

Behavior:

- success returns exit code `0` and writes one pretty-printed `FeedSlate`;
- malformed JSON, contract violations, semantic violations, and I/O errors
  return a non-zero exit code;
- parsing and ranking failures occur before the output is written;
- failure details are written to stderr;
- stderr is human-readable implementation output, not a cross-language error
  envelope or a compatibility-stable machine protocol.

Hosts must check the exit code before reading the output. They must not treat a
missing or stale output file as an empty successful result.

## 5. Evidence boundary

Evidence rules are part of conformance, not optional diagnostics:

- Evidence IDs are globally unique within one `RankRequest`.
- Candidate `source.*` Signals may reference only Evidence owned by that
  candidate.
- Analysis `analysis.*` Signals may reference the union of the corresponding
  candidate Evidence and AnalysisRecord Evidence.
- `clusterAssignment.evidenceRefs` uses the same union as Analysis Signals.
- every reference must exist and every reference list must be non-empty and
  duplicate-free.
- RankingReason copies the Signal, value, weight, contribution, and Evidence
  references that actually participated in scoring.

A Host or Analyzer must not invent explanation text after ranking as a
replacement for these structured references.

## 6. Determinism and replay

For the same valid request and locked implementation, Core must produce the
same serialized slate content. Ranking does not read the clock: `generatedAt`
is copied from `request.context.generatedAt`.

For replayable integration tests, freeze:

- the complete input JSON;
- the expected output JSON;
- the Concept lock commit;
- the Rust implementation version;
- any Adapter or Analyzer versions already recorded in the request.

## 7. Host responsibilities

The Host owns orchestration outside Core:

```text
source data
  -> Adapter normalization
  -> optional Analyzer enrichment
  -> RankRequest assembly and persistence
  -> Core invocation
  -> FeedSlate display and audit UI
```

Platform-specific behavior belongs in Adapter modules. Analyzer failure policy,
network consent, local storage, caching, and UI behavior belong in the Host.
Core must remain source-independent and executable without those components.

## 8. Integration checklist

Before connecting a Host or Feed Lab:

- [ ] Pin the exact Concept/Foundation revision consumed by the Host.
- [ ] Validate representative requests against the locked Schema.
- [ ] Confirm every candidate has exactly one corresponding analysis record.
- [ ] Confirm every score-affecting Signal is backed by valid Evidence.
- [ ] Exercise a successful request and compare the complete expected slate.
- [ ] Exercise malformed JSON, schema failure, and semantic failure paths.
- [ ] Check the CLI exit code before reading output, if using the CLI.
- [ ] Verify repeated runs produce identical results.
- [ ] Keep frozen product samples outside `tests/spec/v0.1/`.
- [ ] Record unresolved Analyzer or Adapter assumptions explicitly.

## 9. Current readiness

Appropriate now:

- Rust-level integration against a pinned Foundation revision;
- offline CLI execution over frozen JSON;
- preparation for an external frozen-sample experiment, currently PiliNara;
- a minimal Feed Lab that only consumes frozen inputs and Core outputs.

Not yet provided:

- published crates or a release/tag compatibility promise;
- live GitHub, RSS, Bilibili, or other source adapters;
- a stable machine-readable CLI error envelope;
- Flutter/Dart FFI, WASM, HTTP, streaming, or plugin APIs;
- AI Analyzer orchestration;
- evidence that the baseline ranking improves real user outcomes.

Product integration should advance only after current conformance passes from a
clean checkout and the release checklist identifies the exact supported
Foundation revision.

The current PiliNara product sequence is a Host-level experiment, not a change
to the source-independent Core contract. It also does not by itself satisfy the
locked Concept maintainer guardrail that calls for later GitHub and RSS
cross-source validation.
