//! Demonstrates: every typed refusal class in docs/refusals.md (REFUSAL-01
//! MalformedAst, REFUSAL-02 UnimplementedStatement, REFUSAL-03 ScopeIntegrity,
//! REFUSAL-04 Capability, plus admission-seal / canonicalization / unclassified
//! gaps), triggered FOR REAL via minimal inputs, printing the exact
//! user-visible message for each.
//!
//! Run: cargo run --example refusal_probe
//! Output: one `=== <probe label> ===` block per refusal with the verbatim
//! message and a `[exit-class MATCH|MISS]` line, then
//! `ALL REFUSAL PROBES MATCHED`; exit 0 iff every probe fired its expected
//! class, exit 1 with `MISSED: [...]` on stderr otherwise.
//! Court: none; documents docs/refusal-playbook.md (refusal-message capture
//! probe; assertion-backed courts live in tests/sql_rdf_dialect.rs and
//! tests/chicago_validation.rs).

use graphlaw::duckdb_eval::{canonical_ast_graph_bytes, GraphLawDuckDB};
use graphlaw::rules::builtins::{builtin_eval_sql, BuiltinContext};
use graphlaw::sql_ast::{
    compile_turtle, validate_ast_shacl, CompiledQuery, RdfAstGraphReader, SelectStatement,
    SqlExpr, TableRef,
};

fn main() -> std::process::ExitCode {
    let mut fails: Vec<String> = Vec::new();
    let mut check = |label: &str, got: String, ok: bool| {
        println!("=== {label} ===\n{got}\n[exit-class {}]\n", if ok { "MATCH" } else { "MISS" });
        if !ok {
            fails.push(label.to_string());
        }
    };

    // REFUSAL-01 (MalformedAst family): missing sql:statementIndex.
    {
        let doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) .
"#;
        let got = compile_turtle(doc).unwrap_err().to_string();
        let ok = got.contains("REFUSAL-01") && got.contains("statementIndex");
        check("REFUSAL-01 missing statementIndex", got, ok);
    }

    // REFUSAL-01: xsd:integer literal with non-integer lexical form
    // (LITERAL_NOT_INT, RefusalCode = 36).
    {
        let doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) ;
    sql:whereClause [
        sql:operator sql:GreaterThan ;
        sql:leftOperand [ sql:columnName "latency_ms" ] ;
        sql:rightOperand [ sql:literalValue "not-a-number"^^xsd:integer ] ] .
"#;
        let got = compile_turtle(doc).unwrap_err().to_string();
        let ok = got.contains("REFUSAL-01") && got.contains("not-a-number");
        check("REFUSAL-01 LITERAL_NOT_INT", got, ok);
    }

    // REFUSAL-01: unknown join-type IRI (JOIN_TYPE_IRI arm, never downgrades).
    {
        let doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "t1" ] ;
    sql:selectList ( [ sql:columnName "a" ] ) ;
    sql:joinClause [
        sql:joinType <http://graphlaw.org/sql#FullJoin> ;
        sql:joinTable [ sql:tableName "t2" ] ;
        sql:joinCondition [
            sql:operator sql:Equals ;
            sql:leftOperand [ sql:columnName "a" ] ;
            sql:rightOperand [ sql:columnName "b" ] ] ] .
"#;
        let got = match compile_turtle(doc) {
            Err(e) => e.to_string(),
            Ok(_) => "<COMPILED — expected refusal>".into(),
        };
        let ok = got.contains("FullJoin");
        check("REFUSAL-01 JOIN_TYPE_IRI (sql:FullJoin)", got, ok);
    }

    // REFUSAL-02: ConnectStatement has no evaluation path.
    {
        let doc = "@prefix sql: <http://graphlaw.org/sql#> .\n\
                   <http://example.org/s> a sql:ConnectStatement ; sql:statementIndex 1 .\n";
        let got = compile_turtle(doc).unwrap_err().to_string();
        let ok = got.contains("REFUSAL-02") && got.contains("compile refusal");
        check("REFUSAL-02 ConnectStatement", got, ok);
    }

    // REFUSAL-03: aggregate inside sql:whereClause.
    {
        let doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) ;
    sql:whereClause [
        sql:operator sql:GreaterThan ;
        sql:leftOperand [ sql:function <http://graphlaw.org/functions#SUM> ;
                          sql:argument [ sql:columnName "latency_ms" ] ] ;
        sql:rightOperand [ sql:literalValue "100"^^xsd:integer ] ] .
"#;
        let got = compile_turtle(doc).unwrap_err().to_string();
        let ok = got.contains("REFUSAL-03");
        check("REFUSAL-03 aggregate in WHERE", got, ok);
    }

    // REFUSAL-03: sql:ProcessScope column in the projection list.
    {
        let doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList (
        [ sql:columnName "latency_ms" ; sql:scope sql:ProcessScope ] ) .
"#;
        let got = compile_turtle(doc).unwrap_err().to_string();
        let ok = got.contains("REFUSAL-03") && got.contains("ProcessScope");
        check("REFUSAL-03 ProcessScope in SELECT", got, ok);
    }

    // REFUSAL-04: builtin eval with a reader that is not document-backed.
    {
        struct NoDocReader;
        impl RdfAstGraphReader for NoDocReader {
            type Node = String;
            fn get_single_object(&self, _: &String, _: &str) -> Option<String> { None }
            fn get_datatype_literal(&self, _: &String, _: &str) -> Option<(String, Option<String>)> { None }
            fn get_iri(&self, _: &String) -> Option<String> { None }
            fn object_count(&self, _: &String, _: &str) -> usize { 0 }
            fn iri_object_count(&self, _: &String, _: &str) -> usize { 0 }
        }
        let db = GraphLawDuckDB::open_in_memory().expect("db opens");
        let ctx = BuiltinContext { db: &db };
        let got = match builtin_eval_sql::<NoDocReader>(
            &["http://example.org/q", "http://example.org/data", "http://example.org/vocab"],
            &["receipt", "rowsEmitted", "queryHash"],
            &ctx,
            &NoDocReader,
        ) {
            Err(e) => e.to_string(),
            Ok(_) => "<SUCCEEDED — expected refusal>".into(),
        };
        let ok = got.contains("REFUSAL-04");
        check("REFUSAL-04 non-document-backed reader", got, ok);
    }

    // NO-CANONICAL-GRAPH: public SelectStatement::compile() yields a sealed
    // but graph-free CompiledQuery; eval refuses with NoCanonicalGraph.
    {
        let stmt = SelectStatement {
            from: TableRef { name: "t".into(), alias: None, schema: None },
            joins: Vec::new(),
            select_list: vec![SqlExpr::ColumnRef { column: "a".into(), table_alias: None }],
            where_clause: None,
            group_by: Vec::new(),
            having: None,
            order_by: Vec::new(),
            limit: None,
            offset: None,
        };
        let got = match stmt.compile() {
            Ok(q) => {
                let db = GraphLawDuckDB::open_in_memory().expect("db opens");
                match db.eval_ast_query("http://example.org/handbuilt", &q, "s", "v") {
                    Err(e) => e.to_string(),
                    Ok(_) => "<SUCCEEDED — expected refusal>".into(),
                }
            }
            Err(e) => format!("compile side refused: {e}"),
        };
        let ok = got.contains("no canonical graph bytes");
        check("NO-CANONICAL-GRAPH hand-built CompiledQuery", got, ok);
    }

    // ATTEST/RECEIPT admission-seal: mutate a compiled query's pub sql field
    // after compile; the sole-admission seal breaks and eval refuses typed.
    {
        let doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) .
"#;
        let mut q: CompiledQuery = compile_turtle(doc).expect("compiles");
        q.sql = q.sql.replace("batch_id", "tampered");
        let db = GraphLawDuckDB::open_in_memory().expect("db opens");
        let got = match db.eval_ast_query("http://example.org/tampered", &q, "s", "v") {
            Err(e) => e.to_string(),
            Ok(_) => "<SUCCEEDED — expected refusal>".into(),
        };
        let ok = got.contains("admission seal mismatch");
        check("ATTEST-REFUSED-class admission-seal ReceiptError", got, ok);
    }

    // ReceiptError via canonical_ast_graph_bytes on a malformed Turtle doc.
    {
        let got = match canonical_ast_graph_bytes("this is not turtle @@@") {
            Err(e) => e.to_string(),
            Ok(_) => "<SUCCEEDED — expected refusal>".into(),
        };
        let ok = got.contains("Receipt generation error") && got.contains("refused");
        check("ReceiptError canonicalization refused (malformed Turtle)", got, ok);
    }

    // UnsupportedTerm gap: unrecognized function IRI (no REFUSAL marker).
    {
        let doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix fn: <http://graphlaw.org/functions#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList (
        [ sql:function <http://example.org/functions#MEDIAN> ;
          sql:argument [ sql:columnName "latency_ms" ] ] ) .
"#;
        let got = compile_turtle(doc).unwrap_err().to_string();
        let ok = got.contains("MEDIAN") && !got.contains("REFUSAL-0");
        check("UNCLASSIFIED unknown function IRI (no marker)", got, ok);
    }

    // Shape-check that validate_ast_shacl is reachable directly (DX surface).
    {
        let shapes = std::fs::read_to_string("ontologies/sql-ast.ttl").expect("shapes exist");
        let got = match validate_ast_shacl(
            "@prefix sql: <http://graphlaw.org/sql#> .\n<http://x> a sql:SelectStatement .\n",
            &shapes,
        ) {
            Err(e) => e.to_string(),
            Ok(()) => "<VALIDATED — expected violation>".into(),
        };
        let ok = got.contains("SHACL AST structural violation");
        check("direct validate_ast_shacl (missing index, no context fix-up)", got, ok);
    }

    if fails.is_empty() {
        println!("ALL REFUSAL PROBES MATCHED");
        std::process::ExitCode::SUCCESS
    } else {
        eprintln!("MISSED: {fails:?}");
        std::process::ExitCode::FAILURE
    }
}
