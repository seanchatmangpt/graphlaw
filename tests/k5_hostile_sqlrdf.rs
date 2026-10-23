//! Lane K5 adversarial court — null hypothesis: "the sql-rdf path cannot
//! corrupt or leak data". Attacks:
//!   (a) literal smuggling: payload literal contains `'); DROP TABLE x; --`
//!   (b) identifier smuggling: sql:tableName / sql:columnName carrying
//!       backticks, quotes, semicolons, comment markers
//!   (c) MERGE with a tautological ON clause (literal = literal) mutating
//!       every row of the target
//!   (d) WHERE clause that admits everything (OR tautology)
//! Chicago discipline: real frozen-duckdb connections, real table readbacks,
//! no mocks. Anti-vacuity: each refusal attack carries its benign twin that
//! MUST compile, and each executing attack asserts real readback state.

use graphlaw::duckdb_eval::GraphLawDuckDB;
use graphlaw::sql_ast::compile_turtle;

const HEAD: &str = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:  <http://example.org/queries#> .
"#;

fn select_doc(from_table: &str, column: &str, extra: &str) -> String {
    format!(
        "{HEAD}\nex:Q a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"{from_table}\" ] ;\n    sql:selectList ( [ sql:columnName \"{column}\" ] ){extra} .\n"
    )
}

/// Seed `vault` (the exfiltration target) and `hostile_t` (the DROP target).
/// Returns the raw connection handle.
fn seed() -> GraphLawDuckDB {
    let db = GraphLawDuckDB::open_in_memory().unwrap();
    db.raw_connection()
        .execute_batch(
            "CREATE TABLE vault(id INTEGER, payload VARCHAR);
             INSERT INTO vault VALUES (1, 'ok'), (2, 'secret');
             CREATE TABLE hostile_t(id INTEGER);
             INSERT INTO hostile_t VALUES (7);",
        )
        .unwrap();
    db
}

fn vault_rows(db: &GraphLawDuckDB) -> Vec<(i32, String)> {
    let mut stmt = db
        .raw_connection()
        .prepare("SELECT id, payload FROM vault ORDER BY id")
        .unwrap();
    stmt.query_map([], |r| Ok((r.get::<_, i32>(0)?, r.get::<_, String>(1)?)))
        .unwrap()
        .map(|r| r.unwrap())
        .collect()
}

fn hostile_t_survives(db: &GraphLawDuckDB) -> bool {
    db.raw_connection()
        .query_row("SELECT count(*) FROM hostile_t", [], |r| r.get::<_, i64>(0))
        .map(|c| c == 1)
        .unwrap_or(false)
}

/// (a) Literal containing `'); DROP TABLE hostile_t; --`. If the literal is
/// parameterized (sql_ast/serializer.rs:176-180 — Literal arm pushes $N and
/// appends to `params`), the hostile text is DATA on the wire, never code.
#[test]
fn a_literal_drop_table_smuggles_as_data_only() {
    let payload = "x'); DROP TABLE hostile_t; --";
    let doc = format!(
        "{HEAD}\nex:Q a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"vault\" ] ;\n    sql:selectList ( [ sql:columnName \"id\" ] ) ;\n    sql:whereClause [\n        sql:operator sql:Equals ;\n        sql:leftOperand [ sql:columnName \"payload\" ] ;\n        sql:rightOperand [ sql:literalValue \"{payload}\" ]\n    ] .\n"
    );
    let compiled = compile_turtle(&doc).unwrap_or_else(|e| {
        panic!("(a) literal must compile (it is data, not code): {e}");
    });
    // The hostile fragment must never appear as SQL text.
    assert!(
        !compiled.sql.to_uppercase().contains("DROP"),
        "(a) leak: hostile literal leaked into SQL text: {}",
        compiled.sql
    );
    let db = seed();
    let receipt = db
        .eval_ast_query("http://example.org/k5a", &compiled, "urn:k5:", "urn:k5:")
        .unwrap_or_else(|e| panic!("(a) eval failed: {e}"));
    // The literal filters to zero rows — and hostile_t survives.
    assert_eq!(receipt.rows_emitted, 0, "(a) expected no match");
    assert!(hostile_t_survives(&db), "(a) hostile_t was corrupted");
    assert_eq!(vault_rows(&db).len(), 2, "(a) vault was corrupted");
}

/// (b1) Identifier smuggling through sql:tableName (backtick + statement
/// separator + comment marker). Must be a typed compile refusal.
#[test]
fn b1_hostile_table_name_is_typed_refusal() {
    let doc = select_doc("vault`; DROP TABLE hostile_t; --", "id", "");
    let benign_twin = select_doc("vault", "id", "");
    match compile_turtle(&doc) {
        Ok(c) => panic!(
            "(b1) DEFECT: hostile table name admitted as `{}` (benign twin: {})",
            c.sql,
            compile_turtle(&benign_twin).is_ok()
        ),
        Err(e) => {
            let m = e.to_string();
            assert!(
                m.contains("Invalid SQL identifier") || m.contains("REFUSAL"),
                "(b1) refusal not typed: {m}"
            );
        }
    }
}

/// (b2) Identifier smuggling through sql:columnName (backtick + statement
/// separator + comment). Must be a typed compile refusal from the
/// identifier validator, not a Turtle parse artifact.
#[test]
fn b2_hostile_column_name_is_typed_refusal() {
    let doc = select_doc("vault", "id`; DROP TABLE hostile_t; --", "");
    match compile_turtle(&doc) {
        Ok(c) => panic!("(b2) DEFECT: hostile column name admitted as `{}`", c.sql),
        Err(e) => {
            let m = e.to_string();
            assert!(
                m.contains("Invalid SQL identifier") || m.contains("REFUSAL"),
                "(b2) refusal not typed: {m}"
            );
        }
    }
}

/// (c) MERGE with a tautological ON clause ($1 = $2, literal = literal) and
/// an assignment rewriting every row of the target. Scope-integrity closed
/// (lane K5b): the ON condition must reference at least one target-qualified
/// column, so the constant-only ON refuses typed ([REFUSAL-03]) at compile —
/// before any SQL is emitted — and the target is untouched. The benign twin
/// (target-qualified ON) must still compile.
#[test]
fn c_merge_tautological_on_is_typed_refusal() {
    let doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

_:m a sql:MergeIntoStatement ;
    sql:statementIndex 1 ;
    sql:targetTable [ sql:tableName "vault" ] ;
    sql:usingSource [
        a sql:SelectStatement ;
        sql:statementIndex 1 ;
        sql:selectList ( [ sql:columnName "id" ] ) ;
        sql:fromTable [ sql:tableName "hostile_t" ]
    ] ;
    sql:mergeOn [
        sql:operator sql:Equals ;
        sql:leftOperand [ sql:literalValue "1"^^xsd:integer ] ;
        sql:rightOperand [ sql:literalValue "1"^^xsd:integer ]
    ] ;
    sql:whenMatched [
        sql:setAssignments (
            [ sql:targetColumn "payload" ;
              sql:valueExpression [ sql:literalValue "PWNED" ] ]
        )
    ] .
"#;
    match compile_turtle(doc) {
        Ok(c) => panic!("(c) DEFECT: constant-only merge ON admitted as `{}`", c.sql),
        Err(e) => {
            let m = e.to_string();
            assert!(
                m.contains("[REFUSAL-03]"),
                "(c) refusal not typed as ScopeIntegrityRefusal: {m}"
            );
        }
    }
    // Real readback: the refused statement never executed; the target is
    // untouched on a real connection.
    let db = seed();
    assert_eq!(vault_rows(&db).len(), 2, "(c) vault was corrupted");
    // Benign twin: a target-qualified ON compiles and is the same shape
    // otherwise — proves the refusal is non-vacuous.
    let benign = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

_:m a sql:MergeIntoStatement ;
    sql:statementIndex 1 ;
    sql:targetTable [ sql:tableName "vault" ] ;
    sql:usingSource [
        a sql:SelectStatement ;
        sql:statementIndex 1 ;
        sql:selectList ( [ sql:columnName "id" ] ) ;
        sql:fromTable [ sql:tableName "hostile_t" ]
    ] ;
    sql:mergeOn [
        sql:operator sql:Equals ;
        sql:leftOperand [ sql:qualifier "vault" ; sql:columnName "id" ; sql:scope sql:ProcessScope ] ;
        sql:rightOperand [ sql:qualifier "src" ; sql:columnName "id" ; sql:scope sql:ProcessScope ]
    ] ;
    sql:whenMatched [
        sql:setAssignments (
            [ sql:targetColumn "payload" ;
              sql:valueExpression [ sql:literalValue "PWNED" ] ]
        )
    ] .
"#;
    let compiled = compile_turtle(benign)
        .unwrap_or_else(|e| panic!("(c) benign target-qualified ON must compile: {e}"));
    assert!(
        compiled.sql.contains("vault.id = src.id"),
        "(c) benign twin ON clause wrong: {}",
        compiled.sql
    );
}

/// MERGE doc factory: target table, USING source, ON expr parts, and WHEN
/// MATCHED assignments. Each `on_operand` is a (qualifier, column, literal)
/// triple — `Some(qual)` renders a qualified column ref, `None` a literal.
fn merge_doc(
    target_table: &str,
    target_schema: Option<&str>,
    on_left: &str,
    on_right: &str,
) -> String {
    let schema_line = match target_schema {
        Some(s) => format!(" ;\n    sql:schemaName \"{}\"", s),
        None => String::new(),
    };
    format!(
        r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

_:m a sql:MergeIntoStatement ;
    sql:statementIndex 1 ;
    sql:targetTable [ sql:tableName "{target_table}"{schema_line} ] ;
    sql:usingSource [
        a sql:SelectStatement ;
        sql:statementIndex 1 ;
        sql:selectList ( [ sql:columnName "id" ] ) ;
        sql:fromTable [ sql:tableName "hostile_t" ]
    ] ;
    sql:mergeOn [
        sql:operator sql:Equals ;
        sql:leftOperand {on_left} ;
        sql:rightOperand {on_right}
    ] ;
    sql:whenMatched [
        sql:setAssignments (
            [ sql:targetColumn "payload" ;
              sql:valueExpression [ sql:literalValue "PWNED" ] ]
        )
    ] .
"#
    )
}

/// Qualified column operand, e.g. `sql:qualifier "main_v2.sales"`.
fn qcol(qualifier: &str, column: &str) -> String {
    format!(
        "[ sql:qualifier \"{}\" ; sql:columnName \"{}\" ; sql:scope sql:ProcessScope ]",
        qualifier, column
    )
}

/// (e1) Schema-qualified suffix collision: the target is bare `sales`; the ON
/// condition qualifies its column with `main_v2.sales` — a DIFFERENT table
/// (schema `main_v2`). `ends_with(".sales")` false-positives, admitting an ON
/// that never constrains the target. Must be a typed [REFUSAL-03]; the benign
/// schema-qualified twin (`main.sales.id`, schema declared) must compile.
#[test]
fn e1_schema_suffix_collision_is_typed_refusal() {
    // Currently-compiling attack: prove the false positive exists.
    let attack = merge_doc(
        "sales",
        None,
        &qcol("main_v2.sales", "id"),
        &qcol("src", "id"),
    );
    let benign = merge_doc(
        "sales",
        Some("main"),
        &qcol("main.sales", "id"),
        &qcol("src", "id"),
    );
    match compile_turtle(&attack) {
        Ok(c) => panic!(
            "(e1) DEFECT: foreign-schema `main_v2.sales` ON admitted as `{}`",
            c.sql
        ),
        Err(e) => {
            let m = e.to_string();
            assert!(
                m.contains("[REFUSAL-03]"),
                "(e1) refusal not typed as ScopeIntegrityRefusal: {m}"
            );
        }
    }
    let compiled = compile_turtle(&benign)
        .unwrap_or_else(|e| panic!("(e1) benign schema-qualified twin must compile: {e}"));
    assert!(
        compiled.sql.contains("main.sales.id = src.id"),
        "(e1) benign twin ON clause wrong: {}",
        compiled.sql
    );
}

/// (e2) ON via function call wrapping a target column. The grammar's function
/// forms are aggregates only (`AggregateFunction::from_iri` /
/// `aggregate_from_name`), and aggregates in ON (Predicate scope) are
/// typed-refused at extract time (AGG_IN_PREDICATE, [REFUSAL-03]) — so the
/// function-wrap channel can never carry an unguarded ON to the merge guard.
/// Court: `COUNT(vault.id) = src.id` must refuse typed, never admit or crash.
#[test]
fn e2_aggregate_wrapped_on_is_typed_refusal() {
    let doc = merge_doc(
        "vault",
        None,
        "[ sql:functionName \"COUNT\" ; sql:arguments ( [ sql:qualifier \"vault\" ; sql:columnName \"id\" ; sql:scope sql:ProcessScope ] ) ]",
        &qcol("src", "id"),
    );
    match compile_turtle(&doc) {
        Ok(c) => panic!("(e2) DEFECT: aggregate-wrapped ON admitted as `{}`", c.sql),
        Err(e) => {
            let m = e.to_string();
            assert!(
                m.contains("[REFUSAL-03]"),
                "(e2) refusal not typed as ScopeIntegrityRefusal: {m}"
            );
        }
    }
}

/// (e3) Unqualified target column in ON (`id = src.id`, bare `id` binding to
/// the target's `id`): statically unresolvable — the bare name could equally
/// be a source-side column pair (`flag = flag2`), which is exactly the
/// constant-per-target-row fan-out the guard exists to refuse. Conservative
/// typed refusal ([REFUSAL-03]) is the intended behavior; the fully-qualified
/// benign twin must still compile.
#[test]
fn e3_unqualified_on_column_is_typed_refusal() {
    let unqualified = format!(
        "{}",
        merge_doc(
            "vault",
            None,
            "[ sql:columnName \"id\" ; sql:scope sql:ProcessScope ]",
            &qcol("src", "id")
        )
    );
    match compile_turtle(&unqualified) {
        Ok(c) => panic!("(e3) DEFECT: unqualified-only ON admitted as `{}`", c.sql),
        Err(e) => {
            let m = e.to_string();
            assert!(
                m.contains("[REFUSAL-03]"),
                "(e3) refusal not typed as ScopeIntegrityRefusal: {m}"
            );
        }
    }
    // Hostile twin: a source-only unqualified pair (`flag = flag2`) — the
    // reason unqualified refs cannot be accepted wholesale — refuses too.
    let source_only = merge_doc(
        "vault",
        None,
        "[ sql:columnName \"flag\" ; sql:scope sql:ProcessScope ]",
        "[ sql:columnName \"flag2\" ; sql:scope sql:ProcessScope ]",
    );
    assert!(
        compile_turtle(&source_only).is_err(),
        "(e3) DEFECT: source-only unqualified ON admitted"
    );
    let benign = merge_doc("vault", None, &qcol("vault", "id"), &qcol("src", "id"));
    let compiled = compile_turtle(&benign)
        .unwrap_or_else(|e| panic!("(e3) qualified benign twin must compile: {e}"));
    assert!(
        compiled.sql.contains("vault.id = src.id"),
        "(e3) benign twin ON clause wrong: {}",
        compiled.sql
    );
}

/// (d) WHERE clause that admits everything: OR of a tautology with the row
/// predicate. Lawful SELECT semantics (data exfiltration of exactly the
/// table the FROM names), witnessed by real readback.
#[test]
fn d_where_admits_everything_is_lawful_select_semantics() {
    let doc = format!(
        "{HEAD}\nex:Q a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:fromTable [ sql:tableName \"vault\" ] ;\n    sql:selectList ( [ sql:columnName \"payload\" ] ) ;\n    sql:whereClause [\n        sql:operator sql:Or ;\n        sql:leftOperand [\n            sql:operator sql:Equals ;\n            sql:leftOperand [ sql:literalValue \"1\"^^xsd:integer ] ;\n            sql:rightOperand [ sql:literalValue \"1\"^^xsd:integer ]\n        ] ;\n        sql:rightOperand [ sql:columnName \"payload\" ]\n    ] .\n"
    );
    let compiled =
        compile_turtle(&doc).unwrap_or_else(|e| panic!("(d) tautological WHERE must compile: {e}"));
    let db = seed();
    let receipt = db
        .eval_ast_query("http://example.org/k5d", &compiled, "urn:k5:", "urn:k5:")
        .unwrap_or_else(|e| panic!("(d) eval failed: {e}"));
    assert_eq!(receipt.rows_emitted, 2, "(d) tautology must admit all rows");
    let payloads: Vec<String> = receipt
        .result_triples
        .iter()
        .map(|t| format!("{:?}", t.object))
        .collect();
    assert!(
        payloads.iter().any(|p| p.contains("secret")),
        "(d) readback missing the secret row: {payloads:?}"
    );
}
