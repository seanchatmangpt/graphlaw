//! SQLRDF-27 dialect gap #1 courts: UNION [ALL] [BY NAME], INTERSECT, EXCEPT
//! as sql-ast nodes over two nested SelectStatement operands.
//!
//! Chicago discipline: every semantic claim below is a real frozen-duckdb
//! readback through `compile_turtle` + `eval_ast_query` — no mock, no twin
//! SQL, no sleeps. The refusal twins assert the typed markers from
//! `docs/refusals.md` (REFUSAL-01 arms 53-56), never a generic DB error.

use graphlaw::duckdb_eval::{ExecutionReceipt, GraphLawDuckDB};
use graphlaw::sql_ast::{compile_turtle, SqlLiteral};

// -----------------------------------------------------------------------------
// Fixtures
// -----------------------------------------------------------------------------

/// Standard directional fixture: `a = {1,2,3}`, `b = {2,3,4}` (as multisets),
/// so every set operation has a DISTINCT expected value:
/// UNION -> {1,2,3,4}, INTERSECT -> {2,3}, EXCEPT -> {1}.
fn directional_doc(op: &str, extra_pred: &str) -> String {
    format!(
        r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/queries#> .
ex:U a sql:SetOperationStatement ;
    sql:statementIndex 1 ;
    sql:setOperator "{op}"{extra} ;
    sql:setLeft [ a sql:SelectStatement ;
        sql:statementIndex 0 ;
        sql:fromTable [ sql:tableName "setop_a" ] ;
        sql:selectList ( [ sql:columnName "v" ] ) ] ;
    sql:setRight [ a sql:SelectStatement ;
        sql:statementIndex 0 ;
        sql:fromTable [ sql:tableName "setop_b" ] ;
        sql:selectList ( [ sql:columnName "v" ] ) ] .
"#,
        op = op,
        extra = extra_pred
    )
}

fn db() -> GraphLawDuckDB {
    let db = GraphLawDuckDB::open_in_memory().unwrap();
    db.raw_connection()
        .execute_batch(
            "CREATE TABLE setop_a (v INTEGER);
             CREATE TABLE setop_b (v INTEGER);
             INSERT INTO setop_a VALUES (1),(2),(3);
             INSERT INTO setop_b VALUES (2),(3),(4);",
        )
        .unwrap();
    db
}

/// The query's own output values for one column, read back from the execution
/// receipt's row projection (real frozen-duckdb rows, sorted).
fn column_values(receipt: &ExecutionReceipt, column: &str) -> Vec<String> {
    let suffix = format!("/{column}");
    let mut values: Vec<String> = receipt
        .result_triples
        .iter()
        .filter(|t| t.predicate.ends_with(&suffix))
        .filter_map(|t| match &t.object {
            purrdf::TermValue::Literal { lexical_form, .. } => Some(lexical_form.clone()),
            _ => None,
        })
        .collect();
    values.sort();
    values
}

// -----------------------------------------------------------------------------
// Semantics courts (real frozen-duckdb readbacks)
// -----------------------------------------------------------------------------

/// UNION dedups; UNION ALL preserves duplicates. Both compile from RDF and
/// execute on the same seeded connection; the compiled SQL text carries the
/// combinator it was admitted under.
#[test]
fn union_dedups_union_all_preserves() {
    let db = db();

    let union = compile_turtle(&directional_doc("UNION", "")).unwrap();
    assert!(
        union.sql.contains(" UNION (") && !union.sql.contains(" UNION ALL"),
        "UNION must not render ALL: {}",
        union.sql
    );
    let r = db
        .eval_ast_query("urn:ex:u", &union, "urn:s:", "urn:p:")
        .unwrap();
    assert_eq!(r.rows_emitted, 4, "UNION = {{1,2,3,4}}: {}", union.sql);
    assert_eq!(column_values(&r, "v"), vec!["1", "2", "3", "4"]);

    let all = compile_turtle(&directional_doc("UNION", " ;\n    sql:setAll true")).unwrap();
    assert!(all.sql.contains(" UNION ALL ("), "{}", all.sql);
    let r = db
        .eval_ast_query("urn:ex:uall", &all, "urn:s:", "urn:p:")
        .unwrap();
    assert_eq!(r.rows_emitted, 6, "UNION ALL = 3+3 multiset rows");
    assert_eq!(column_values(&r, "v"), vec!["1", "2", "2", "3", "3", "4"]);

    // Distinct receipt identity: flipping the flag changes Canon(G_AST), so
    // the two documents must not share a receipt anchor (sql-rdf Rec 1.4).
    assert_ne!(union.receipt_state, all.receipt_state);
}

/// INTERSECT keeps the shared rows; EXCEPT keeps the left-only rows. The
/// fixture is directional (a={1,2,3}, b={2,3,4}) so a swapped-operand mutation
/// flips the EXCEPT readback from {1} to {4}.
#[test]
fn intersect_except_directional_readbacks() {
    let db = db();

    let inter = compile_turtle(&directional_doc("INTERSECT", "")).unwrap();
    assert!(inter.sql.contains(" INTERSECT ("), "{}", inter.sql);
    let r = db
        .eval_ast_query("urn:ex:i", &inter, "urn:s:", "urn:p:")
        .unwrap();
    assert_eq!(r.rows_emitted, 2, "INTERSECT = {{2,3}}");
    assert_eq!(column_values(&r, "v"), vec!["2", "3"]);

    let except = compile_turtle(&directional_doc("EXCEPT", "")).unwrap();
    assert!(except.sql.contains(" EXCEPT ("), "{}", except.sql);
    let r = db
        .eval_ast_query("urn:ex:e", &except, "urn:s:", "urn:p:")
        .unwrap();
    assert_eq!(r.rows_emitted, 1, "EXCEPT = {{1}} (left minus right)");
    assert_eq!(column_values(&r, "v"), vec!["1"]);
}

/// Engine-witnessed law: `ALL` is legal with every kind (INTERSECT ALL keeps
/// multiset duplicates), but `BY NAME` pairs only with UNION — the engine
/// itself parses `INTERSECT BY NAME` as an error, so the compiler refuses it
/// typed at compile time (court below).
#[test]
fn intersect_all_readback_all_is_engine_legal_for_every_kind() {
    let db = db();
    let doc = directional_doc("INTERSECT", " ;\n    sql:setAll true");
    let compiled = compile_turtle(&doc).unwrap();
    assert!(compiled.sql.contains(" INTERSECT ALL ("), "{}", compiled.sql);
    let r = db
        .eval_ast_query("urn:ex:iall", &compiled, "urn:s:", "urn:p:")
        .unwrap();
    // a={1,2,3}, b={2,3,4}: INTERSECT ALL = {2,3} here (no dupes in fixture).
    assert_eq!(r.rows_emitted, 2);
    assert_eq!(column_values(&r, "v"), vec!["2", "3"]);
}

/// UNION BY NAME matches columns by NAME, not position. Fixture: the right
/// operand's columns are physically reversed (b, a), so positional UNION
/// would pair a-values with b-values — the readback proves name-matching.
#[test]
fn union_by_name_matches_columns_by_name() {
    let db = GraphLawDuckDB::open_in_memory().unwrap();
    db.raw_connection()
        .execute_batch(
            "CREATE TABLE setop_l (a INTEGER, b INTEGER);
             CREATE TABLE setop_r (b INTEGER, a INTEGER);
             INSERT INTO setop_l VALUES (1,10),(2,20);
             INSERT INTO setop_r VALUES (30,3),(10,1);",
        )
        .unwrap();

    let doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:U a sql:SetOperationStatement ;
    sql:statementIndex 1 ;
    sql:setOperator "UNION" ;
    sql:byName true ;
    sql:setLeft [ a sql:SelectStatement ;
        sql:statementIndex 0 ;
        sql:fromTable [ sql:tableName "setop_l" ] ;
        sql:selectList ( [ sql:columnName "a" ] [ sql:columnName "b" ] ) ] ;
    sql:setRight [ a sql:SelectStatement ;
        sql:statementIndex 0 ;
        sql:fromTable [ sql:tableName "setop_r" ] ;
        sql:selectList ( [ sql:columnName "b" ] [ sql:columnName "a" ] ) ] .
"#;
    let compiled = compile_turtle(doc).unwrap();
    assert!(compiled.sql.contains(" UNION BY NAME ("), "{}", compiled.sql);

    let r = db
        .eval_ast_query("urn:ex:byname", &compiled, "urn:s:", "urn:p:")
        .unwrap();
    // Name-paired rows: (1,10),(2,20),(3,30) — the (10,1) duplicate dedups.
    assert_eq!(r.rows_emitted, 3, "BY NAME pairs by column name: {}", compiled.sql);
    assert_eq!(column_values(&r, "a"), vec!["1", "2", "3"]);
    assert_eq!(column_values(&r, "b"), vec!["10", "20", "30"]);

    // The positional twin on the same tables pairs by POSITION instead:
    // 4 rows with a-column {1,2,10,30} — the mutation witness for BY NAME.
    let positional = compiled.sql.replace(" UNION BY NAME ", " UNION ");
    let mut stmt = db
        .raw_connection()
        .prepare(&positional)
        .expect("positional twin prepares");
    let mut rows = stmt.query([]).unwrap();
    let mut count = 0;
    while rows.next().unwrap().is_some() {
        count += 1;
    }
    assert_eq!(count, 4, "positional UNION does NOT name-pair: 4 rows");
}

/// UNION ALL BY NAME renders and executes (engine-probed legal form).
#[test]
fn union_all_by_name_readback() {
    let db = GraphLawDuckDB::open_in_memory().unwrap();
    db.raw_connection()
        .execute_batch(
            "CREATE TABLE setop_l (a INTEGER, b INTEGER);
             CREATE TABLE setop_r (b INTEGER, a INTEGER);
             INSERT INTO setop_l VALUES (1,10);
             INSERT INTO setop_r VALUES (30,3);",
        )
        .unwrap();
    let doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:U a sql:SetOperationStatement ;
    sql:statementIndex 1 ;
    sql:setOperator "UNION" ;
    sql:setAll true ;
    sql:byName true ;
    sql:setLeft [ a sql:SelectStatement ;
        sql:statementIndex 0 ;
        sql:fromTable [ sql:tableName "setop_l" ] ;
        sql:selectList ( [ sql:columnName "a" ] [ sql:columnName "b" ] ) ] ;
    sql:setRight [ a sql:SelectStatement ;
        sql:statementIndex 0 ;
        sql:fromTable [ sql:tableName "setop_r" ] ;
        sql:selectList ( [ sql:columnName "b" ] [ sql:columnName "a" ] ) ] .
"#;
    let compiled = compile_turtle(doc).unwrap();
    assert!(
        compiled.sql.contains(" UNION ALL BY NAME ("),
        "{}",
        compiled.sql
    );
    let r = db
        .eval_ast_query("urn:ex:allbyname", &compiled, "urn:s:", "urn:p:")
        .unwrap();
    assert_eq!(r.rows_emitted, 2, "ALL keeps both rows");
    assert_eq!(column_values(&r, "a"), vec!["1", "3"]);
    assert_eq!(column_values(&r, "b"), vec!["10", "30"]);
}

// -----------------------------------------------------------------------------
// Refusal courts (typed, before any engine call)
// -----------------------------------------------------------------------------

fn refusal_is(doc: &str, markers: &[&str]) {
    let err = compile_turtle(doc)
        .expect_err("the document must refuse")
        .to_string();
    for m in markers {
        assert!(
            err.contains(m),
            "refusal text must contain '{m}': {err}"
        );
    }
}

/// The missing-side twin: a set operation without sql:setRight is the
/// dedicated REFUSAL-01 arm (SET_OP_NO_RIGHT); without sql:setLeft it is the
/// internal MissingProperty refusal (MERGE-clause precedent). Neither ever
/// reaches the engine.
#[test]
fn missing_right_and_left_side_refuse_typed() {
    let no_right = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:U a sql:SetOperationStatement ;
    sql:statementIndex 1 ;
    sql:setOperator "UNION" ;
    sql:setLeft [ a sql:SelectStatement ;
        sql:statementIndex 0 ;
        sql:fromTable [ sql:tableName "setop_a" ] ;
        sql:selectList ( [ sql:columnName "v" ] ) ] .
"#;
    refusal_is(
        no_right,
        &["[REFUSAL-01] MalformedAstRefusal:", "carries no sql:setRight"],
    );

    let no_left = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:U a sql:SetOperationStatement ;
    sql:statementIndex 1 ;
    sql:setOperator "UNION" ;
    sql:setRight [ a sql:SelectStatement ;
        sql:statementIndex 0 ;
        sql:fromTable [ sql:tableName "setop_b" ] ;
        sql:selectList ( [ sql:columnName "v" ] ) ] .
"#;
    refusal_is(no_left, &["Missing required property", "setLeft"]);
}

/// Unknown combinator token: typed REFUSAL-01 (SET_OP_KIND_TOKEN), never a
/// silent default to UNION.
#[test]
fn unknown_set_operator_token_refuses() {
    let doc = directional_doc("MINUS", "");
    refusal_is(
        &doc,
        &[
            "[REFUSAL-01] MalformedAstRefusal:",
            "is not UNION, INTERSECT or EXCEPT",
        ],
    );
}

/// BY NAME beside INTERSECT: engine-witnessed parser error, refused typed at
/// compile time. The second half executes the same shape as raw SQL on the
/// real connection and asserts the ENGINE refuses it too — the compile-time
/// law is engine-true, not stricter-than-reality decoration.
#[test]
fn by_name_with_intersect_refuses_and_engine_agrees() {
    let doc = directional_doc("INTERSECT", " ;\n    sql:byName true");
    refusal_is(
        &doc,
        &[
            "[REFUSAL-01] MalformedAstRefusal:",
            "pairs only with sql:setOperator UNION",
        ],
    );

    // Engine-true witness on the real frozen-duckdb connection (no mock).
    let db = db();
    let raw = "(SELECT v FROM setop_a) INTERSECT BY NAME (SELECT v FROM setop_b)";
    let prepared = db.raw_connection().prepare(raw);
    assert!(
        prepared.is_err(),
        "the engine now accepts INTERSECT BY NAME — freeze moved; re-pin the SET_OP_BY_NAME_KIND law against the new engine truth"
    );
}

/// A non-boolean flag lexical ("yes") refuses typed (SET_OP_FLAG_LEX); it is
/// never read as absent (silent false) nor coerced.
#[test]
fn malformed_flag_lexical_refuses() {
    let doc = directional_doc("UNION", " ;\n    sql:setAll \"yes\"");
    refusal_is(
        &doc,
        &[
            "[REFUSAL-01] MalformedAstRefusal:",
            "must be an xsd:boolean true/false literal",
        ],
    );
}

/// Duplicate clause predicate on the set-op node: REFUSAL-01
/// (DUP_CLAUSE_PREDICATE), document order never wins.
#[test]
fn duplicate_set_operator_refuses_on_cardinality() {
    let doc = format!(
        r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:U a sql:SetOperationStatement ;
    sql:statementIndex 1 ;
    sql:setOperator "UNION", "INTERSECT" ;
    sql:setLeft [ a sql:SelectStatement ;
        sql:statementIndex 0 ;
        sql:fromTable [ sql:tableName "setop_a" ] ;
        sql:selectList ( [ sql:columnName "v" ] ) ] ;
    sql:setRight [ a sql:SelectStatement ;
        sql:statementIndex 0 ;
        sql:fromTable [ sql:tableName "setop_b" ] ;
        sql:selectList ( [ sql:columnName "v" ] ) ] .
"#
    );
    refusal_is(&doc, &["appears more than once"]);
}

/// D1 closed-predicate gate: an unknown sql: predicate on the set-op node is
/// REFUSAL-01, never dropped on the floor.
#[test]
fn unknown_predicate_on_set_op_node_refuses() {
    let doc = directional_doc("UNION", " ;\n    sql:whereX []");
    refusal_is(&doc, &["unknown sql: predicate"]);
}

/// Missing operator: internal MissingProperty, never a default UNION.
#[test]
fn missing_set_operator_refuses() {
    let doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:U a sql:SetOperationStatement ;
    sql:statementIndex 1 ;
    sql:setLeft [ a sql:SelectStatement ;
        sql:statementIndex 0 ;
        sql:fromTable [ sql:tableName "setop_a" ] ;
        sql:selectList ( [ sql:columnName "v" ] ) ] ;
    sql:setRight [ a sql:SelectStatement ;
        sql:statementIndex 0 ;
        sql:fromTable [ sql:tableName "setop_b" ] ;
        sql:selectList ( [ sql:columnName "v" ] ) ] .
"#;
    refusal_is(doc, &["Missing required property", "setOperator"]);
}

// -----------------------------------------------------------------------------
// Admission / receipt law courts
// -----------------------------------------------------------------------------

/// The admitted set-op carries the graph-anchored receipt state (sealed,
/// canon-backed, non-cacheable composite classification) and executes with a
/// real query hash. C1-a: composite arms never enter the cacheable class.
#[test]
fn set_op_admission_is_sealed_and_non_cacheable() {
    let compiled = compile_turtle(&directional_doc("UNION", "")).unwrap();
    assert!(compiled.verify_seal(), "admission seal verifies");
    assert!(compiled.receipt_state.is_some(), "h_ast anchored");
    assert!(compiled.canon_graph_bytes.is_some(), "canon carried");
    assert!(!compiled.cacheable, "composite arm is not cacheable (C1-a)");
    assert!(compiled.sql.starts_with('('), "{}", compiled.sql);
    assert!(compiled.sql.ends_with(';'), "{}", compiled.sql);

    let db = db();
    let r = db
        .eval_ast_query("urn:ex:seal", &compiled, "urn:s:", "urn:p:")
        .unwrap();
    assert_ne!(r.query_hash, [0u8; 32]);
}

/// The hand-built AST node compiles, seals, and its SQL executes on a real
/// frozen-duckdb connection. (Builtin eval refuses non-graph-backed compiles —
/// RUST-7 — so execution goes through the raw connection directly.)
#[test]
fn hand_built_set_op_compiles_and_executes() {
    use graphlaw::sql_ast::{
        SelectItem, SelectStatement, SetOpKind, SetOperationStatement, SqlExpr, TableRef,
    };
    let operand = |table: &str| SelectStatement {
        ctes: Vec::new(),
        from: TableRef {
            name: table.to_string(),
            alias: None,
            schema: None,
        },
        joins: Vec::new(),
        select_list: vec![SelectItem::Expr(SqlExpr::ColumnRef {
            column: "v".to_string(),
            table_alias: None,
        })],
        where_clause: None,
        group_by: Vec::new(),
        having: None,
        order_by: Vec::new(),
        limit: None,
        offset: None,
        qualify: None,
    };
    let setop = SetOperationStatement {
        left: operand("setop_a"),
        kind: SetOpKind::Except,
        all: false,
        by_name: false,
        right: operand("setop_b"),
    };
    let compiled = setop.compile_set_operation().unwrap();
    assert!(
        compiled.sql.contains(") EXCEPT ("),
        "{}",
        compiled.sql
    );
    // Hand-built compile is raw (no graph anchor) but still sealed.
    assert!(compiled.verify_seal());
    assert!(compiled.receipt_state.is_none());

    let db = db();
    let mut stmt = db
        .raw_connection()
        .prepare(&compiled.sql)
        .expect("hand-built set-op prepares");
    let mut rows = stmt.query([]).unwrap();
    let mut got = Vec::new();
    while let Ok(Some(row)) = rows.next() {
        got.push(row.get::<usize, i64>(0).unwrap().to_string());
    }
    got.sort();
    assert_eq!(got, vec!["1"], "EXCEPT = left minus right = {{1}}");
}

/// The `sql:` prefixed parameters bind positionally across BOTH operands:
/// a literal in the left operand is $1, one in the right operand is $2, and
/// the executed readback proves each filter landed on its own side.
#[test]
fn parameters_span_both_operands_in_order() {
    let doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/queries#> .
ex:U a sql:SetOperationStatement ;
    sql:statementIndex 1 ;
    sql:setOperator "UNION" ;
    sql:setLeft [ a sql:SelectStatement ;
        sql:statementIndex 0 ;
        sql:fromTable [ sql:tableName "setop_a" ] ;
        sql:selectList ( [ sql:columnName "v" ] ) ;
        sql:where [ sql:operator "=" ;
            sql:leftOperand [ sql:columnName "v" ] ;
            sql:rightOperand [ sql:literalValue "1"^^xsd:integer ] ] ] ;
    sql:setRight [ a sql:SelectStatement ;
        sql:statementIndex 0 ;
        sql:fromTable [ sql:tableName "setop_b" ] ;
        sql:selectList ( [ sql:columnName "v" ] ) ;
        sql:where [ sql:operator "=" ;
            sql:leftOperand [ sql:columnName "v" ] ;
            sql:rightOperand [ sql:literalValue "4"^^xsd:integer ] ] ] .
"#;
    let compiled = compile_turtle(doc).unwrap();
    assert_eq!(
        compiled.params,
        vec![SqlLiteral::Integer(1), SqlLiteral::Integer(4)]
    );
    let db = db();
    let r = db
        .eval_ast_query("urn:ex:params", &compiled, "urn:s:", "urn:p:")
        .unwrap();
    assert_eq!(r.rows_emitted, 2, "one filtered row per operand");
    assert_eq!(column_values(&r, "v"), vec!["1", "4"]);
}

/// Placeholder-safety: the operand parenthesization must keep the engine from
/// binding an operand-local LIMIT to the whole compound (hostile nesting —
/// the rendered SQL is what a reviewer reads).
#[test]
fn operand_parentheses_survive_hostile_nesting() {
    let doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/queries#> .
ex:U a sql:SetOperationStatement ;
    sql:statementIndex 1 ;
    sql:setOperator "UNION" ;
    sql:setLeft [ a sql:SelectStatement ;
        sql:statementIndex 0 ;
        sql:fromTable [ sql:tableName "setop_a" ] ;
        sql:selectList ( [ sql:columnName "v" ] ) ;
        sql:limit 1 ] ;
    sql:setRight [ a sql:SelectStatement ;
        sql:statementIndex 0 ;
        sql:fromTable [ sql:tableName "setop_b" ] ;
        sql:selectList ( [ sql:columnName "v" ] ) ] .
"#;
    let compiled = compile_turtle(doc).unwrap();
    // LIMIT stays INSIDE the left operand's parens.
    let left = compiled
        .sql
        .split(" UNION ")
        .next()
        .expect("combinator present");
    assert!(
        left.contains("LIMIT 1") && left.contains('('),
        "LIMIT must stay operand-local: {}",
        compiled.sql
    );
    let db = db();
    let r = db
        .eval_ast_query("urn:ex:paren", &compiled, "urn:s:", "urn:p:")
        .unwrap();
    assert_eq!(r.rows_emitted, 4, "LIMIT 1 on the left operand only");
    assert_eq!(column_values(&r, "v"), vec!["1", "2", "3", "4"]);
}
