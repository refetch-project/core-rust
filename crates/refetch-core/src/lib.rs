use refetch_contract::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("missing analysis for candidate {0}")]
    MissingAnalysis(String),
    #[error("invalid evidence reference {0}")]
    InvalidEvidenceReference(String),
}

pub fn rank(request: &RankRequest) -> Result<FeedSlate, CoreError> {
    let analyses: BTreeMap<_, _> = request
        .analyses
        .iter()
        .map(|a| (a.candidate_id.as_str(), a))
        .collect();
    let evidence: BTreeSet<_> = request
        .candidates
        .iter()
        .flat_map(|c| c.evidence.iter().map(|e| e.id.as_str()))
        .chain(
            request
                .analyses
                .iter()
                .flat_map(|a| a.evidence.iter().map(|e| e.id.as_str())),
        )
        .collect();
    let mut scored = Vec::new();
    for c in &request.candidates {
        if !request.lens.filters.source_types.is_empty()
            && !request.lens.filters.source_types.contains(&c.source.kind)
        {
            continue;
        }
        let a = analyses
            .get(c.id.as_str())
            .ok_or_else(|| CoreError::MissingAnalysis(c.id.clone()))?;
        let mut score = 0.0;
        let mut reasons = Vec::new();
        for (feature, weight) in &request.lens.weights {
            let contribution = a.signals.get(feature).copied().unwrap_or(0.0) * weight;
            score += contribution;
            if contribution > 0.0 {
                let refs = vec![
                    format!("ev:{}:meta", c.id),
                    format!("ev:analysis:{}:topics", c.id),
                ];
                for r in &refs {
                    if !evidence.contains(r.as_str()) {
                        return Err(CoreError::InvalidEvidenceReference(r.clone()));
                    }
                }
                reasons.push(RankingReason {
                    code: "FEATURE_MATCH".into(),
                    feature: feature.clone(),
                    contribution: round6(contribution),
                    evidence_refs: refs,
                });
            }
        }
        let evidence_refs = reasons
            .iter()
            .flat_map(|r| r.evidence_refs.clone())
            .collect();
        scored.push((
            round6(score),
            c.id.clone(),
            RankingDecision {
                spec_version: SPEC_VERSION.into(),
                candidate_id: c.id.clone(),
                lens_id: request.lens.id.clone(),
                eligible: true,
                rank: 0,
                score: round6(score),
                reasons,
                evidence_refs,
                extensions: Default::default(),
            },
        ));
    }
    scored.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    let mut seen_clusters = BTreeSet::new();
    let mut items = Vec::new();
    for (_, cid, mut d) in scored {
        let cluster = cid.clone();
        if request.lens.limits.max_per_cluster == 1 && !seen_clusters.insert(cluster) {
            continue;
        }
        d.rank = items.len() + 1;
        items.push(SlateItem {
            candidate_id: cid,
            rank: d.rank,
            decision: d,
        });
        if items.len() >= request.lens.limits.max_items {
            break;
        }
    }
    Ok(FeedSlate {
        spec_version: SPEC_VERSION.into(),
        id: format!("slate:{}", request.lens.id),
        lens_id: request.lens.id.clone(),
        generated_at: "2026-01-15T00:00:00Z".into(),
        engine: Engine {
            name: "refetch-core".into(),
            version: env!("CARGO_PKG_VERSION").into(),
        },
        diversity: serde_json::json!({"clusters": items.iter().map(|i| &i.candidate_id).collect::<Vec<_>>() }),
        coverage: serde_json::json!({"sourceTypes": request.lens.filters.source_types}),
        exploration: serde_json::json!({"budget": request.lens.policy.exploration_budget, "used": 0}),
        items,
        extensions: Default::default(),
    })
}
fn round6(v: f64) -> f64 {
    (v * 1_000_000.0).round() / 1_000_000.0
}
