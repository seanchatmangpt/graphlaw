//! SHACL guard refusals: `component` is the typed limit name, `message` the prose.

use graphlaw::dialect::Dialect;
use graphlaw::law::{LawError, LawState, Step};

#[test]
fn bytes_guard_puts_limit_name_in_component_and_prose_in_message() {
    let state = LawState::parse(b"@prefix : <https://e/> .\n:a :p :b .\n", Dialect::Turtle, None)
        .expect("parses");
    let pad = "#".repeat(graphlaw::shacl_guard::MAX_SHACL_SHAPES_BYTES + 1);
    let shapes = format!("@prefix sh: <http://www.w3.org/ns/shacl#> .\n{pad}\n");
    match state.transition(&Step::AdmitShacl { shapes_ttl: &shapes }) {
        Err(LawError::NotAdmitted { violations, results }) => {
            assert_eq!(violations, 1);
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].component, "shacl_validation_bytes");
            assert!(!results[0].message.is_empty());
            assert!(results[0].message.contains("shacl_validation_bytes"));
        }
        other => panic!("expected guard refusal, got {other:?}"),
    }
}
