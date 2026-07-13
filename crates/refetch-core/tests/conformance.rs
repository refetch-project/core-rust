use refetch_contract::*;
use refetch_core::*;
use std::{
    fs,
    path::{Path, PathBuf},
};
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn load<T: serde::de::DeserializeOwned>(p: &str) -> T {
    serde_json::from_str(&fs::read_to_string(root().join(p)).unwrap()).unwrap()
}
fn req(lens: &str) -> RankRequest {
    RankRequest {
        spec_version: "v0.1".into(),
        id: format!("request:{lens}"),
        candidates: load("tests/spec/v0.1/candidates.json"),
        analyses: load("tests/spec/v0.1/analyses.json"),
        lens: load(&format!("tests/spec/v0.1/lenses/{lens}.json")),
    }
}
#[test]
fn json_round_trip() {
    let r = req("production-readiness");
    let s = serde_json::to_string(&r).unwrap();
    let back: RankRequest = serde_json::from_str(&s).unwrap();
    assert_eq!(r, back);
}
#[test]
fn expected_orders() {
    for lens in [
        "production-readiness",
        "frontier-watch",
        "contribution-opportunities",
    ] {
        let actual = rank(&req(lens)).unwrap();
        let expected: FeedSlate = load(&format!("tests/spec/v0.1/expected-slates/{lens}.json"));
        let a: Vec<_> = actual.items.iter().map(|i| &i.candidate_id).collect();
        let e: Vec<_> = expected.items.iter().map(|i| &i.candidate_id).collect();
        assert_eq!(a, e);
    }
}
#[test]
fn stable_tie_breaker() {
    let mut r = req("production-readiness");
    r.lens.weights.clear();
    let slate = rank(&r).unwrap();
    let ids: Vec<_> = slate.items.iter().map(|i| i.candidate_id.clone()).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted[..ids.len()]);
}
#[test]
fn invalid_evidence_reference() {
    let mut r = req("production-readiness");
    r.candidates[0].evidence.clear();
    assert!(matches!(
        rank(&r),
        Err(CoreError::InvalidEvidenceReference(_))
    ));
}
#[test]
fn missing_analysis_record() {
    let mut r = req("production-readiness");
    let missing = r.analyses.pop().unwrap().candidate_id;
    let err = rank(&r).unwrap_err().to_string();
    assert!(err.contains(&missing));
}
#[test]
fn cluster_dedup() {
    let mut r = req("production-readiness");
    r.lens.weights.clear();
    r.lens.limits.max_items = 10;
    r.candidates[1].id = r.candidates[0].id.clone();
    r.analyses[1].candidate_id = r.candidates[0].id.clone();
    let slate = rank(&r).unwrap();
    assert!(slate.items.len() < r.candidates.len());
}
#[test]
fn repeatable() {
    let r = req("frontier-watch");
    assert_eq!(rank(&r).unwrap(), rank(&r).unwrap());
}
#[test]
fn spec_version_file_exists() {
    assert!(root().join("SPEC_VERSION").exists());
}
