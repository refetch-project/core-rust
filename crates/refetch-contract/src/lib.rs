use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SPEC_VERSION: &str = "v0.1";
pub type Extensions = BTreeMap<String, serde_json::Value>;

#[derive(Debug, thiserror::Error)]
pub enum ContractError {
    #[error("unsupported spec version: {0}")]
    UnsupportedSpecVersion(String),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FeedCandidate {
    pub spec_version: String,
    pub id: String,
    pub subject: Subject,
    pub trigger: Trigger,
    pub source: Source,
    pub provenance: Provenance,
    #[serde(default)]
    pub evidence: Vec<Evidence>,
    #[serde(default)]
    pub source_signals: BTreeMap<String, f64>,
    #[serde(default)]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Subject {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub title: String,
    pub url: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Trigger {
    #[serde(rename = "type")]
    pub kind: String,
    pub occurred_at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    #[serde(rename = "type")]
    pub kind: String,
    pub name: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Provenance {
    pub captured_at: String,
    pub adapter: String,
    pub adapter_version: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Evidence {
    pub id: String,
    pub kind: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisRecord {
    pub spec_version: String,
    pub id: String,
    pub candidate_id: String,
    pub analyzer: Analyzer,
    pub summary: String,
    #[serde(default)]
    pub topics: Vec<String>,
    #[serde(default)]
    pub signals: BTreeMap<String, f64>,
    pub confidence: f64,
    #[serde(default)]
    pub evidence: Vec<Evidence>,
    pub created_at: String,
    #[serde(default)]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Analyzer {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LensProfile {
    pub spec_version: String,
    pub id: String,
    pub title: String,
    pub task: String,
    #[serde(default)]
    pub weights: BTreeMap<String, f64>,
    pub filters: Filters,
    pub limits: Limits,
    pub policy: Policy,
    #[serde(default)]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Filters {
    #[serde(default)]
    pub source_types: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Limits {
    pub max_items: usize,
    pub max_per_cluster: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Policy {
    pub tie_breaker: String,
    pub exploration_budget: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RankingReason {
    pub code: String,
    pub feature: String,
    pub contribution: f64,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RankingDecision {
    pub spec_version: String,
    pub candidate_id: String,
    pub lens_id: String,
    pub eligible: bool,
    pub rank: usize,
    pub score: f64,
    #[serde(default)]
    pub reasons: Vec<RankingReason>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FeedSlate {
    pub spec_version: String,
    pub id: String,
    pub lens_id: String,
    pub generated_at: String,
    pub engine: Engine,
    pub items: Vec<SlateItem>,
    pub diversity: serde_json::Value,
    pub coverage: serde_json::Value,
    pub exploration: serde_json::Value,
    #[serde(default)]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Engine {
    pub name: String,
    pub version: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SlateItem {
    pub candidate_id: String,
    pub rank: usize,
    pub decision: RankingDecision,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RankRequest {
    pub spec_version: String,
    pub id: String,
    pub candidates: Vec<FeedCandidate>,
    pub analyses: Vec<AnalysisRecord>,
    pub lens: LensProfile,
}
