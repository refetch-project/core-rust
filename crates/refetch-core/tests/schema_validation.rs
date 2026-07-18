use refetch_contract::{FeedSlate, Fixed6, RankRequest};
use refetch_core::{rank, RankError};
use serde_json::{json, Value};
use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn request_json() -> Value {
    serde_json::from_str(
        &fs::read_to_string(
            root().join("tests/spec/v0.1/fixtures/v0.1/valid/production.rank-request.json"),
        )
        .unwrap(),
    )
    .unwrap()
}

fn request() -> RankRequest {
    serde_json::from_value(request_json()).unwrap()
}

fn fixed(json: &str) -> Fixed6 {
    serde_json::from_str(json).unwrap()
}

fn assert_schema_violation(request: &RankRequest, expected_path: &str) {
    match rank(request) {
        Err(RankError::SchemaViolation { path, message }) => {
            assert_eq!(path, expected_path, "schema message: {message}");
            assert!(!message.is_empty());
        }
        result => panic!("expected schema violation at {expected_path}, got {result:?}"),
    }
}

#[test]
fn unknown_request_and_nested_fields_are_rejected() {
    let mut top_level = request_json();
    top_level["unexpectedTopLevel"] = json!(true);
    let error = serde_json::from_value::<RankRequest>(top_level)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("unknown field `unexpectedTopLevel`"),
        "{error}"
    );

    let mut nested = request_json();
    nested["candidates"][0]["source"]["unexpectedNested"] = json!(true);
    let error = serde_json::from_value::<RankRequest>(nested)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("unknown field `unexpectedNested`"),
        "{error}"
    );
}

#[test]
fn unknown_feed_slate_fields_are_rejected() {
    let mut slate = serde_json::to_value(rank(&request()).unwrap()).unwrap();
    slate["items"][0]["decision"]["unexpectedDecisionField"] = json!(true);
    let error = serde_json::from_value::<FeedSlate>(slate)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("unknown field `unexpectedDecisionField`"),
        "{error}"
    );
}

#[test]
fn empty_required_request_collections_are_rejected() {
    let mut value = request();
    value.candidates.clear();
    assert_schema_violation(&value, "request.candidates");

    let mut value = request();
    value.analysis.clear();
    assert_schema_violation(&value, "request.analysis");

    let mut value = request();
    value.candidates[0].evidence.clear();
    assert_schema_violation(&value, "request.candidates[0].evidence");

    let mut value = request();
    value.analysis[0].signals.clear();
    assert_schema_violation(&value, "request.analysis[0].signals");
}

#[test]
fn identifiers_tokens_versions_dates_and_uris_are_validated() {
    let mut value = request();
    value.id = "INVALID SPACE".into();
    assert_schema_violation(&value, "request.id");

    let mut value = request();
    value.context.generated_at = "not-a-date".into();
    assert_schema_violation(&value, "request.context.generatedAt");

    let mut value = request();
    value.candidates[0].source.source_type = "Git Hub".into();
    assert_schema_violation(&value, "request.candidates[0].source.type");

    let mut value = request();
    value.candidates[0].provenance.adapter.version = "version-one".into();
    assert_schema_violation(&value, "request.candidates[0].provenance.adapter.version");

    let mut value = request();
    value.candidates[0].subject.url = "not a uri".into();
    assert_schema_violation(&value, "request.candidates[0].subject.url");
}

#[test]
fn signal_and_weight_names_and_ranges_are_validated() {
    let mut value = request();
    value.candidates[0].signals[0].name = "source.".into();
    assert!(matches!(
        rank(&value),
        Err(RankError::InvalidSignalNamespace { .. })
    ));

    let mut value = request();
    value.candidates[0].signals[0].value = fixed("1.000001");
    assert_schema_violation(&value, "request.candidates[0].signals[0].value");

    let mut value = request();
    value.lens.weights.insert("bad.weight".into(), fixed("1"));
    assert_schema_violation(&value, "request.lens.weights.bad.weight");

    let mut value = request();
    value
        .lens
        .weights
        .insert("source.activity".into(), fixed("10.000001"));
    assert_schema_violation(&value, "request.lens.weights.source.activity");
}

#[test]
fn schema_unique_and_nonempty_constraints_are_validated() {
    let mut value = request();
    value.lens.allowed_source_types = Some(Vec::new());
    assert_schema_violation(&value, "request.lens.allowedSourceTypes");

    let mut value = request();
    value.lens.allowed_source_types = Some(vec!["github".into(), "github".into()]);
    assert_schema_violation(&value, "request.lens.allowedSourceTypes[1]");

    let mut value = request();
    value.lens.weights.clear();
    assert_schema_violation(&value, "request.lens.weights");

    let mut value = request();
    let evidence_ref = value.candidates[0].signals[0].evidence_refs[0].clone();
    value.candidates[0].signals[0]
        .evidence_refs
        .push(evidence_ref);
    assert_schema_violation(&value, "request.candidates[0].signals[0].evidenceRefs[1]");
}

#[test]
fn six_decimal_signal_values_rank_successfully() {
    let mut value = request();
    value.candidates[0].signals[0].value = fixed("0.000001");
    assert!(rank(&value).is_ok());
}
