use refetch_contract::FeedSlate;
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_TEMP_DIR: AtomicUsize = AtomicUsize::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "refetch-cli-test-{}-{}",
            std::process::id(),
            NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn join(&self, path: impl AsRef<Path>) -> PathBuf {
        self.0.join(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run_rank(input: &Path, output: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_refetch"))
        .arg("rank")
        .arg("--input")
        .arg(input)
        .arg("--output")
        .arg(output)
        .output()
        .unwrap()
}

fn invalid_request(name: &str) -> Value {
    let wrapper: Value = serde_json::from_str(
        &fs::read_to_string(
            root().join(format!("tests/spec/v0.1/fixtures/v0.1/invalid/{name}.json")),
        )
        .unwrap(),
    )
    .unwrap();
    wrapper["request"].clone()
}

#[test]
fn valid_request_produces_the_expected_deterministic_slate() {
    let temp = TempDir::new();
    let input = root().join("tests/spec/v0.1/fixtures/v0.1/valid/production.rank-request.json");
    let expected: FeedSlate = serde_json::from_str(
        &fs::read_to_string(
            root().join("tests/spec/v0.1/fixtures/v0.1/expected/production.feed-slate.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let first_output = temp.join("first.feed-slate.json");
    let second_output = temp.join("second.feed-slate.json");

    let first = run_rank(&input, &first_output);
    assert!(
        first.status.success(),
        "CLI failed: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    let actual: FeedSlate =
        serde_json::from_str(&fs::read_to_string(&first_output).unwrap()).unwrap();
    assert_eq!(actual, expected);

    let second = run_rank(&input, &second_output);
    assert!(
        second.status.success(),
        "CLI failed: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_eq!(
        fs::read(&first_output).unwrap(),
        fs::read(&second_output).unwrap()
    );
}

#[test]
fn semantic_error_returns_nonzero_without_creating_a_slate() {
    let temp = TempDir::new();
    let input = temp.join("unsupported-spec-version.json");
    let output = temp.join("output.json");
    fs::write(
        &input,
        serde_json::to_vec(&invalid_request("unsupported-spec-version")).unwrap(),
    )
    .unwrap();

    let result = run_rank(&input, &output);

    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("unsupported spec version"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(!output.exists());
}

#[test]
fn schema_error_reports_the_failing_path_without_creating_a_slate() {
    let temp = TempDir::new();
    let input = temp.join("empty-candidates.json");
    let output = temp.join("output.json");
    fs::write(
        &input,
        serde_json::to_vec(&invalid_request("empty-candidates")).unwrap(),
    )
    .unwrap();

    let result = run_rank(&input, &output);

    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("schema violation at request.candidates"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(!output.exists());
}

#[test]
fn malformed_json_returns_nonzero_without_creating_a_slate() {
    let temp = TempDir::new();
    let input = temp.join("malformed.json");
    let output = temp.join("output.json");
    fs::write(&input, b"{\"specVersion\":").unwrap();

    let result = run_rank(&input, &output);

    assert!(!result.status.success());
    assert!(!result.stderr.is_empty());
    assert!(!output.exists());
}
