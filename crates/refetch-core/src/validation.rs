use crate::RankError;
use refetch_contract::*;
use std::collections::BTreeSet;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uriparse::URI;

const SIGNAL_LIMIT: i64 = 1_000_000;
const WEIGHT_LIMIT: i64 = 10_000_000;

pub fn validate(request: &RankRequest) -> Result<(), RankError> {
    if request.spec_version != SPEC_VERSION {
        return Err(RankError::UnsupportedSpecVersion(
            request.spec_version.clone(),
        ));
    }
    require_schema(
        is_id(&request.id),
        "request.id",
        "must match ^[a-z][a-z0-9:-]*$",
    )?;
    require_schema(
        is_rfc3339(&request.context.generated_at),
        "request.context.generatedAt",
        "must be an RFC 3339 date-time",
    )?;
    require_schema(
        !request.candidates.is_empty(),
        "request.candidates",
        "must contain at least one candidate",
    )?;
    require_schema(
        !request.analysis.is_empty(),
        "request.analysis",
        "must contain at least one analysis record",
    )?;
    validate_lens(&request.lens)?;

    let mut candidate_ids = BTreeSet::new();
    for (index, candidate) in request.candidates.iter().enumerate() {
        validate_candidate(candidate, index)?;
        if !candidate_ids.insert(candidate.id.as_str()) {
            return Err(RankError::DuplicateId {
                kind: "candidate",
                id: candidate.id.clone(),
            });
        }
    }

    let mut analysis_ids = BTreeSet::new();
    let mut analyzed_candidates = BTreeSet::new();
    for (index, analysis) in request.analysis.iter().enumerate() {
        validate_analysis(analysis, index)?;
        if !analysis_ids.insert(analysis.id.as_str()) {
            return Err(RankError::DuplicateId {
                kind: "analysis",
                id: analysis.id.clone(),
            });
        }
        if !candidate_ids.contains(analysis.candidate_id.as_str()) {
            return Err(RankError::UnknownCandidate(analysis.candidate_id.clone()));
        }
        if !analyzed_candidates.insert(analysis.candidate_id.as_str()) {
            return Err(RankError::DuplicateId {
                kind: "analysisForCandidate",
                id: analysis.candidate_id.clone(),
            });
        }
    }

    for id in candidate_ids {
        if !analyzed_candidates.contains(id) {
            return Err(RankError::MissingAnalysis(id.into()));
        }
    }
    Ok(())
}

fn validate_lens(lens: &LensProfile) -> Result<(), RankError> {
    if lens.spec_version != SPEC_VERSION {
        return Err(RankError::UnsupportedSpecVersion(lens.spec_version.clone()));
    }
    require_schema(
        is_id(&lens.id),
        "request.lens.id",
        "must match ^[a-z][a-z0-9:-]*$",
    )?;
    require_schema(
        !lens.title.is_empty(),
        "request.lens.title",
        "must not be empty",
    )?;
    if let Some(source_types) = &lens.allowed_source_types {
        require_schema(
            !source_types.is_empty(),
            "request.lens.allowedSourceTypes",
            "must contain at least one source type when present",
        )?;
        let mut unique = BTreeSet::new();
        for (index, source_type) in source_types.iter().enumerate() {
            let path = format!("request.lens.allowedSourceTypes[{index}]");
            require_schema(is_token(source_type), &path, "must be a lowercase token")?;
            require_schema(
                unique.insert(source_type.as_str()),
                &path,
                "must not duplicate another allowed source type",
            )?;
        }
    }
    require_schema(
        !lens.weights.is_empty(),
        "request.lens.weights",
        "must contain at least one weight",
    )?;
    for (name, weight) in &lens.weights {
        let path = format!("request.lens.weights.{name}");
        require_schema(
            is_weight_name(name),
            &path,
            "name must use the source.* or analysis.* namespace",
        )?;
        require_schema(
            (-WEIGHT_LIMIT..=WEIGHT_LIMIT).contains(&weight.raw()),
            &path,
            "value must be between -10 and 10",
        )?;
    }
    if lens.policy.max_items < 1 {
        return Err(RankError::InvalidPolicy("maxItems must be >= 1".into()));
    }
    if lens.policy.max_per_cluster < 1 {
        return Err(RankError::InvalidPolicy(
            "maxPerCluster must be >= 1".into(),
        ));
    }
    if lens.policy.tie_breaker != "candidateIdAsc" {
        return Err(RankError::InvalidPolicy(
            "tieBreaker must be candidateIdAsc".into(),
        ));
    }
    Ok(())
}

fn validate_candidate(candidate: &FeedCandidate, index: usize) -> Result<(), RankError> {
    let path = format!("request.candidates[{index}]");
    if candidate.spec_version != SPEC_VERSION {
        return Err(RankError::UnsupportedSpecVersion(
            candidate.spec_version.clone(),
        ));
    }
    require_schema(
        is_id(&candidate.id),
        &format!("{path}.id"),
        "must match ^[a-z][a-z0-9:-]*$",
    )?;
    require_schema(
        is_token(&candidate.source.source_type),
        &format!("{path}.source.type"),
        "must be a lowercase token",
    )?;
    require_schema(
        !candidate.source.name.is_empty(),
        &format!("{path}.source.name"),
        "must not be empty",
    )?;
    require_schema(
        is_id(&candidate.subject.id),
        &format!("{path}.subject.id"),
        "must match ^[a-z][a-z0-9:-]*$",
    )?;
    require_schema(
        is_token(&candidate.subject.subject_type),
        &format!("{path}.subject.type"),
        "must be a lowercase token",
    )?;
    require_schema(
        !candidate.subject.title.is_empty(),
        &format!("{path}.subject.title"),
        "must not be empty",
    )?;
    require_schema(
        is_uri(&candidate.subject.url),
        &format!("{path}.subject.url"),
        "must be an absolute URI",
    )?;
    require_schema(
        is_token(&candidate.trigger.trigger_type),
        &format!("{path}.trigger.type"),
        "must be a lowercase token",
    )?;
    require_schema(
        is_rfc3339(&candidate.trigger.observed_at),
        &format!("{path}.trigger.observedAt"),
        "must be an RFC 3339 date-time",
    )?;
    validate_component(
        &candidate.provenance.adapter,
        &format!("{path}.provenance.adapter"),
    )?;
    require_schema(
        is_rfc3339(&candidate.provenance.retrieved_at),
        &format!("{path}.provenance.retrievedAt"),
        "must be an RFC 3339 date-time",
    )?;
    validate_record(
        &candidate.id,
        &candidate.evidence,
        &candidate.signals,
        "source.",
        &path,
    )
}

fn validate_analysis(analysis: &AnalysisRecord, index: usize) -> Result<(), RankError> {
    let path = format!("request.analysis[{index}]");
    if analysis.spec_version != SPEC_VERSION {
        return Err(RankError::UnsupportedSpecVersion(
            analysis.spec_version.clone(),
        ));
    }
    require_schema(
        is_id(&analysis.id),
        &format!("{path}.id"),
        "must match ^[a-z][a-z0-9:-]*$",
    )?;
    require_schema(
        is_id(&analysis.candidate_id),
        &format!("{path}.candidateId"),
        "must match ^[a-z][a-z0-9:-]*$",
    )?;
    validate_component(&analysis.analyzer, &format!("{path}.analyzer"))?;
    require_schema(
        is_rfc3339(&analysis.created_at),
        &format!("{path}.createdAt"),
        "must be an RFC 3339 date-time",
    )?;
    validate_record(
        &analysis.id,
        &analysis.evidence,
        &analysis.signals,
        "analysis.",
        &path,
    )?;
    if let Some(cluster) = &analysis.cluster_assignment {
        let cluster_path = format!("{path}.clusterAssignment");
        require_schema(
            is_token(&cluster.namespace),
            &format!("{cluster_path}.namespace"),
            "must be a lowercase token",
        )?;
        require_schema(
            is_id(&cluster.id),
            &format!("{cluster_path}.id"),
            "must match ^[a-z][a-z0-9:-]*$",
        )?;
        validate_evidence_refs(
            &analysis.id,
            &analysis.evidence,
            &cluster.evidence_refs,
            &format!("{cluster_path}.evidenceRefs"),
        )?;
    }
    Ok(())
}

fn validate_component(component: &Component, path: &str) -> Result<(), RankError> {
    require_schema(
        is_token(&component.name),
        &format!("{path}.name"),
        "must be a lowercase token",
    )?;
    require_schema(
        is_component_version(&component.version),
        &format!("{path}.version"),
        "must be a three-part component version",
    )
}

fn validate_record(
    record: &str,
    evidence: &[Evidence],
    signals: &[Signal],
    namespace: &str,
    path: &str,
) -> Result<(), RankError> {
    require_schema(
        !evidence.is_empty(),
        &format!("{path}.evidence"),
        "must contain at least one evidence item",
    )?;
    require_schema(
        !signals.is_empty(),
        &format!("{path}.signals"),
        "must contain at least one signal",
    )?;
    let mut evidence_ids = BTreeSet::new();
    for (index, item) in evidence.iter().enumerate() {
        let item_path = format!("{path}.evidence[{index}]");
        validate_evidence(item, &item_path)?;
        if !evidence_ids.insert(item.id.as_str()) {
            return Err(RankError::DuplicateId {
                kind: "evidence",
                id: item.id.clone(),
            });
        }
    }
    let mut signal_names = BTreeSet::new();
    for (index, signal) in signals.iter().enumerate() {
        let signal_path = format!("{path}.signals[{index}]");
        if !is_signal_name(&signal.name, namespace) {
            return Err(RankError::InvalidSignalNamespace {
                record: record.into(),
                signal: signal.name.clone(),
            });
        }
        require_schema(
            (-SIGNAL_LIMIT..=SIGNAL_LIMIT).contains(&signal.value.raw()),
            &format!("{signal_path}.value"),
            "must be between -1 and 1",
        )?;
        if !signal_names.insert(signal.name.as_str()) {
            return Err(RankError::DuplicateSignal {
                record: record.into(),
                signal: signal.name.clone(),
            });
        }
        validate_evidence_refs(
            record,
            evidence,
            &signal.evidence_refs,
            &format!("{signal_path}.evidenceRefs"),
        )?;
    }
    Ok(())
}

fn validate_evidence(item: &Evidence, path: &str) -> Result<(), RankError> {
    require_schema(
        is_id(&item.id),
        &format!("{path}.id"),
        "must match ^[a-z][a-z0-9:-]*$",
    )?;
    require_schema(
        is_token(&item.kind),
        &format!("{path}.kind"),
        "must be a lowercase token",
    )?;
    require_schema(
        !item.description.is_empty(),
        &format!("{path}.description"),
        "must not be empty",
    )?;
    if let Some(url) = &item.url {
        require_schema(
            is_uri(url),
            &format!("{path}.url"),
            "must be an absolute URI",
        )?;
    }
    Ok(())
}

fn validate_evidence_refs(
    record: &str,
    evidence: &[Evidence],
    evidence_refs: &[String],
    path: &str,
) -> Result<(), RankError> {
    require_schema(
        !evidence_refs.is_empty(),
        path,
        "must contain at least one evidence reference",
    )?;
    let evidence_ids: BTreeSet<_> = evidence.iter().map(|item| item.id.as_str()).collect();
    let mut unique = BTreeSet::new();
    for (index, evidence_ref) in evidence_refs.iter().enumerate() {
        let item_path = format!("{path}[{index}]");
        require_schema(
            is_id(evidence_ref),
            &item_path,
            "must match ^[a-z][a-z0-9:-]*$",
        )?;
        require_schema(
            unique.insert(evidence_ref.as_str()),
            &item_path,
            "must not duplicate another evidence reference",
        )?;
        if !evidence_ids.contains(evidence_ref.as_str()) {
            return Err(RankError::DanglingEvidenceRef {
                record: record.into(),
                evidence_ref: evidence_ref.clone(),
            });
        }
    }
    Ok(())
}

fn require_schema(condition: bool, path: &str, message: &str) -> Result<(), RankError> {
    if condition {
        Ok(())
    } else {
        Err(RankError::SchemaViolation {
            path: path.into(),
            message: message.into(),
        })
    }
}

fn is_id(value: &str) -> bool {
    let mut bytes = value.bytes();
    matches!(bytes.next(), Some(b'a'..=b'z'))
        && bytes
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b":-".contains(&byte))
}

fn is_token(value: &str) -> bool {
    let bytes = value.as_bytes();
    if !matches!(bytes.first(), Some(b'a'..=b'z')) {
        return false;
    }
    let mut separator = false;
    for byte in &bytes[1..] {
        if byte.is_ascii_lowercase() || byte.is_ascii_digit() {
            separator = false;
        } else if b"._-".contains(byte) && !separator {
            separator = true;
        } else {
            return false;
        }
    }
    !separator
}

fn is_signal_name(value: &str, namespace: &str) -> bool {
    let Some(suffix) = value.strip_prefix(namespace) else {
        return false;
    };
    let mut bytes = suffix.bytes();
    matches!(bytes.next(), Some(b'a'..=b'z'))
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || b"_.-".contains(&byte))
}

fn is_weight_name(value: &str) -> bool {
    is_signal_name(value, "source.") || is_signal_name(value, "analysis.")
}

fn is_component_version(value: &str) -> bool {
    let suffix_start = value.find(['-', '+']);
    let (core, suffix) = match suffix_start {
        Some(index) => (&value[..index], Some(&value[index + 1..])),
        None => (value, None),
    };
    let mut parts = core.split('.');
    let core_is_valid = (0..3).all(|_| {
        parts
            .next()
            .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
    }) && parts.next().is_none();
    core_is_valid
        && suffix.is_none_or(|suffix| {
            !suffix.is_empty()
                && suffix
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b".-".contains(&byte))
        })
}

fn is_rfc3339(value: &str) -> bool {
    OffsetDateTime::parse(value, &Rfc3339).is_ok()
}

fn is_uri(value: &str) -> bool {
    URI::try_from(value).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_identifier_and_token_patterns_are_exact() {
        for value in ["a", "req:one-two", "abc123"] {
            assert!(is_id(value), "valid id rejected: {value}");
        }
        for value in ["", "A", "bad_id", "bad space"] {
            assert!(!is_id(value), "invalid id accepted: {value}");
        }
        for value in ["a", "github.release", "feed-item_2"] {
            assert!(is_token(value), "valid token rejected: {value}");
        }
        for value in ["", "GitHub", "bad..token", "bad-"] {
            assert!(!is_token(value), "invalid token accepted: {value}");
        }
    }

    #[test]
    fn schema_signal_and_component_version_patterns_are_exact() {
        assert!(is_signal_name("source.activity", "source."));
        assert!(is_signal_name("analysis.someSignal-v2", "analysis."));
        assert!(!is_signal_name("source.", "source."));
        assert!(!is_signal_name("analysis.bad", "source."));

        for value in ["0.1.0", "1.2.3-alpha.1", "1.2.3+build.7"] {
            assert!(
                is_component_version(value),
                "valid version rejected: {value}"
            );
        }
        for value in ["1", "1.2", "v1.2.3", "1.2.3-alpha+build"] {
            assert!(
                !is_component_version(value),
                "invalid version accepted: {value}"
            );
        }
    }

    #[test]
    fn rfc3339_validation_checks_calendar_time_and_zone() {
        for value in [
            "2026-01-15T00:00:00Z",
            "2024-02-29t23:59:59.123456+08:00",
            "2026-01-15T00:00:00-00:00",
        ] {
            assert!(is_rfc3339(value), "valid date-time rejected: {value}");
        }
        for value in [
            "not-a-date",
            "2023-02-29T00:00:00Z",
            "2026-13-01T00:00:00Z",
            "2026-01-01T24:00:00Z",
            "2026-01-01T00:00:00+24:00",
            "2026-01-01T00:00:00",
        ] {
            assert!(!is_rfc3339(value), "invalid date-time accepted: {value}");
        }
    }

    #[test]
    fn uri_validation_accepts_absolute_uris_and_rejects_malformed_values() {
        for value in [
            "https://example.com/path?q=1#fragment",
            "https://[::1]:443/path",
            "urn:isbn:9780141036144",
            "mailto:user@example.com",
            "file:///tmp/refetch.json",
        ] {
            assert!(is_uri(value), "valid URI rejected: {value}");
        }
        for value in [
            "relative/path",
            "not a uri",
            "https://example.com/%zz",
            "https://[broken/path",
            "https://example.com:port/path",
        ] {
            assert!(!is_uri(value), "invalid URI accepted: {value}");
        }
    }
}
