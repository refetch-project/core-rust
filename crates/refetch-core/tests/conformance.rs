use refetch_contract::*;
use refetch_core::*;
use serde::Deserialize;
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const LOCKED_INVALID_FIXTURE_COUNT: usize = 15;

#[derive(Debug, Deserialize)]
struct InvalidFixture {
    #[serde(rename = "expectedError")]
    expected_error: String,
    request: serde_json::Value,
    #[serde(default)]
    slate: Option<FeedSlate>,
}

#[derive(Debug, PartialEq, Eq)]
struct InvalidFixtureRun {
    discovered: usize,
    executed: usize,
}

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

fn invalid_fixture_paths(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let entries = fs::read_dir(dir).map_err(|error| {
        format!(
            "failed to read invalid fixture directory {}: {error}",
            dir.display()
        )
    })?;
    let mut paths = Vec::new();
    for entry in entries {
        let path = entry
            .map_err(|error| {
                format!(
                    "failed to read an entry in invalid fixture directory {}: {error}",
                    dir.display()
                )
            })?
            .path();
        if path.extension() == Some(OsStr::new("json")) {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn fixture_error(expected_error: &str, actual_error: impl std::fmt::Display) -> String {
    format!("expectedError: {expected_error}\nactual error: {actual_error}")
}

fn parse_request(fixture: &InvalidFixture) -> Result<RankRequest, serde_json::Error> {
    serde_json::from_value(fixture.request.clone())
}

fn expect_rank_error(
    fixture: &InvalidFixture,
    expected_error: &str,
    error_matches: impl FnOnce(&RankRequest, &RankError) -> bool,
) -> Result<(), String> {
    let request = parse_request(fixture).map_err(|error| {
        fixture_error(
            expected_error,
            format!("request serde deserialization failed: {error}"),
        )
    })?;
    match rank(&request) {
        Err(error) if error_matches(&request, &error) => Ok(()),
        Err(error) => Err(fixture_error(
            expected_error,
            format!("unexpected RankError {error:?}: {error}"),
        )),
        Ok(slate) => Err(fixture_error(
            expected_error,
            format!("rank succeeded with slate {slate:?}"),
        )),
    }
}

fn expect_schema_error(fixture: &InvalidFixture) -> Result<(), String> {
    let request = match parse_request(fixture) {
        Ok(request) => request,
        Err(_) => return Ok(()),
    };
    match rank(&request) {
        Err(
            RankError::UnsupportedSpecVersion(_)
            | RankError::InvalidSignalNamespace { .. }
            | RankError::InvalidPolicy(_),
        ) => Ok(()),
        Err(error) => Err(fixture_error(
            "schema",
            format!("unexpected RankError {error:?}: {error}"),
        )),
        Ok(slate) => Err(fixture_error(
            "schema",
            format!("request passed serde and rank succeeded with slate {slate:?}"),
        )),
    }
}

fn rank_for_slate_comparison(
    fixture: &InvalidFixture,
    expected_error: &str,
) -> Result<(FeedSlate, FeedSlate), String> {
    let request = parse_request(fixture).map_err(|error| {
        fixture_error(
            expected_error,
            format!("request serde deserialization failed: {error}"),
        )
    })?;
    let actual = rank(&request).map_err(|error| {
        fixture_error(expected_error, format!("rank returned {error:?}: {error}"))
    })?;
    let expected = fixture.slate.clone().ok_or_else(|| {
        fixture_error(expected_error, "fixture did not declare the required slate")
    })?;
    Ok((actual, expected))
}

fn expect_score_mismatch(fixture: &InvalidFixture) -> Result<(), String> {
    let expected_error = "expectedScoreMismatch";
    let (actual, expected) = rank_for_slate_comparison(fixture, expected_error)?;
    let mut normalized = expected.clone();
    let mut score_mismatch_found = false;
    for (actual_item, expected_item) in actual.items.iter().zip(&mut normalized.items) {
        if actual_item.decision.score != expected_item.decision.score {
            score_mismatch_found = true;
        }
        expected_item.decision.score = actual_item.decision.score;
    }
    if !score_mismatch_found {
        return Err(fixture_error(
            expected_error,
            format!("no item score differed; actual slate was {actual:?}"),
        ));
    }
    if normalized != actual {
        return Err(fixture_error(
            expected_error,
            format!(
                "slate differed outside item decision.score; expected slate {expected:?}; actual slate {actual:?}"
            ),
        ));
    }
    Ok(())
}

fn expect_coverage_mismatch(fixture: &InvalidFixture) -> Result<(), String> {
    let expected_error = "coverageMismatch";
    let (actual, expected) = rank_for_slate_comparison(fixture, expected_error)?;
    if expected.coverage == actual.coverage {
        return Err(fixture_error(
            expected_error,
            format!("coverage did not differ; actual slate was {actual:?}"),
        ));
    }
    let mut normalized = expected.clone();
    normalized.coverage = actual.coverage.clone();
    if normalized != actual {
        return Err(fixture_error(
            expected_error,
            format!(
                "slate differed outside coverage; expected slate {expected:?}; actual slate {actual:?}"
            ),
        ));
    }
    Ok(())
}

fn execute_invalid_fixture(path: &Path, contents: &str) -> Result<(), String> {
    let fixture: InvalidFixture = serde_json::from_str(contents).map_err(|error| {
        format!(
            "fixture path: {}\nexpectedError: <unavailable>\nactual error: wrapper deserialization failed: {error}",
            path.display()
        )
    })?;
    let expected_error = fixture.expected_error.clone();
    let result = match expected_error.as_str() {
        "schema" => expect_schema_error(&fixture),
        "duplicateAnalysisId" => expect_rank_error(&fixture, &expected_error, |_, error| {
            matches!(
                error,
                RankError::DuplicateId {
                    kind: "analysis",
                    ..
                }
            )
        }),
        "duplicateCandidateId" => expect_rank_error(&fixture, &expected_error, |_, error| {
            matches!(
                error,
                RankError::DuplicateId {
                    kind: "candidate",
                    ..
                }
            )
        }),
        "duplicateEvidenceId" => expect_rank_error(&fixture, &expected_error, |_, error| {
            matches!(
                error,
                RankError::DuplicateId {
                    kind: "evidence",
                    ..
                }
            )
        }),
        "duplicateAnalysisSignalName" => {
            expect_rank_error(&fixture, &expected_error, |request, error| match error {
                RankError::DuplicateSignal { record, .. } => request
                    .analysis
                    .iter()
                    .any(|analysis| analysis.id == *record),
                _ => false,
            })
        }
        "duplicateCandidateSignalName" => {
            expect_rank_error(&fixture, &expected_error, |request, error| match error {
                RankError::DuplicateSignal { record, .. } => request
                    .candidates
                    .iter()
                    .any(|candidate| candidate.id == *record),
                _ => false,
            })
        }
        "analysisCandidateMissing" => expect_rank_error(&fixture, &expected_error, |_, error| {
            matches!(error, RankError::UnknownCandidate(_))
        }),
        "danglingEvidenceRef" => expect_rank_error(&fixture, &expected_error, |_, error| {
            matches!(error, RankError::DanglingEvidenceRef { .. })
        }),
        "expectedScoreMismatch" => expect_score_mismatch(&fixture),
        "coverageMismatch" => expect_coverage_mismatch(&fixture),
        _ => Err(fixture_error(
            &expected_error,
            format!("unknown expectedError {expected_error:?}"),
        )),
    };
    result.map_err(|error| format!("fixture path: {}\n{error}", path.display()))
}

fn run_invalid_fixtures(dir: &Path, expected_count: usize) -> Result<InvalidFixtureRun, String> {
    let paths = invalid_fixture_paths(dir)?;
    let discovered = paths.len();
    if discovered != expected_count {
        return Err(format!(
            "invalid fixtures discovered: {discovered}\ninvalid fixtures executed: 0\nexpected exactly {expected_count} invalid JSON fixtures in {}",
            dir.display()
        ));
    }
    let mut executed = 0;
    for path in paths {
        executed += 1;
        let contents = fs::read_to_string(&path).map_err(|error| {
            format!(
                "invalid fixtures discovered: {discovered}\ninvalid fixtures executed: {executed}\nfixture path: {}\nexpectedError: <unavailable>\nactual error: failed to read fixture: {error}",
                path.display()
            )
        })?;
        match execute_invalid_fixture(&path, &contents) {
            Ok(()) => {}
            Err(error) => {
                return Err(format!(
                    "invalid fixtures discovered: {discovered}\ninvalid fixtures executed: {executed}\n{error}"
                ));
            }
        }
    }
    Ok(InvalidFixtureRun {
        discovered,
        executed,
    })
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
    let run = match run_invalid_fixtures(&dir, LOCKED_INVALID_FIXTURE_COUNT) {
        Ok(run) => run,
        Err(error) => panic!("{error}"),
    };
    eprintln!(
        "invalid fixtures discovered: {}; invalid fixtures executed: {}",
        run.discovered, run.executed
    );
    assert_eq!(
        run,
        InvalidFixtureRun {
            discovered: LOCKED_INVALID_FIXTURE_COUNT,
            executed: LOCKED_INVALID_FIXTURE_COUNT,
        },
        "invalid fixture count or execution count did not match the locked snapshot"
    );
}

#[test]
fn invalid_fixture_parses_wrapped_request() {
    let request = req("production");
    let wrapped = serde_json::json!({
        "expectedError": "schema",
        "request": request,
    });
    let fixture: InvalidFixture = serde_json::from_value(wrapped).unwrap();
    let parsed = parse_request(&fixture).unwrap();
    assert_eq!(parsed.id, "req:foundation-v012");
}

#[test]
fn empty_invalid_fixture_directory_fails() {
    let dir = std::env::temp_dir().join(format!(
        "refetch-core-empty-invalid-fixtures-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    fs::create_dir(&dir).unwrap();
    let result = run_invalid_fixtures(&dir, LOCKED_INVALID_FIXTURE_COUNT);
    fs::remove_dir(&dir).unwrap();
    let error = match result {
        Ok(run) => panic!("empty invalid fixture directory unexpectedly passed: {run:?}"),
        Err(error) => error,
    };
    assert!(error.contains("invalid fixtures discovered: 0"), "{error}");
    assert!(error.contains("invalid fixtures executed: 0"), "{error}");
}

#[test]
fn unknown_expected_error_fails() {
    let contents = serde_json::json!({
        "expectedError": "futureUnknownError",
        "request": req("production"),
    })
    .to_string();
    let path = Path::new("/synthetic/unknown-expected-error.json");
    let error = match execute_invalid_fixture(path, &contents) {
        Ok(()) => panic!("unknown expectedError unexpectedly passed"),
        Err(error) => error,
    };
    assert!(error.contains(&path.display().to_string()), "{error}");
    assert!(
        error.contains("expectedError: futureUnknownError"),
        "{error}"
    );
    assert!(
        error.contains("actual error: unknown expectedError"),
        "{error}"
    );
}

#[test]
fn unexpectedly_valid_invalid_fixture_fails() {
    let contents = serde_json::json!({
        "expectedError": "duplicateCandidateId",
        "request": req("production"),
    })
    .to_string();
    let path = Path::new("/synthetic/unexpectedly-valid-invalid-fixture.json");
    let error = match execute_invalid_fixture(path, &contents) {
        Ok(()) => panic!("unexpectedly valid invalid fixture passed"),
        Err(error) => error,
    };
    assert!(error.contains(&path.display().to_string()), "{error}");
    assert!(
        error.contains("expectedError: duplicateCandidateId"),
        "{error}"
    );
    assert!(error.contains("actual error: rank succeeded"), "{error}");
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
