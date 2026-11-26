//! A hook trigger SELECT is executed once per evaluation: variables and rows
//! come back from the same call.

use graphlaw::dialect::Dialect;
use graphlaw::hooks::trigger_solutions;
use graphlaw::law::LawState;

#[test]
fn trigger_solutions_returns_variables_and_rows_together() {
    let state = LawState::parse(
        b"@prefix : <https://e/> .\n:a :p :b .\n:c :p :d .\n",
        Dialect::Turtle,
        None,
    )
    .expect("parses");
    let (vars, rows) =
        trigger_solutions(&state, "SELECT ?s ?o WHERE { ?s <https://e/p> ?o }").expect("select");
    assert_eq!(vars, vec!["s".to_string(), "o".to_string()]);
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|r| r.len() == vars.len()));
}

#[test]
fn non_select_trigger_is_refused() {
    let state = LawState::parse(b"@prefix : <https://e/> .\n:a :p :b .\n", Dialect::Turtle, None)
        .expect("parses");
    assert!(trigger_solutions(&state, "ASK { ?s ?p ?o }").is_err());
}

#[test]
fn materialize_calls_the_single_helper_only() {
    let src = std::fs::read_to_string(format!("{}/src/hooks.rs", env!("CARGO_MANIFEST_DIR")))
        .expect("read hooks.rs");
    assert!(!src.contains("trigger_vars("), "double-select helper must be gone");
    assert_eq!(src.matches("trigger_solutions(&current").count(), 1);
}
