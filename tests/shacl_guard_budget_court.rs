//! Court: expansion-budget exhaustion is its own typed limit
//! (`shacl_validation_expansion_budget`), never a fabricated `Depth`, and the
//! legacy `check_shape_depth` entry point cannot bypass the physical caps.
//!
//! Anti-vacuity: the budget graph has true depth 9 (< MAX), so a guard that
//! reports `Depth` (the old fabrication) or admits fails this court; a
//! `check_shape_depth` that is depth-only admits the oversize graph and fails.

use graphlaw::dialect::Dialect;
use graphlaw::law::{LawError, LawState};
use graphlaw::shacl_guard::{self, ShaclLimitExceeded};

const PREFIX: &str =
    "@prefix sh: <http://www.w3.org/ns/shacl#> .\n@prefix ex: <http://example.org/> .\n";

/// Layered DAG: `layers` layers of `width` shapes, every shape linking to
/// every shape of the next layer. Depth = layers-1 edges (<= MAX), paths =
/// width^(layers-1) (>> expansion budget).
fn layered_dag(layers: usize, width: usize) -> String {
    let mut ttl = String::from(PREFIX);
    for l in 0..layers - 1 {
        for j in 0..width {
            for k in 0..width {
                ttl.push_str(&format!(
                    "ex:l{l}n{j} a sh:NodeShape ; sh:node ex:l{}n{k} .\n",
                    l + 1
                ));
            }
        }
    }
    ttl
}

#[test]
fn budget_exhaustion_is_typed_expansion_budget_not_depth() {
    let ttl = layered_dag(10, 4);
    assert!(ttl.len() < shacl_guard::MAX_SHACL_SHAPES_BYTES);
    let err = shacl_guard::check_shape_limits(&ttl).expect_err("budget must refuse");
    assert!(
        matches!(err, ShaclLimitExceeded::ExpansionBudget { .. }),
        "expected ExpansionBudget, got {err:?}"
    );
    assert_eq!(err.limit_name(), "shacl_validation_expansion_budget");
    assert_eq!(err.cap(), shacl_guard::EXPANSION_BUDGET as usize);
}

#[test]
fn budget_exhaustion_renders_through_law_with_typed_component() {
    let ttl = layered_dag(10, 4);
    let state = LawState::parse(b"", Dialect::NTriples, None).expect("empty state parses");
    match state.transition(&graphlaw::law::Step::AdmitShacl { shapes_ttl: &ttl }) {
        Err(LawError::NotAdmitted { results, .. }) => {
            assert_eq!(results[0].component, "shacl_validation_expansion_budget");
        }
        other => panic!("expected NotAdmitted, got {:?}", other.map(|_| ())),
    }
}

#[test]
fn shallow_layered_dag_under_budget_is_admitted() {
    // 4 layers x width 3 = 27 paths: well inside the budget.
    assert!(shacl_guard::check_shape_limits(&layered_dag(4, 3)).is_ok());
}

#[test]
fn check_shape_depth_cannot_bypass_byte_cap() {
    let big = format!(
        "{PREFIX}# {}\n",
        "x".repeat(shacl_guard::MAX_SHACL_SHAPES_BYTES)
    );
    let err = shacl_guard::check_shape_depth(&big).expect_err("byte cap must apply");
    assert!(matches!(err, ShaclLimitExceeded::Bytes { .. }), "{err:?}");
}

#[test]
fn check_shape_depth_cannot_bypass_degree_cap() {
    let mut ttl = String::from(PREFIX);
    ttl.push_str("ex:root a sh:NodeShape");
    for i in 0..=shacl_guard::MAX_SHACL_DEGREE {
        ttl.push_str(&format!(" ; sh:node ex:c{i}"));
    }
    ttl.push_str(" .\n");
    let err = shacl_guard::check_shape_depth(&ttl).expect_err("degree cap must apply");
    assert!(matches!(err, ShaclLimitExceeded::Degree { .. }), "{err:?}");
}
