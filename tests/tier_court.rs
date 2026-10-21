//! R04 two-tier admission court: declared-tier dispatch, tier-mismatch
//! refusals, and unchanged undeclared sniff behavior.

use std::sync::Arc;

use graphlaw::dialect::{Dialect, RefusalKind, admit_bytes};
use graphlaw::tier::{Admitted, DialectDecl, Tier, admit_declared, admit_bytes_tiered};

const N3_RULES: &[u8] = b"\
@prefix : <https://e/> .\n\
:tom a :Cat .\n\
{ ?x a :Cat } => { ?x a :Animal } .\n";

const TURTLE_DATA: &[u8] = b"\
@prefix : <https://e/> .\n\
:tom a :Cat .\n";

/// N3 formula/brace syntax that is NOT valid Turtle: sniff alone refuses this
/// as ambiguous (braces), and a Turtle parse fails.
const N3_FORMULAS: &[u8] = b"\
@prefix : <https://e/> .\n\
{ :tom a :Cat } => { :tom a :Animal } .\n";

#[test]
fn declared_n3_loads_as_rules_without_dataset_entanglement() {
    let decl = DialectDecl {
        tier: Tier::N3Logic,
        dialect: Dialect::N3,
    };
    let admitted = admit_declared(N3_RULES, decl, None).expect("declared N3 admits as rules");
    match admitted {
        Admitted::Rules(rules) => {
            assert_eq!(rules.rule_count(), 1, "one logic rule expected");
            assert_eq!(rules.document.facts.len(), 1, "one fact expected");
        }
        Admitted::Dataset(_) => panic!("N3Logic tier must not produce a dataset"),
        _ => panic!("unexpected Admitted variant"),
    }
}

#[test]
fn declared_rdf_with_formula_syntax_is_typed_tier_mismatch() {
    let decl = DialectDecl {
        tier: Tier::Rdf,
        dialect: Dialect::Turtle,
    };
    let err = admit_declared(N3_FORMULAS, decl, None).expect_err("formulas are not Turtle");
    assert_eq!(err.kind, RefusalKind::TierMismatch, "actual: {err}");
}

#[test]
fn inconsistent_declaration_is_refused() {
    // dialect N3 carries N3Logic, so an Rdf tier declaration is contradictory.
    let decl = DialectDecl {
        tier: Tier::Rdf,
        dialect: Dialect::N3,
    };
    let err = admit_declared(TURTLE_DATA, decl, None).expect_err("inconsistent decl");
    assert_eq!(err.kind, RefusalKind::TierMismatch, "actual: {err}");
}

#[test]
fn undeclared_sniff_behavior_unchanged_for_datasets() {
    // Turtle data with no declaration: same outcome as the historical
    // admit_bytes path — a dataset.
    let tiered = admit_bytes_tiered(TURTLE_DATA, None, None, None).expect("turtle admits");
    match (tiered, admit_bytes(TURTLE_DATA, None, None)) {
        (Admitted::Dataset(a), Ok(state)) => {
            let direct = Arc::clone(&a);
            assert_eq!(direct.quad_count(), state.quad_count());
        }
        _ => panic!("dataset path diverged between tiered and legacy admit"),
    }
}

#[test]
fn undeclared_n3_logic_refused_as_tier_mismatch() {
    // Sniff routes `=>` documents to Dialect::N3; the RDF admit path used to
    // surface a confusing dataset refusal. Now it is a typed TierMismatch.
    let err = admit_bytes_tiered(N3_RULES, None, None, None)
        .expect_err("N3 logic needs a declaration");
    assert_eq!(err.kind, RefusalKind::TierMismatch, "actual: {err}");
}

#[test]
fn undeclared_nonsemantic_failure_is_not_upgraded() {
    // HTML is not N3 and not RDF: the original NotSemanticContent refusal
    // must survive the tier cross-check.
    let html = b"<!doctype html><html><body>404</body></html>";
    let err = admit_bytes_tiered(html, None, None, None).expect_err("html refused");
    assert_eq!(err.kind, RefusalKind::NotSemanticContent, "actual: {err}");
}

#[test]
fn declared_rdf_data_still_admits_as_dataset() {
    let decl = DialectDecl {
        tier: Tier::Rdf,
        dialect: Dialect::Turtle,
    };
    let admitted = admit_declared(TURTLE_DATA, decl, None).expect("turtle data admits");
    match admitted {
        Admitted::Dataset(ds) => assert_eq!(ds.quad_count(), 1),
        _ => panic!("Rdf tier must produce a dataset"),
    }
}
