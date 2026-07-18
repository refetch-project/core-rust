use refetch_contract::*;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use thiserror::Error;

mod validation;
pub use validation::validate;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RankError {
    #[error("unsupported spec version: {0}")]
    UnsupportedSpecVersion(String),
    #[error("duplicate id in {kind}: {id}")]
    DuplicateId { kind: &'static str, id: String },
    #[error("missing analysis for candidate: {0}")]
    MissingAnalysis(String),
    #[error("analysis references unknown candidate: {0}")]
    UnknownCandidate(String),
    #[error("invalid signal namespace in {record}: {signal}")]
    InvalidSignalNamespace { record: String, signal: String },
    #[error("duplicate signal in {record}: {signal}")]
    DuplicateSignal { record: String, signal: String },
    #[error("dangling evidence ref in {record}: {evidence_ref}")]
    DanglingEvidenceRef {
        record: String,
        evidence_ref: String,
    },
    #[error("invalid policy: {0}")]
    InvalidPolicy(String),
    #[error("schema violation at {path}: {message}")]
    SchemaViolation { path: String, message: String },
    #[error("arithmetic overflow")]
    ArithmeticOverflow,
}

pub fn rank(request: &RankRequest) -> Result<FeedSlate, RankError> {
    validate(request)?;
    let analyses: HashMap<_, _> = request
        .analysis
        .iter()
        .map(|a| (a.candidate_id.as_str(), a))
        .collect();
    let allowed: Option<BTreeSet<_>> = request
        .lens
        .allowed_source_types
        .as_ref()
        .map(|v| v.iter().cloned().collect());
    let mut scored = Vec::new();
    for cand in &request.candidates {
        if let Some(allowed) = &allowed {
            if !allowed.contains(&cand.source.source_type) {
                continue;
            }
        }
        let analysis = analyses
            .get(cand.id.as_str())
            .ok_or_else(|| RankError::MissingAnalysis(cand.id.clone()))?;
        let mut reasons = Vec::new();
        let mut score = Fixed6::ZERO;
        for sig in cand.signals.iter().chain(analysis.signals.iter()) {
            let Some(weight) = request.lens.weights.get(&sig.name) else {
                continue;
            };
            let contribution = sig
                .value
                .checked_mul(*weight)
                .ok_or(RankError::ArithmeticOverflow)?;
            if contribution.is_zero() {
                continue;
            }
            score = score
                .checked_add(contribution)
                .ok_or(RankError::ArithmeticOverflow)?;
            reasons.push(RankingReason {
                signal: sig.name.clone(),
                value: sig.value,
                weight: *weight,
                contribution,
                evidence_refs: sig.evidence_refs.clone(),
            });
        }
        scored.push(Scored {
            candidate: cand,
            analysis,
            score,
            reasons,
        });
    }
    scored.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.candidate.id.cmp(&b.candidate.id))
    });
    let mut items = Vec::new();
    let mut cluster_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut suppressed = 0usize;
    for row in scored {
        if items.len() >= request.lens.policy.max_items {
            break;
        }
        let key = row.analysis.cluster_assignment.as_ref().map(cluster_key);
        if let Some(k) = &key {
            if *cluster_counts.get(k).unwrap_or(&0) >= request.lens.policy.max_per_cluster {
                suppressed += 1;
                continue;
            }
        }
        let rank = items.len() + 1;
        if let Some(k) = key {
            *cluster_counts.entry(k).or_insert(0) += 1;
        }
        items.push(FeedSlateItem {
            candidate_id: row.candidate.id.clone(),
            decision: RankingDecision {
                rank,
                score: row.score,
                reasons: row.reasons,
            },
        });
    }
    let cand_by_id: HashMap<_, _> = request
        .candidates
        .iter()
        .map(|c| (c.id.as_str(), c))
        .collect();
    let mut coverage = BTreeMap::new();
    let mut unclustered = 0usize;
    for item in &items {
        let cand = cand_by_id[item.candidate_id.as_str()];
        *coverage.entry(cand.source.source_type.clone()).or_insert(0) += 1;
        let analysis = analyses[item.candidate_id.as_str()];
        if analysis.cluster_assignment.is_none() {
            unclustered += 1;
        }
    }
    Ok(FeedSlate {
        spec_version: SPEC_VERSION.into(),
        request_id: request.id.clone(),
        lens_id: request.lens.id.clone(),
        generated_at: request.context.generated_at.clone(),
        algorithm_id: ALGORITHM_ID.into(),
        items,
        coverage: Coverage {
            by_source_type: coverage,
            extensions: BTreeMap::new(),
        },
        diversity: Diversity {
            clusters_selected: cluster_counts,
            unclustered_selected: unclustered,
            suppressed_by_cluster_limit: suppressed,
            extensions: BTreeMap::new(),
        },
        extensions: BTreeMap::new(),
    })
}
struct Scored<'a> {
    candidate: &'a FeedCandidate,
    analysis: &'a AnalysisRecord,
    score: Fixed6,
    reasons: Vec<RankingReason>,
}
fn cluster_key(c: &ClusterAssignment) -> String {
    format!("{}:{}", c.namespace, c.id)
}
