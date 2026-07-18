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
    let (negative, unsigned) = match s.strip_prefix('-') {
        Some(unsigned) => (true, unsigned),
        None => (false, s),
    };
    let (mantissa, exponent) = match unsigned.find(['e', 'E']) {
        Some(index) => {
            let exponent = unsigned[index + 1..]
                .parse::<i32>()
                .map_err(|_| "invalid decimal exponent")?;
            (&unsigned[..index], exponent)
        }
        None => (unsigned, 0),
    };

    let mut digits = 0i128;
    let mut decimal_places = 0usize;
    let mut decimal_point_seen = false;
    let mut digit_seen = false;
    for byte in mantissa.bytes() {
        match byte {
            b'0'..=b'9' => {
                digit_seen = true;
                digits = digits
                    .checked_mul(10)
                    .and_then(|value| value.checked_add(i128::from(byte - b'0')))
                    .ok_or("fixed6 overflow")?;
                if decimal_point_seen {
                    decimal_places += 1;
                }
            }
            b'.' if !decimal_point_seen => decimal_point_seen = true,
            _ => return Err("invalid decimal".into()),
        }
    }
    if !digit_seen {
        return Err("invalid decimal".into());
    }
    if digits == 0 {
        return Ok(Fixed6::ZERO);
    }

    let scale_power = i64::from(exponent) + 6 - decimal_places as i64;
    let scaled = if scale_power >= 0 {
        digits
            .checked_mul(checked_pow10(scale_power as u32).ok_or("fixed6 overflow")?)
            .ok_or("fixed6 overflow")?
    } else {
        let divisor = checked_pow10((-scale_power) as u32).ok_or("more than six decimals")?;
        if digits % divisor != 0 {
            return Err("more than six decimals".into());
        }
        digits / divisor
    };
    let signed = if negative { -scaled } else { scaled };
    i64::try_from(signed)
        .map(Fixed6)
        .map_err(|_| "fixed6 overflow".into())
}

fn checked_pow10(exponent: u32) -> Option<i128> {
    let mut value = 1i128;
    for _ in 0..exponent {
        value = value.checked_mul(10)?;
    }
    Some(value)
}

pub type Extensions = BTreeMap<String, Value>;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub name: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct Signal {
    pub name: String,
    pub value: Fixed6,
    #[serde(rename = "evidenceRefs")]
    pub evidence_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ClusterAssignment {
    pub namespace: String,
    pub id: String,
    #[serde(rename = "evidenceRefs")]
    pub evidence_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Source {
    #[serde(rename = "type")]
    pub source_type: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct Trigger {
    #[serde(rename = "type")]
    pub trigger_type: String,
    #[serde(rename = "observedAt")]
    pub observed_at: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct Context {
    #[serde(rename = "generatedAt")]
    pub generated_at: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct RankingReason {
    pub signal: String,
    pub value: Fixed6,
    pub weight: Fixed6,
    pub contribution: Fixed6,
    #[serde(rename = "evidenceRefs")]
    pub evidence_refs: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RankingDecision {
    pub rank: usize,
    pub score: Fixed6,
    pub reasons: Vec<RankingReason>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FeedSlateItem {
    #[serde(rename = "candidateId")]
    pub candidate_id: String,
    pub decision: RankingDecision,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Coverage {
    #[serde(rename = "bySourceType")]
    pub by_source_type: BTreeMap<String, usize>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: Extensions,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed(json: &str) -> Result<Fixed6, serde_json::Error> {
        serde_json::from_str(json)
    }

    #[test]
    fn fixed6_accepts_exact_six_decimal_values_and_exponents() {
        assert_eq!(fixed("0.000001").unwrap().raw(), 1);
        assert_eq!(fixed("1e-6").unwrap().raw(), 1);
        assert_eq!(fixed("-1e-6").unwrap().raw(), -1);
        assert_eq!(fixed("1.234567").unwrap().raw(), 1_234_567);
        assert_eq!(fixed("123e-2").unwrap().raw(), 1_230_000);
    }

    #[test]
    fn fixed6_rejects_values_beyond_six_decimal_places() {
        for json in ["0.0000001", "1e-7", "-1e-7"] {
            let error = fixed(json).unwrap_err().to_string();
            assert!(error.contains("more than six decimals"), "{json}: {error}");
        }
    }

    #[test]
    fn fixed6_smallest_value_round_trips() {
        let value = fixed("0.000001").unwrap();
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(fixed(&json).unwrap(), value, "serialized as {json}");
    }

    #[test]
    fn contract_objects_reject_unknown_fields() {
        let error = serde_json::from_str::<Component>(
            r#"{"name":"fixture-adapter","version":"0.1.0","unexpected":true}"#,
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("unknown field `unexpected`"), "{error}");
    }
}
