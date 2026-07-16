use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::{collections::BTreeMap, fmt};

pub const SPEC_VERSION: &str = "v0.1";
pub const ALGORITHM_ID: &str = "refetch.rank.baseline.v0.1";
const SCALE: i64 = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Fixed6(i64);
impl Fixed6 {
    pub const ZERO: Self = Self(0);
    pub fn raw(self) -> i64 {
        self.0
    }
    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }
    pub fn checked_mul(self, other: Self) -> Option<Self> {
        let prod = (self.0 as i128).checked_mul(other.0 as i128)?;
        let q = prod / SCALE as i128;
        let r = prod % SCALE as i128;
        let abs_r = r.abs();
        let mut q = q;
        if abs_r * 2 >= SCALE as i128 {
            q += if prod >= 0 { 1 } else { -1 };
        }
        i64::try_from(q)
            .ok()
            .map(Self)
            .map(|v| if v.0 == 0 { Self::ZERO } else { v })
    }
    pub fn is_zero(self) -> bool {
        self.0 == 0
    }
}
impl fmt::Display for Fixed6 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            return write!(f, "0");
        }
        let sign = if self.0 < 0 { "-" } else { "" };
        let abs = self.0.abs();
        let int = abs / SCALE;
        let frac = abs % SCALE;
        if frac == 0 {
            write!(f, "{sign}{int}")
        } else {
            let mut s = format!("{frac:06}");
            while s.ends_with('0') {
                s.pop();
            }
            write!(f, "{sign}{int}.{s}")
        }
    }
}
impl Serialize for Fixed6 {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_f64(
            self.to_string()
                .parse::<f64>()
                .map_err(serde::ser::Error::custom)?,
        )
    }
}
impl<'de> Deserialize<'de> for Fixed6 {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(d)?;
        match value {
            Value::Number(n) => parse_fixed6(&n.to_string()).map_err(de::Error::custom),
            _ => Err(de::Error::custom("expected JSON number")),
        }
    }
}

fn parse_fixed6(s: &str) -> Result<Fixed6, String> {
    let neg = s.starts_with('-');
    let t = s.trim_start_matches('-');
    let parts: Vec<_> = t.split('.').collect();
    if parts.len() > 2 {
        return Err("invalid decimal".into());
    }
    let int: i64 = parts[0].parse().map_err(|_| "invalid integer")?;
    let frac = if parts.len() == 2 {
        if parts[1].len() > 6 {
            return Err("more than six decimals".into());
        }
        let mut fs = parts[1].to_string();
        while fs.len() < 6 {
            fs.push('0')
        }
        fs.parse::<i64>().map_err(|_| "invalid fraction")?
    } else {
        0
    };
    let raw = int
        .checked_mul(SCALE)
        .and_then(|x| x.checked_add(frac))
        .ok_or("fixed6 overflow")?;
    Ok(Fixed6(if neg { -raw } else { raw }))
}

pub type Extensions = BTreeMap<String, Value>;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Component {
    pub name: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Evidence {
    pub id: String,
    pub kind: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Signal {
    pub name: String,
    pub value: Fixed6,
    #[serde(rename = "evidenceRefs")]
    pub evidence_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClusterAssignment {
    pub namespace: String,
    pub id: String,
    #[serde(rename = "evidenceRefs")]
    pub evidence_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Source {
    #[serde(rename = "type")]
    pub source_type: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Subject {
    pub id: String,
    #[serde(rename = "type")]
    pub subject_type: String,
    pub title: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Trigger {
    #[serde(rename = "type")]
    pub trigger_type: String,
    #[serde(rename = "observedAt")]
    pub observed_at: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Provenance {
    #[serde(rename = "retrievedAt")]
    pub retrieved_at: String,
    pub adapter: Component,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeedCandidate {
    #[serde(rename = "specVersion")]
    pub spec_version: String,
    pub id: String,
    pub source: Source,
    pub subject: Subject,
    pub trigger: Trigger,
    pub provenance: Provenance,
    pub evidence: Vec<Evidence>,
    pub signals: Vec<Signal>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalysisRecord {
    #[serde(rename = "specVersion")]
    pub spec_version: String,
    pub id: String,
    #[serde(rename = "candidateId")]
    pub candidate_id: String,
    pub analyzer: Component,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    pub evidence: Vec<Evidence>,
    pub signals: Vec<Signal>,
    #[serde(rename = "clusterAssignment", skip_serializing_if = "Option::is_none")]
    pub cluster_assignment: Option<ClusterAssignment>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Policy {
    #[serde(rename = "maxItems")]
    pub max_items: usize,
    #[serde(rename = "maxPerCluster")]
    pub max_per_cluster: usize,
    #[serde(rename = "tieBreaker")]
    pub tie_breaker: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LensProfile {
    #[serde(rename = "specVersion")]
    pub spec_version: String,
    pub id: String,
    pub title: String,
    #[serde(rename = "allowedSourceTypes", skip_serializing_if = "Option::is_none")]
    pub allowed_source_types: Option<Vec<String>>,
    pub weights: BTreeMap<String, Fixed6>,
    pub policy: Policy,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Context {
    #[serde(rename = "generatedAt")]
    pub generated_at: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RankRequest {
    #[serde(rename = "specVersion")]
    pub spec_version: String,
    pub id: String,
    pub context: Context,
    pub candidates: Vec<FeedCandidate>,
    pub analysis: Vec<AnalysisRecord>,
    pub lens: LensProfile,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RankingReason {
    pub signal: String,
    pub value: Fixed6,
    pub weight: Fixed6,
    pub contribution: Fixed6,
    #[serde(rename = "evidenceRefs")]
    pub evidence_refs: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RankingDecision {
    pub rank: usize,
    pub score: Fixed6,
    pub reasons: Vec<RankingReason>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeedSlateItem {
    #[serde(rename = "candidateId")]
    pub candidate_id: String,
    pub decision: RankingDecision,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Coverage {
    #[serde(rename = "bySourceType")]
    pub by_source_type: BTreeMap<String, usize>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Diversity {
    #[serde(rename = "clustersSelected")]
    pub clusters_selected: BTreeMap<String, usize>,
    #[serde(rename = "unclusteredSelected")]
    pub unclustered_selected: usize,
    #[serde(rename = "suppressedByClusterLimit")]
    pub suppressed_by_cluster_limit: usize,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeedSlate {
    #[serde(rename = "specVersion")]
    pub spec_version: String,
    #[serde(rename = "requestId")]
    pub request_id: String,
    #[serde(rename = "lensId")]
    pub lens_id: String,
    #[serde(rename = "generatedAt")]
    pub generated_at: String,
    #[serde(rename = "algorithmId")]
    pub algorithm_id: String,
    pub items: Vec<FeedSlateItem>,
    pub coverage: Coverage,
    pub diversity: Diversity,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: Extensions,
}
