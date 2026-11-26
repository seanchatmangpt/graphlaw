//! Court: the guard parses with the engine's own parser, so a parse failure
//! is never admitted and cannot hide cap violations.

use graphlaw::dialect::Dialect;
use graphlaw::law::{LawError, LawState, Step};
use graphlaw::shacl_guard::{self, ShaclLimitExceeded};

const PREFIX: &str =
    "@prefix sh: <http://www.w3.org/ns/shacl#> .\n@prefix ex: <http://example.org/> .\n";

#[test]
fn guard_parse_failure_is_not_ok() {
    let bad = format!("{PREFIX}ex:a sh:node ex:b . this is not turtle");
    let err = shacl_guard::check_shape_limits(&bad).expect_err("unparseable must not be Ok");
    assert!(err.is_parse_failure(), "got {err:?}");
    assert!(purrdf::shapes::engine::parse_shapes(&bad, None).is_err());
}

#[test]
fn malformed_shapes_are_refused_by_law() {
    let bad = format!("{PREFIX}ex:a sh:node ex:b . garbage garbage");
    let state = LawState::parse(b"", Dialect::NTriples, None).expect("empty state");
    let r = state.transition(&Step::AdmitShacl { shapes_ttl: &bad });
    assert!(r.is_err(), "malformed shapes must not be admitted");
    assert!(
        !matches!(r, Err(LawError::NotAdmitted { .. })),
        "must be the engine parse refusal"
    );
}

#[test]
fn guard_and_engine_agree_on_parse_verdict() {
    let cases = [
        format!("{PREFIX}ex:a a sh:NodeShape ."),
        format!("{PREFIX}ex:a sh:node <rel> ."),
        "ex:a sh:node ex:b .".to_string(),
        format!("{PREFIX}ex:a sh:node"),
        "@base <http://e/> . <a> <http://www.w3.org/ns/shacl#node> <b> .".to_string(),
    ];
    for c in &cases {
        let guard_ok = shacl_guard::parse_shapes_for_guard(c, None).is_ok();
        let engine_parse_ok = !matches!(
            purrdf::shapes::engine::parse_shapes(c, None),
            Err(purrdf::shapes::ShapesError::Invalid(_))
        );
        assert_eq!(guard_ok, engine_parse_ok, "parse differential on {c:?}");
    }
}

#[test]
fn cap_violation_still_refused_when_parseable() {
    let mut ttl = String::from(PREFIX);
    for k in 0..(shacl_guard::MAX_SHACL_DEGREE + 1) {
        ttl.push_str(&format!("ex:root sh:node ex:c{k} .\n"));
    }
    let err = shacl_guard::check_shape_limits(&ttl).expect_err("degree");
    assert!(matches!(err, ShaclLimitExceeded::Degree { .. }), "{err:?}");
}
