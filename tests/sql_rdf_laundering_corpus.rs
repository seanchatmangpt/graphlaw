//! SQL-in-RDF laundering corpus (E1 falsifier court).
//!
//! Every case is a real `compile_turtle` document that previously escaped
//! admission review through one of the compiler-hardening escape-audit
//! findings E1–E5 (docs/jira/v26.10.22/RECEIPTS.md) and the two scope
//! fuzz findings; each is now a typed refusal at compile time, before any
//! DuckDB engine call. THESIS.md moonshot E1: the same corpus later becomes
//! by-construction (generated from ontologies/sql-ast.ttl shapes) instead
//! of hand-frozen — until then this file is the regression net.
//!
//! Case -> escape mapping:
//!   e1_mixed_projection_select_item ....... E1 (mixed projection)
//!   e2_hostile_from_under_benign_fromtable  E2 (hostile sql:from under
//!                                            benign sql:fromTable decoy)
//!   e3_duplicate_where_clause ............. E3 (duplicate whereClause)
//!   e4_malformed_limit .................... E4 (malformed limit/offset)
//!   e4_malformed_offset ................... E4 (malformed limit/offset)
//!   e5_left_lefthand_dual_key ............. E5 (leftOperand/left dual key)
//!   scope_on_operator_node ................ scope fuzz finding (scope on a
//!                                            non-column expression node)
//!   dual_scope_on_column .................. scope fuzz finding (scope is
//!                                            single-cardinality)
//!   view_scope_column_in_where ............ REFUSAL-03 scope integrity
//!   process_scope_column_in_select ........ REFUSAL-03 scope integrity
//!
//! Anti-vacuity (rec §6.1, scripts/mutation_court.sh discipline): every
//! refusal court carries an inline FLIP check — the benign twin of the
//! laundered fragment (mutation: replace the hostile predicate/clause with
//! its well-formed counterpart, or drop it) MUST compile, so a gate that
//! refuses everything (or nothing) fails the corpus. Header marker for the
//! sweep: ANTI-VACUITY: inline-flip per court.

use graphlaw::sql_ast::compile_turtle;

/// ANTI-VACUITY (inline-flip): the benign twin of `doc` (hostile fragment
/// rewritten per `fix`) must compile, proving this court's refusal is
/// attributable to the laundered fragment alone — a gate that refuses
/// everything cannot pass the corpus.
fn assert_flip_compiles(doc: &str, fix: (&str, &str), label: &str) {
    let benign = doc.replace(fix.0, fix.1);
    let compiled = compile_turtle(&benign)
        .unwrap_or_else(|e| panic!("{label}: benign twin must compile, got: {e}"));
    assert!(
        compiled.sql.contains("SELECT"),
        "{label}: benign twin must project a SELECT, got: {}",
        compiled.sql
    );
}

fn assert_refusal_01(doc: &str, needle: &str) {
    match compile_turtle(doc) {
        Ok(compiled) => panic!(
            "laundering defect: document silently admitted as `{}` (expected REFUSAL-01 containing '{needle}'):\n{doc}",
            compiled.sql
        ),
        Err(err) => {
            let message = err.to_string();
            assert!(
                message.contains("REFUSAL-01"),
                "wrong refusal class, expected REFUSAL-01, got: {message}"
            );
            assert!(
                message.contains(needle),
                "refusal message missing '{needle}': {message}"
            );
        }
    }
}

fn assert_refusal_03(doc: &str, needle: &str) {
    match compile_turtle(doc) {
        Ok(compiled) => panic!(
            "laundering defect: document silently admitted as `{}` (expected REFUSAL-03 containing '{needle}'):\n{doc}",
            compiled.sql
        ),
        Err(err) => {
            let message = err.to_string();
            assert!(
                message.contains("REFUSAL-03"),
                "wrong refusal class, expected REFUSAL-03, got: {message}"
            );
            assert!(
                message.contains(needle),
                "refusal message missing '{needle}': {message}"
            );
        }
    }
}

const HEAD: &str = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:  <http://example.org/queries#> .
"#;

/// Baseline sanity: the corpus's benign skeleton must compile, so every
/// refusal below is attributable to the laundered fragment and nothing else.
#[test]
fn benign_skeleton_compiles() {
    let doc = format!(
        "{HEAD}\nex:Q a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"telemetry_events\" ] ;\n    sql:selectList ( [ sql:columnName \"batch_id\" ] ) .\n"
    );
    let compiled =
        compile_turtle(&doc).unwrap_or_else(|e| panic!("benign skeleton must compile, got: {e}"));
    assert!(compiled.sql.contains("SELECT"));
}

/// E1 — mixed projection: one select-list item carries two expression
/// discriminators (sql:columnName AND sql:literalValue), laundering a
/// constant past review as a "column". REFUSAL-01, never first-key-wins.
#[test]
fn e1_mixed_projection_select_item() {
    let doc = format!(
        "{HEAD}\nex:Q a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"telemetry_events\" ] ;\n    sql:selectList ( [ sql:columnName \"batch_id\" ; sql:literalValue \"secret\" ] ) .\n"
    );
    assert_refusal_01(&doc, "expression discriminators");
    // FLIP: drop the hostile second discriminator — the twin must compile.
    assert_flip_compiles(
        &doc,
        (
            " sql:columnName \"batch_id\" ; sql:literalValue \"secret\"",
            " sql:columnName \"batch_id\"",
        ),
        "e1 flip",
    );
}

/// E2 — hostile sql:from under a benign sql:fromTable decoy: the validated
/// fromTable must not launder an unreviewed sql:from list past the
/// identifier checks. Exactly one data source clause is allowed.
#[test]
fn e2_hostile_from_under_benign_fromtable() {
    let doc = format!(
        "{HEAD}\nex:Q a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"telemetry_events\" ] ;\n    sql:from ( [ sql:tableName \"events; drop table x\" ] ) ;\n    sql:selectList ( [ sql:columnName \"batch_id\" ] ) .\n"
    );
    assert_refusal_01(&doc, "both sql:from and sql:fromTable");
    // FLIP: drop the laundered sql:from list — the twin must compile.
    assert_flip_compiles(
        &doc,
        (
            " sql:from ( [ sql:tableName \"events; drop table x\" ] ) ;",
            "",
        ),
        "e2 flip",
    );
}

/// E3 — duplicate where clause: sql:whereClause and sql:where on one
/// statement; the duplicate must not win by document order.
#[test]
fn e3_duplicate_where_clause() {
    let doc = format!(
        "{HEAD}\nex:Q a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"telemetry_events\" ] ;\n    sql:selectList ( [ sql:columnName \"batch_id\" ] ) ;\n    sql:whereClause [\n        sql:operator sql:GreaterThan ;\n        sql:leftOperand [ sql:columnName \"latency_ms\" ] ;\n        sql:rightOperand [ sql:literalValue \"400\"^^xsd:integer ]\n    ] ;\n    sql:where [\n        sql:operator sql:GreaterThan ;\n        sql:leftOperand [ sql:columnName \"batch_id\" ] ;\n        sql:rightOperand [ sql:literalValue \"alpha\" ]\n    ] .\n"
    );
    assert_refusal_01(&doc, "both sql:whereClause and sql:where");
    // FLIP: DELETE the duplicate sql:where clause — the twin must compile.
    // (RUST-7: the old rename flip `sql:where` -> `sql:whereX` changed two
    // things at once and, if the unknown predicate were silently ignored,
    // would pass for the wrong reason; deletion changes exactly one
    // dimension — the duplicate key's presence.)
    assert_flip_compiles(
        &doc,
        (
            " ;\n    sql:where [\n        sql:operator sql:GreaterThan ;\n        sql:leftOperand [ sql:columnName \"batch_id\" ] ;\n        sql:rightOperand [ sql:literalValue \"alpha\" ]\n    ]",
            "",
        ),
        "e3 flip",
    );
}

/// E3 twin law (RUST-7): an UNKNOWN where-predicate predicate IRI
/// (`sql:whereX`) must refuse typed (REFUSAL-01, term out of place), never
/// be silently dropped — a rename must not be an escape hatch past the
/// duplicate-key law.
#[test]
fn e3_unknown_where_predicate_refuses_typed() {
    let doc = format!(
        "{HEAD}\nex:Q a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"telemetry_events\" ] ;\n    sql:selectList ( [ sql:columnName \"batch_id\" ] ) ;\n    sql:whereX [\n        sql:operator sql:GreaterThan ;\n        sql:leftOperand [ sql:columnName \"batch_id\" ] ;\n        sql:rightOperand [ sql:literalValue \"alpha\" ]\n    ] .\n"
    );
    match compile_turtle(&doc) {
        Ok(compiled) => panic!(
            "silent-ignore defect: unknown predicate sql:whereX admitted as `{}`:\n{doc}",
            compiled.sql
        ),
        Err(err) => {
            let message = err.to_string();
            assert!(
                message.contains("REFUSAL-01"),
                "unknown predicate must refuse typed REFUSAL-01, got: {message}"
            );
        }
    }
    // FLIP: the same clause under the KNOWN predicate sql:where compiles.
    assert_flip_compiles(
        &doc,
        ("sql:whereX [", "sql:where ["),
        "e3-unknown-predicate flip",
    );
}

/// E4 — malformed limit: a non-integer sql:limit must refuse, never silently
/// drop the clause (unbounded scan from a SHACL-clean document).
#[test]
fn e4_malformed_limit() {
    let doc = format!(
        "{HEAD}\nex:Q a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"telemetry_events\" ] ;\n    sql:selectList ( [ sql:columnName \"batch_id\" ] ) ;\n    sql:limit \"ten\"^^xsd:integer .\n"
    );
    assert_refusal_01(&doc, "sql:limit 'ten'");
    // FLIP: well-formed limit — the twin must compile with the clause present.
    assert_flip_compiles(
        &doc,
        ("\"ten\"^^xsd:integer", "\"10\"^^xsd:integer"),
        "e4-limit flip",
    );
}

/// E4 — malformed offset: same law, negative integer.
#[test]
fn e4_malformed_offset() {
    let doc = format!(
        "{HEAD}\nex:Q a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"telemetry_events\" ] ;\n    sql:selectList ( [ sql:columnName \"batch_id\" ] ) ;\n    sql:offset \"-1\"^^xsd:integer .\n"
    );
    assert_refusal_01(&doc, "sql:offset '-1'");
    // FLIP: well-formed offset — the twin must compile with the clause present.
    assert_flip_compiles(
        &doc,
        ("\"-1\"^^xsd:integer", "\"5\"^^xsd:integer"),
        "e4-offset flip",
    );
}

/// E5 — dual operand key: a binary expression carrying both sql:leftOperand
/// and sql:left mixes operand names to launder a term past review.
#[test]
fn e5_left_leftoperand_dual_key() {
    let doc = format!(
        "{HEAD}\nex:Q a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"telemetry_events\" ] ;\n    sql:selectList ( [ sql:columnName \"batch_id\" ] ) ;\n    sql:whereClause [\n        sql:operator sql:GreaterThan ;\n        sql:leftOperand [ sql:columnName \"latency_ms\" ] ;\n        sql:left [ sql:columnName \"batch_id\" ] ;\n        sql:rightOperand [ sql:literalValue \"400\"^^xsd:integer ]\n    ] .\n"
    );
    assert_refusal_01(&doc, "both sql:leftOperand and sql:left");
    // FLIP: drop the dual sql:left key — the twin must compile.
    assert_flip_compiles(
        &doc,
        (" sql:left [ sql:columnName \"batch_id\" ] ;", ""),
        "e5 flip",
    );
}

/// Scope fuzz finding — sql:scope on an operator node: scope is only defined
/// on column / output-expression nodes, never on literals or operators.
#[test]
fn scope_on_operator_node_refuses() {
    let doc = format!(
        "{HEAD}\nex:Q a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"telemetry_events\" ] ;\n    sql:selectList ( [ sql:columnName \"batch_id\" ] ) ;\n    sql:whereClause [\n        sql:operator sql:GreaterThan ;\n        sql:scope sql:ViewScope ;\n        sql:leftOperand [ sql:columnName \"latency_ms\" ] ;\n        sql:rightOperand [ sql:literalValue \"400\"^^xsd:integer ]\n    ] .\n"
    );
    assert_refusal_01(&doc, "only defined on column or output-expression nodes");
    // FLIP: remove the misplaced scope — the twin must compile.
    assert_flip_compiles(
        &doc,
        ("        sql:scope sql:ViewScope ;\n", ""),
        "scope-op flip",
    );
}

/// Scope fuzz finding — dual scope: sql:scope is single-cardinality; two
/// values on one node must refuse, never last-wins.
#[test]
fn dual_scope_on_column_refuses() {
    let doc = format!(
        "{HEAD}\nex:Q a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"telemetry_events\" ] ;\n    sql:selectList ( [ sql:columnName \"batch_id\" ] ) ;\n    sql:whereClause [\n        sql:operator sql:GreaterThan ;\n        sql:leftOperand [ sql:columnName \"latency_ms\" ; sql:scope sql:ViewScope , sql:ProcessScope ] ;\n        sql:rightOperand [ sql:literalValue \"400\"^^xsd:integer ]\n    ] .\n"
    );
    assert_refusal_01(&doc, "carries 2 sql:scope values");
    // FLIP: drop the scope entirely — a single ViewScope here would still
    // refuse (REFUSAL-03), so the lawful twin carries none.
    assert_flip_compiles(
        &doc,
        (
            " sql:columnName \"latency_ms\" ; sql:scope sql:ViewScope , sql:ProcessScope",
            " sql:columnName \"latency_ms\"",
        ),
        "dual-scope flip",
    );
}

/// Scope integrity (section 1.3/5): a sql:ViewScope column inside a
/// predicate is processed, not viewed — REFUSAL-03.
#[test]
fn view_scope_column_in_where_refuses_03() {
    let doc = format!(
        "{HEAD}\nex:Q a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"telemetry_events\" ] ;\n    sql:selectList ( [ sql:columnName \"batch_id\" ] ) ;\n    sql:whereClause [\n        sql:operator sql:GreaterThan ;\n        sql:leftOperand [ sql:columnName \"latency_ms\" ; sql:scope sql:ViewScope ] ;\n        sql:rightOperand [ sql:literalValue \"400\"^^xsd:integer ]\n    ] .\n"
    );
    assert_refusal_03(&doc, "sql:ViewScope but sits inside a predicate");
    // FLIP: drop the scope from the predicate column — the twin must compile.
    assert_flip_compiles(
        &doc,
        (
            " sql:columnName \"latency_ms\" ; sql:scope sql:ViewScope",
            " sql:columnName \"latency_ms\"",
        ),
        "view-scope flip",
    );
}

/// Scope integrity (section 1.3): a sql:ProcessScope column in the
/// projection list would cross into the consumer's view — REFUSAL-03.
#[test]
fn process_scope_column_in_select_refuses_03() {
    let doc = format!(
        "{HEAD}\nex:Q a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"telemetry_events\" ] ;\n    sql:selectList ( [ sql:columnName \"internal_key\" ; sql:scope sql:ProcessScope ] ) .\n"
    );
    assert_refusal_03(&doc, "sql:ProcessScope but sits in the projection list");
    // FLIP: drop the scope — the twin must compile.
    assert_flip_compiles(
        &doc,
        (" ; sql:scope sql:ProcessScope", ""),
        "process-scope flip",
    );
}

/// Scope integrity (TC-MUT-01 corpus row, section 1.3): an aggregate
/// function node carrying `sql:scope sql:ProcessScope` inside a whereClause
/// operand — the aggregate itself is process-only (REFUSAL-03), and a
/// ProcessScope marker must not launder it into view territory.
#[test]
fn aggregate_scope_process_in_where_refuses_03() {
    let doc = format!(
        "{HEAD}\nex:Q a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"telemetry_events\" ] ;\n    sql:selectList ( [ sql:columnName \"batch_id\" ] ) ;\n    sql:whereClause [\n        sql:operator sql:GreaterThan ;\n        sql:leftOperand [ sql:function <http://graphlaw.org/functions#COUNT> ; sql:argument [ sql:columnName \"latency_ms\" ] ; sql:scope sql:ProcessScope ] ;\n        sql:rightOperand [ sql:literalValue \"400\"^^xsd:integer ]\n    ] .\n"
    );
    assert_refusal_03(&doc, "aggregate");
    // FLIP: replace the laundered aggregate operand with a plain column —
    // the twin must compile.
    assert_flip_compiles(
        &doc,
        (
            "sql:leftOperand [ sql:function <http://graphlaw.org/functions#COUNT> ; sql:argument [ sql:columnName \"latency_ms\" ] ; sql:scope sql:ProcessScope ]",
            "sql:leftOperand [ sql:columnName \"latency_ms\" ]",
        ),
        "aggregate-scope flip",
    );
}

/// TC-MUT-01 arm B conjunction (section 1.3/5): the full conjunction — a
/// whereClause operand that is an aggregate function node carrying
/// `sql:scope sql:ProcessScope`. The aggregate itself is process-only in a
/// predicate position (REFUSAL-03 aggregate gate); the ProcessScope marker
/// must not launder it into view territory.
#[test]
fn tcb_aggregate_node_process_scope_in_where_refuses_03() {
    let doc = format!(
        "{HEAD}\nex:Q a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"telemetry_events\" ] ;\n    sql:selectList ( [ sql:columnName \"batch_id\" ] ) ;\n    sql:whereClause [\n        sql:operator sql:GreaterThan ;\n        sql:leftOperand [ sql:function <http://graphlaw.org/functions#SUM> ; sql:argument [ sql:columnName \"latency_ms\" ] ; sql:scope sql:ProcessScope ] ;\n        sql:rightOperand [ sql:literalValue \"400\"^^xsd:integer ]\n    ] .\n"
    );
    assert_refusal_03(&doc, "aggregate");
    // FLIP: replace the laundered aggregate operand with a plain column.
    assert_flip_compiles(
        &doc,
        (
            "sql:leftOperand [ sql:function <http://graphlaw.org/functions#SUM> ; sql:argument [ sql:columnName \"latency_ms\" ] ; sql:scope sql:ProcessScope ]",
            "sql:leftOperand [ sql:columnName \"latency_ms\" ]",
        ),
        "tcb conjunction flip",
    );
}

/// RUST-7 control (wrong-reason-twin repair): an aggregate WITHOUT any
/// sql:scope marker in a whereClause operand must STILL refuse REFUSAL-03 —
/// proving the marker is inert in predicate position and the two
/// `*_aggregate_*_process_scope_in_where_refuses_03` twins above are gated
/// by the aggregate-in-predicate law, not merely by the aggregate's
/// presence. (Their old flips changed two dimensions at once: aggregate ->
/// plain column AND scope dropped.)
#[test]
fn aggregate_unscoped_in_where_still_refuses_03() {
    let doc = format!(
        "{HEAD}\nex:Q a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"telemetry_events\" ] ;\n    sql:selectList ( [ sql:columnName \"batch_id\" ] ) ;\n    sql:whereClause [\n        sql:operator sql:GreaterThan ;\n        sql:leftOperand [ sql:function <http://graphlaw.org/functions#COUNT> ; sql:argument [ sql:columnName \"latency_ms\" ] ] ;\n        sql:rightOperand [ sql:literalValue \"400\"^^xsd:integer ]\n    ] .\n"
    );
    assert_refusal_03(&doc, "aggregate");
    // FLIP: the same aggregate node in a LAWFUL position (HAVING) must
    // compile — see aggregate_process_scope_in_having_compiles.
}

/// RUST-7 positive control: an aggregate node CARRYING
/// `sql:scope sql:ProcessScope` in a lawful HAVING aggregate must COMPILE —
/// the marker does not launder an otherwise-lawful aggregate, and HAVING's
/// legality is not revoked by the marker's presence. Mirrors the structure
/// of `tcb_process_scope_argument_in_legal_having_aggregate_refuses_03`
/// (which refuses when the scope sits on the ARGUMENT column, per section
/// 1.3), isolating the marker-on-aggregate-node dimension alone.
#[test]
fn aggregate_process_scope_in_having_compiles() {
    let doc = format!(
        "{HEAD}\nex:Q a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"telemetry_events\" ] ;\n    sql:selectList ( [ sql:columnName \"batch_id\" ] ) ;\n    sql:havingClause [\n        sql:operator sql:GreaterThan ;\n        sql:leftOperand [ sql:function <http://graphlaw.org/functions#COUNT> ; sql:argument [ sql:columnName \"latency_ms\" ] ; sql:scope sql:ProcessScope ] ;\n        sql:rightOperand [ sql:literalValue \"4\"^^xsd:integer ]\n    ] .\n"
    );
    let compiled = compile_turtle(&doc).unwrap_or_else(|e| {
        panic!("rust7: lawful HAVING aggregate with ProcessScope marker must compile, got: {e}")
    });
    assert!(
        compiled.sql.contains("COUNT") && compiled.sql.contains("HAVING"),
        "rust7: expected HAVING ... COUNT(...) in compiled SQL, got: {}",
        compiled.sql
    );
}

/// TC-MUT-01 arm B variant (section 1.3): an aggregate ARGUMENT column
/// carrying `sql:scope sql:ProcessScope` inside a legal HAVING aggregate —
/// the aggregate is in a lawful position, but the explicitly scoped argument
/// is a laundering marker and must refuse typed, not inherit HAVING's
/// legality.
#[test]
fn tcb_process_scope_argument_in_legal_having_aggregate_refuses_03() {
    let doc = format!(
        "{HEAD}\nex:Q a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"telemetry_events\" ] ;\n    sql:selectList ( [ sql:columnName \"batch_id\" ] ) ;\n    sql:havingClause [\n        sql:operator sql:GreaterThan ;\n        sql:leftOperand [ sql:function <http://graphlaw.org/functions#COUNT> ; sql:argument [ sql:columnName \"latency_ms\" ; sql:scope sql:ProcessScope ] ] ;\n        sql:rightOperand [ sql:literalValue \"4\"^^xsd:integer ]\n    ] .\n"
    );
    assert_refusal_03(&doc, "aggregate argument");
    // FLIP: drop the explicit scope from the aggregate argument — the lawful
    // twin (legal HAVING aggregate, unscoped argument) must compile.
    assert_flip_compiles(
        &doc,
        (
            " sql:columnName \"latency_ms\" ; sql:scope sql:ProcessScope ] ]",
            " sql:columnName \"latency_ms\" ] ]",
        ),
        "tcb having-argument flip",
    );
}

/// Depth-guard poka-yoke court (p-FMEA open finding): the expression-depth
/// channel (RecursionLimitExceeded(32) at src/sql_ast.rs:719-720, serialize
/// MAX_DEPTH at :573-574) previously had NO flipping court — deleting the
/// guard passed the whole suite. This court flips: a 40-deep nested
/// sql:leftOperand chain must refuse typed RecursionLimitExceeded, while the
/// benign shallow twin (identical shape, 4 levels) must compile. Removing or
/// raising the guard past 40 makes this fail.
#[test]
fn deep_nested_expression_refuses_recursion_limit() {
    const DEEP_LEVELS: usize = 40;

    let mut nested = String::from(
        "[ sql:operator sql:GreaterThan ;\n            sql:rightOperand [ sql:literalValue \"1\"^^xsd:integer ] ;\n            sql:leftOperand [ sql:columnName \"latency_ms\" ] ]",
    );
    for _ in 1..DEEP_LEVELS {
        nested = format!(
            "[ sql:operator sql:GreaterThan ;\n            sql:rightOperand [ sql:literalValue \"1\"^^xsd:integer ] ;\n            sql:leftOperand {} ]",
            nested
        );
    }
    let doc = format!(
        "{HEAD}\nex:Q a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"telemetry_events\" ] ;\n    sql:selectList ( [ sql:columnName \"batch_id\" ] ) ;\n    sql:whereClause {}\n .\n",
        nested
    );

    match compile_turtle(&doc) {
        Ok(compiled) => panic!(
            "depth-guard defect: {}-deep nested expression silently admitted as `{}` (guard at src/sql_ast.rs:719 removed or raised):",
            DEEP_LEVELS,
            truncate_for_panic(&compiled.sql)
        ),
        Err(err) => {
            let message = err.to_string();
            assert!(
                message.contains("Recursion depth limit exceeded (32)"),
                "wrong refusal class, expected typed RecursionLimitExceeded(32), got: {message}"
            );
            assert!(
                !message.contains("REFUSAL-01"),
                "depth refusal must be the typed recursion limit, not a laundering refusal: {message}"
            );
        }
    }

    // FLIP: the benign shallow twin (same shape, 4 nested levels) must
    // compile — proving the refusal is attributable to depth alone.
    let mut shallow = String::from(
        "[ sql:operator sql:GreaterThan ;\n            sql:rightOperand [ sql:literalValue \"1\"^^xsd:integer ] ;\n            sql:leftOperand [ sql:columnName \"latency_ms\" ] ]",
    );
    for _ in 1..4 {
        shallow = format!(
            "[ sql:operator sql:GreaterThan ;\n            sql:rightOperand [ sql:literalValue \"1\"^^xsd:integer ] ;\n            sql:leftOperand {} ]",
            shallow
        );
    }
    // The deep doc's innermost node is a literal-only leaf; build the benign
    // twin directly instead of string surgery to keep the shape identical.
    let benign = format!(
        "{HEAD}\nex:Q a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"telemetry_events\" ] ;\n    sql:selectList ( [ sql:columnName \"batch_id\" ] ) ;\n    sql:whereClause {}\n .\n",
        shallow
    );
    let compiled = compile_turtle(&benign)
        .unwrap_or_else(|e| panic!("depth flip: benign shallow twin must compile, got: {e}"));
    assert!(
        compiled.sql.contains("SELECT"),
        "depth flip: benign twin must project a SELECT, got: {}",
        compiled.sql
    );
}

/// Panic-message helper: keep a leaking compile's SQL bounded in court output.
fn truncate_for_panic(sql: &str) -> &str {
    if sql.len() > 200 { &sql[..200] } else { sql }
}

/// Shape-layer channel witness (TC-MUT-01 M2 flip court): a from-less
/// SELECT must be refused by the SHAPE layer (SHACL structural violation
/// from SelectStatementShape's from-clause sh:or), not only by the
/// reader's code-side backstop — defense-in-depth on BOTH channels.
/// Under the M2 shape mutation (from-clause sh:minCount 1 removed) the
/// shape channel goes silent and this court flips.
#[test]
fn fromless_select_refuses_via_shacl_shape_channel() {
    let doc = format!(
        "{HEAD}\nex:Q a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:selectList ( [ sql:columnName \"batch_id\" ] ) .\n"
    );
    match compile_turtle(&doc) {
        Ok(compiled) => panic!(
            "laundering defect: from-less SELECT silently admitted as `{}`:\n{doc}",
            compiled.sql
        ),
        Err(err) => {
            let message = err.to_string();
            assert!(
                message.contains("SHACL AST structural violation") && message.contains("Node <"),
                "from-less SELECT refused off the shape channel (no SHACL-engine Node diagnostics; shape layer not load-bearing): {message}"
            );
            assert!(
                message.contains("REFUSAL-01"),
                "wrong refusal class, expected REFUSAL-01, got: {message}"
            );
        }
    }
    // FLIP: add the from clause — the twin must compile.
    let benign = doc.replace(
        "    sql:selectList",
        "    sql:fromTable [ sql:tableName \"telemetry_events\" ] ;\n    sql:selectList",
    );
    let compiled = compile_turtle(&benign)
        .unwrap_or_else(|e| panic!("fromless flip: benign twin must compile, got: {e}"));
    assert!(
        compiled.sql.contains("SELECT"),
        "fromless flip: benign twin must project a SELECT, got: {}",
        compiled.sql
    );
}

/// D1 (scope fuzz finding, resilliant-residual hunt): sql:scope on a
/// sql:joinTable table-reference node is a term out of place — REFUSAL-01,
/// never silently ignored (the old compiler read only tableName/tableAlias
/// and dropped the marker on the floor).
#[test]
fn d1_scope_on_join_table_refuses() {
    let doc = format!(
        "{HEAD}\nex:Q a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"telemetry_events\" ] ;\n    sql:selectList ( [ sql:columnName \"batch_id\" ] ) ;\n    sql:joinClause [\n        sql:joinType sql:InnerJoin ;\n        sql:joinTable [ sql:tableName \"batches\" ; sql:scope sql:ViewScope ] ;\n        sql:joinCondition [\n            sql:operator sql:GreaterThan ;\n            sql:leftOperand [ sql:columnName \"batch_id\" ] ;\n            sql:rightOperand [ sql:literalValue \"400\"^^xsd:integer ]\n        ]\n    ] .\n"
    );
    assert_refusal_01(&doc, "term out of place");
    // FLIP: the benign twin — the SAME joinTable WITHOUT the scope marker —
    // must compile, proving the refusal is attributable to the scope marker
    // alone and not to the join shape.
    assert_flip_compiles(
        &doc,
        (
            " sql:tableName \"batches\" ; sql:scope sql:ViewScope",
            " sql:tableName \"batches\"",
        ),
        "d1 flip",
    );
}

/// D1 variant: the same misplaced scope on the sql:fromTable target itself.
#[test]
fn d1_scope_on_from_table_refuses() {
    let doc = format!(
        "{HEAD}\nex:Q a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"telemetry_events\" ; sql:scope sql:ProcessScope ] ;\n    sql:selectList ( [ sql:columnName \"batch_id\" ] ) .\n"
    );
    assert_refusal_01(&doc, "term out of place");
    // FLIP: drop the scope marker — the twin must compile.
    assert_flip_compiles(
        &doc,
        (
            "\"telemetry_events\" ; sql:scope sql:ProcessScope",
            "\"telemetry_events\"",
        ),
        "d1-from flip",
    );
}

/// D3 (residual hunt): an aggregate under sql:orderBy wrapped in
/// sql:expression (no sql:aliasName) previously died as an untyped
/// "Unexpected terminal node" — the ScopePos::OrderBy aggregate arm was
/// unreachable. The wrapper is now transparent: the aggregate extracts in
/// the OrderBy position (lawful, rec 5) and must COMPILE, and the laundering
/// variant (sql:expression alongside another discriminator) must refuse
/// REFUSAL-01.
#[test]
fn d3_order_by_aggregate_expression_wrapper_compiles() {
    let doc = format!(
        "{HEAD}\nex:Q a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"telemetry_events\" ] ;\n    sql:selectList ( [ sql:columnName \"batch_id\" ] ) ;\n    sql:orderBy ( [ sql:expression [ sql:function <http://graphlaw.org/functions#COUNT> ; sql:argument [ sql:columnName \"latency_ms\" ] ] ; sql:direction \"DESC\"^^xsd:string ] ) .\n"
    );
    let compiled = compile_turtle(&doc).unwrap_or_else(|e| {
        panic!("d3: order-by aggregate via sql:expression must compile, got: {e}")
    });
    assert!(
        compiled.sql.contains("COUNT") && compiled.sql.contains("ORDER BY"),
        "d3: expected ORDER BY ... COUNT(...) in compiled SQL, got: {}",
        compiled.sql
    );
}

#[test]
fn d3_expression_wrapper_plus_discriminator_refuses() {
    let doc = format!(
        "{HEAD}\nex:Q a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"telemetry_events\" ] ;\n    sql:selectList ( [ sql:columnName \"batch_id\" ; sql:expression [ sql:columnName \"secret\" ] ] ) .\n"
    );
    assert_refusal_01(&doc, "alongside another expression discriminator");
    // FLIP: drop the laundered sql:expression — the twin must compile.
    assert_flip_compiles(
        &doc,
        (
            " sql:columnName \"batch_id\" ; sql:expression [ sql:columnName \"secret\" ]",
            " sql:columnName \"batch_id\"",
        ),
        "d3-laundering flip",
    );
}
