//! GWS-5 court: the case study's Phase-2 SHACL preflight must run INSIDE the
//! wasm sandbox. The governance shapes (`ontologies/shapes/ast_governance.ttl`)
//! — the sql:SelectStatementShape anti-scan law (whereClause minCount 1) and
//! the sql:TableRefShape identifier pattern — are validated through the wasm
//! ABI's `shacl` op, proving the governance travels with the sandbox: no host
//! involvement, the same wasmi instance drives conforming and hostile ASTs.
//!
//! (a) the conforming cold-chain AST (`tests/fixtures/ast_governance/cold_chain_ast.ttl`)
//!     admits;
//! (b) hostile AST variants get typed SHACL violations naming the violated
//!     shape and constraint component:
//!       - a WHERE-less SELECT (unbounded scan) violates the anti-scan
//!         property shape sql:HasWhereClause (of sql:SelectStatementShape)
//!         on sh:minCount over sql:whereClause;
//!       - a table name carrying an injection token (`ColdChain; DROP`)
//!         violates the pattern property shape sql:TableNamePattern (of
//!         sql:TableRefShape) on sh:pattern.
//!
//! Conventions follow tests/wasm_abi.rs via tests/common/mod.rs. Real wasm
//! module, real shapes file, real SHACL engine — no mocks.

mod common;

use common::{read, wasm};
use serde_json::{Value, json};

fn governance_shapes() -> String {
    read("ontologies/shapes/ast_governance.ttl")
}

fn shacl_req(data: &str, shapes: &str) -> Value {
    json!({"op": "shacl", "data": {"text": data, "dialect": "turtle"}, "shapes": shapes})
}

/// Violations must name the violated shape IRI and the constraint component.
fn violations(r: &Value) -> Vec<(String, String)> {
    assert_eq!(
        r["conforms"],
        Value::Bool(false),
        "hostile AST must not conform: {r}"
    );
    r["results"]
        .as_array()
        .expect("violation results array")
        .iter()
        .map(|v| {
            (
                v["shape"].as_str().unwrap_or_default().to_string(),
                v["component"].as_str().unwrap_or_default().to_string(),
            )
        })
        .collect()
}

#[test]
fn governance_preflight_admits_conforming_cold_chain_ast() {
    let shapes = governance_shapes();
    let ast = read("tests/fixtures/ast_governance/cold_chain_ast.ttl");
    let r = wasm(&shacl_req(&ast, &shapes));
    assert_eq!(r["conforms"], Value::Bool(true), "report: {r}");
    assert!(
        r["results"].as_array().unwrap().is_empty(),
        "conforming AST carries no violations: {r}"
    );
}

#[test]
fn governance_preflight_refuses_whereless_scan_naming_select_statement_shape() {
    let shapes = governance_shapes();
    // Strip the WHERE clause: rename the predicate so the SELECT becomes an
    // unbounded scan. The AST must be refused with a typed violation on
    // sql:HasWhereClause / MinCount.
    let ast = read("tests/fixtures/ast_governance/cold_chain_ast.ttl")
        .replace("sql:whereClause", "sql:comment");
    let r = wasm(&shacl_req(&ast, &shapes));
    let vs = violations(&r);
    assert!(
        vs.iter().any(
            |(shape, component)| shape == "<http://graphlaw.org/sql#HasWhereClause>"
                && component == "<http://www.w3.org/ns/shacl#MinCountConstraintComponent>"
        ),
        "expected sql:HasWhereClause (sql:SelectStatementShape) / MinCount violation, got {vs:?}"
    );
}

#[test]
fn governance_preflight_refuses_injection_token_naming_table_ref_shape() {
    let shapes = governance_shapes();
    // Injection token in the FROM table name: the identifier pattern (via the
    // sequence path sql:fromTable/sql:tableName) must refuse it with a typed
    // Pattern violation naming sql:FromTableNamePattern.
    let ast = read("tests/fixtures/ast_governance/cold_chain_ast.ttl")
        .replace("\"ocel_events\"", "\"ColdChain; DROP\"");
    let r = wasm(&shacl_req(&ast, &shapes));
    let vs = violations(&r);
    assert!(
        vs.iter().any(|(shape, component)| shape
            == "<http://graphlaw.org/sql#FromTableNamePattern>"
            && component == "<http://www.w3.org/ns/shacl#PatternConstraintComponent>"),
        "expected sql:FromTableNamePattern (sql:TableRefShape) / Pattern violation, got {vs:?}"
    );
}
