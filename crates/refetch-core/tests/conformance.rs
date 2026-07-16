use refetch_contract::*;
use refetch_core::*;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn load<T: serde::de::DeserializeOwned>(p: &str) -> T {
    serde_json::from_str(&fs::read_to_string(root().join(p)).unwrap()).unwrap()
}
fn req(name: &str) -> RankRequest {
    load(&format!(
        "tests/spec/v0.1/fixtures/v0.1/valid/{name}.rank-request.json"
    ))
}
fn expected(name: &str) -> FeedSlate {
    load(&format!(
        "tests/spec/v0.1/fixtures/v0.1/expected/{name}.feed-slate.json"
    ))
}
#[test]
fn full_expected_slates() {
    for name in ["production", "frontier", "maintenance"] {
        assert_eq!(rank(&req(name)).unwrap(), expected(name));
    }
}
#[test]
fn json_round_trip() {
    let r = req("production");
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(r, serde_json::from_str(&s).unwrap());
}
#[test]
fn repeatable() {
    let r = req("frontier");
    assert_eq!(rank(&r).unwrap(), rank(&r).unwrap());
}
#[test]
fn snapshot_manifest_valid() {
    let out = Command::new("python3")
        .arg("scripts/verify-spec-snapshot.py")
        .current_dir(root())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
#[test]
fn max_per_cluster_two_and_unclustered() {
    let mut r = req("production");
    r.lens.policy.max_per_cluster = 2;
    r.lens.policy.max_items = 6;
    let slate = rank(&r).unwrap();
    assert!(slate.diversity.clusters_selected.values().any(|v| *v == 2));
    assert!(slate.diversity.unclustered_selected > 0);
}
#[test]
fn stable_tie_breaker() {
    let mut r = req("production");
    for w in r.lens.weights.values_mut() {
        *w = Fixed6::ZERO;
    }
    r.lens.policy.max_items = 10;
    let slate = rank(&r).unwrap();
    let ids: Vec<_> = slate.items.iter().map(|i| i.candidate_id.clone()).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted);
}
#[test]
fn invalid_fixtures_fail() {
    let dir = root().join("tests/spec/v0.1/fixtures/v0.1/invalid");
    for entry in fs::read_dir(dir).unwrap() {
        let p = entry.unwrap().path();
        let r: Result<RankRequest, _> = serde_json::from_str(&fs::read_to_string(&p).unwrap());
        if let Ok(r) = r {
            assert!(rank(&r).is_err(), "{} unexpectedly ranked", p.display());
        }
    }
}
#[test]
fn explicit_validation_errors() {
    let mut r = req("production");
    r.spec_version = "v9".into();
    assert!(matches!(
        rank(&r),
        Err(RankError::UnsupportedSpecVersion(_))
    ));
    let mut r = req("production");
    r.candidates[1].id = r.candidates[0].id.clone();
    assert!(matches!(
        rank(&r),
        Err(RankError::DuplicateId {
            kind: "candidate",
            ..
        })
    ));
    let mut r = req("production");
    let sig = r.candidates[0].signals[0].clone();
    r.candidates[0].signals.push(sig);
    assert!(matches!(rank(&r), Err(RankError::DuplicateSignal { .. })));
    let mut r = req("production");
    let ev = r.candidates[0].evidence[0].clone();
    r.candidates[0].evidence.push(ev);
    assert!(matches!(
        rank(&r),
        Err(RankError::DuplicateId {
            kind: "evidence",
            ..
        })
    ));
    let mut r = req("production");
    r.candidates[0].signals[0].name = "analysis.bad".into();
    assert!(matches!(
        rank(&r),
        Err(RankError::InvalidSignalNamespace { .. })
    ));
}
#[test]
fn slate_contract_fields_and_reasons() {
    let slate = rank(&req("production")).unwrap();
    assert_eq!(slate.request_id, "req:foundation-v012");
    assert_eq!(slate.lens_id, "lens:production");
    assert_eq!(slate.generated_at, "2026-01-15T00:00:00Z");
    assert_eq!(slate.algorithm_id, ALGORITHM_ID);
    assert!(slate
        .items
        .iter()
        .flat_map(|i| &i.decision.reasons)
        .any(|r| r.contribution.raw() < 0));
    assert!(slate
        .items
        .iter()
        .flat_map(|i| &i.decision.reasons)
        .any(|r| r.contribution.raw() > 0));
}
