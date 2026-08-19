# RFC 0001: Semantic feed contract

Status: Draft

## 1. Problem statement

Semantic feeds need portable data contracts so ranking implementations can explain why an item appeared without inventing evidence, platform-specific assumptions, or metrics.

## 2. Goals

Define the v0.1 executable contract for candidates, analysis, lenses, rank requests, feed slates, evidence-backed contributions, deterministic decimal scoring, cluster-constrained selection, and typed metrics.

## 3. Non-goals

RFC 0001 does not specify App Semantic Contract, MCP, AG-UI, A2UI, Bilibili, PiliPlus, Flutter UI, model calls, network crawling, cloud sync, accounts, or telemetry.

## 4. Core terms

The normative terms are FeedCandidate, Subject, Trigger, Source, Provenance, Evidence, AnalysisRecord, Signal, LensProfile, Policy, Context, ClusterAssignment, RankingReason, RankingDecision, FeedSlate, Adapter, Analyzer, and Host as defined in `docs/terminology.md`.

## 5. Normative data flow

```text
validate
→ eligibility/filtering
→ feature contributions
→ deterministic ordering
→ cluster-constrained selection
→ typed slate metrics
```

## 6. Input objects

A `RankRequest` contains `specVersion: "v0.1"`, a stable `id`, `context.generatedAt`, candidates, one analysis record per candidate, and one LensProfile. `FeedCandidate.subject.id` identifies the stable subject separately from candidate and trigger identity. Source, subject, trigger, evidence kind, and allowed source types are extensible lowercase tokens rather than platform enums. GitHub and RSS are fixture examples only.

Candidate provenance includes `adapter.name` and `adapter.version`. AnalysisRecord includes `analyzer.name`, `analyzer.version`, and `createdAt`.

## 7. Output objects

A `FeedSlate` contains `specVersion`, `requestId` copied from `RankRequest.id`, `lensId` copied from `LensProfile.id`, `generatedAt` copied from request context, the shared `algorithmId` `refetch.rank.baseline.v0.1`, selected items with a single rank location in `decision.rank`, and typed `coverage` and `diversity`. Rust crate versions or other implementation audit data must not change cross-language conformance JSON; they belong in explicit extensions or host audit records.

## 8. Signal and Evidence rules

Every score-affecting Signal contains `value` and non-empty `evidenceRefs`. Implementations must not guess Evidence IDs from candidate IDs. Evidence IDs are globally unique within one RankRequest. Candidate signals must use `source.*` and may reference only their own `FeedCandidate.evidence`. AnalysisRecord signals must use `analysis.*` and may reference the union of the corresponding `FeedCandidate.evidence` and `AnalysisRecord.evidence`. `clusterAssignment.evidenceRefs` follows the same union rule as analysis signals. Signal names are unique within a candidate or analysis record.

## 9. Lens weights, decimal scoring, and feature contributions

Signal values and Lens weights are interpreted as decimal fixed-point values with at most six fractional digits for conformance calculations. For each candidate, implementations match Lens weights to signal names. If a Lens has a weight for a missing signal, the contribution is zero and no reason is generated. If a signal has no corresponding Lens weight, it does not participate in scoring.

Contribution is `signal.value * weight`, rounded to six fractional digits using half-away-from-zero. `-0` is normalized to `0`. Candidate score is the sum of rounded contributions, then rounded to six fractional digits using the same mode. Every non-zero contribution becomes a RankingReason with signal name, value, weight, contribution, and evidence refs copied directly from that signal. Positive and negative contributions are both recorded. Reasons are ordered by source signal order followed by analysis signal order.

## 10. Filtering, sorting, and selection

Eligibility is limited to candidates whose source type is allowed by the Lens. Implementations compute every eligible candidate score before selection, sort by descending score and then ascending candidate id, and then traverse that ordered list. v0.1 supports only `candidateIdAsc` as the tie breaker.

Selection proceeds in order: if a candidate has a cluster whose count already equals `maxPerCluster`, skip it and increment `suppressedByClusterLimit`; otherwise select it. Stop as soon as `maxItems` items have been selected. `suppressedByClusterLimit` only counts candidates actually traversed before the slate fills. Ranks start at 1, are continuous, and items must not repeat.

## 11. Cluster and per-cluster limit

v0.1 supports at most one optional `clusterAssignment` per AnalysisRecord. Multi-dimensional clusters are future work. Cluster membership is only established by explicit ClusterAssignment. The cluster key is `namespace:id`. Unassigned candidates are independent and do not suppress each other. `maxPerCluster` applies to every integer value greater than or equal to 1.

## 12. Coverage and Diversity metrics

Coverage reports actual selected item counts by source type and only includes positive counts. Diversity is derived from the final selection process: selected cluster counts by `namespace:id`, selected unclustered count, and the number of traversed candidates skipped by cluster limits before the slate filled.

## 13. Version compatibility

v0.1 fixtures and schemas require exact `specVersion: "v0.1"`. Additive data must use explicit `extensions`; normal unknown fields are rejected.

## 14. Privacy and local-first boundary

The contract is executable without network access, model calls, databases, or telemetry. Hosts may run adapters or analyzers separately, but ranking consumes already supplied JSON.

## 15. Verifiable assumptions

The fixtures verify schema validity, reference integrity, deterministic decimal scoring, deterministic tie breaking, positive and negative reasons, fixed time propagation, request/lens propagation, source filtering, cluster limits, coverage, diversity, and three distinct Lens outputs over one shared candidate and analysis pool.

## 16. Unverified assumptions

Synthetic fixtures do not prove user value, real-world source quality, analyzer accuracy, or the best ranking formula.

## 17. Future work

Future RFCs may define exploration, multi-dimensional clusters, live adapters, analyzer protocols, UI bindings, or broader semantic app contracts after they have deterministic, testable semantics.
