//! SQL-in-RDF is a dialect: sniff routes it, a shape violation never reaches
//! DuckDB, an admitted AST executes on frozen-duckdb, and the section 4/5
//! typed refusal taxonomy (REFUSAL-01/02/03) refuses before any engine call.

use graphlaw::dialect::{Dialect, RefusalKind, sniff};
use graphlaw::sql_ast::compile_turtle;
use graphlaw::tier::{Admitted, admit_bytes_tiered};

const SELECT: &str = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:  <http://example.org/queries#> .

ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) ;
    sql:whereClause [
        sql:operator sql:GreaterThan ;
        sql:leftOperand [ sql:columnName "latency_ms" ] ;
        sql:rightOperand [ sql:literalValue "400"^^xsd:integer ]
    ] .
"#;

#[test]
fn sniff_routes_sql_select_and_leaves_plain_turtle() {
    assert_eq!(sniff(SELECT.as_bytes(), None).unwrap(), Dialect::SqlRdf);
    assert_eq!(
        Dialect::SqlRdf.engine(),
        graphlaw::dialect::Engine::FrozenDuckdb
    );
    let turtle = b"@prefix ex: <http://example.org/> .\nex:a ex:b ex:c .\n";
    assert_eq!(sniff(turtle, None).unwrap(), Dialect::Turtle);
}

#[test]
fn sniff_sql_term_without_select_is_ambiguous() {
    let doc = b"@prefix sql: <http://graphlaw.org/sql#> .\n<http://example.org/x> sql:tableName \"events\" .\n";
    let err = sniff(doc, None).unwrap_err();
    assert_eq!(err.kind, RefusalKind::Ambiguous);
    assert!(err.message.contains("sql"));
    assert_eq!(sniff(doc, Some("sql")).unwrap(), Dialect::SqlRdf);
}

#[test]
fn shape_violation_refuses_before_duckdb() {
    let doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "events; drop table x" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) .
"#;
    let err = compile_turtle(doc).unwrap_err();
    let message = err.to_string();
    assert!(
        message.contains("SHACL") || message.contains("violation") || message.contains("Invalid"),
        "{message}"
    );
    let admitted = admit_bytes_tiered(doc.as_bytes(), None, None, None);
    assert!(
        admitted.is_err(),
        "a shape violation must not admit a query"
    );
}

#[test]
fn missing_statement_index_is_refusal_01_before_duckdb() {
    // The section 4 StatementBaseShape requires exactly one integer
    // sql:statementIndex; its absence is REFUSAL-01 at compile time.
    let doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) .
"#;
    let err = compile_turtle(doc).unwrap_err().to_string();
    assert!(err.contains("REFUSAL-01"), "{err}");
    assert!(err.contains("statementIndex"), "{err}");
    let admitted = admit_bytes_tiered(doc.as_bytes(), None, None, None);
    assert!(admitted.is_err());
}

#[test]
fn disallowed_operator_token_is_refusal_01() {
    // Section 4 operator enum only (= != < <= > >= AND OR + - * / % LIKE ILIKE).
    let doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) ;
    sql:whereClause [
        sql:operator "SOUNDS-LIKE"^^<http://www.w3.org/2001/XMLSchema#string> ;
        sql:leftOperand [ sql:columnName "batch_id" ] ;
        sql:rightOperand [ sql:literalValue "alpha" ]
    ] .
"#;
    let err = compile_turtle(doc).unwrap_err().to_string();
    assert!(err.contains("REFUSAL-01"), "{err}");
    assert!(err.contains("SOUNDS-LIKE"), "{err}");
}

#[test]
fn section4_string_operator_tokens_compile() {
    // The section 4 string token "=" must compile exactly like the axiom IRI.
    let doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) ;
    sql:whereClause [
        sql:operator "="^^<http://www.w3.org/2001/XMLSchema#string> ;
        sql:leftOperand [ sql:columnName "batch_id" ] ;
        sql:rightOperand [ sql:literalValue "alpha" ]
    ] .
"#;
    let compiled = compile_turtle(doc).unwrap();
    assert!(compiled.sql.contains("= $1"), "{}", compiled.sql);
    assert_eq!(compiled.params.len(), 1);
}

mod duckdb_courts {
    use super::*;
    use graphlaw::duckdb_eval::GraphLawDuckDB;

    fn db() -> GraphLawDuckDB {
        let db = GraphLawDuckDB::open_in_memory().unwrap();
        db.raw_connection()
            .execute_batch(
                "CREATE TABLE telemetry_events (batch_id VARCHAR, latency_ms INTEGER);
                 INSERT INTO telemetry_events VALUES ('alpha', 480), ('beta', 10), ('gamma', 900);",
            )
            .unwrap();
        db
    }

    /// The compiled query's own output values for one column, read back from
    /// the execution receipt's row projection (real frozen-duckdb rows, not a
    /// hand-written twin of the query). Mirrors the M07 helper in
    /// tests/chicago_validation.rs.
    fn column_values(
        receipt: &graphlaw::duckdb_eval::ExecutionReceipt,
        column: &str,
    ) -> Vec<String> {
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

    /// Strengthened (lane N7): the admitted SELECT (latency_ms > 400) emits
    /// EXACTLY rows_emitted == 2 and EXACTLY batch_id values [alpha, gamma].
    /// A dropped-filter mutation (3 rows incl. beta) or an inverted-filter
    /// mutation (only beta) now fails loudly — no `>= 1` floor.
    #[test]
    fn admitted_ast_executes_on_frozen_duckdb() {
        let admitted = admit_bytes_tiered(SELECT.as_bytes(), None, None, None).unwrap();
        let compiled = match admitted {
            Admitted::Query(q) => q,
            other => panic!("expected a compiled query, got {other:?}"),
        };
        assert!(
            compiled.sql.to_ascii_uppercase().contains("SELECT"),
            "{}",
            compiled.sql
        );

        let db = db();
        let receipt = db
            .eval_ast_query(
                "http://example.org/queries#Q",
                &compiled,
                "http://example.org/data",
                "http://example.org/vocab",
            )
            .unwrap();
        assert_ne!(receipt.query_hash, [0u8; 32]);
        assert_eq!(receipt.rows_emitted, 2, "exact rowcount, not a >= 1 floor");
        let ids = column_values(&receipt, "batch_id");
        assert_eq!(ids, vec!["alpha", "gamma"]);
        assert!(
            !ids.contains(&"beta".to_string()),
            "inverted/dropped filter would re-admit beta"
        );
    }

    /// SQLRDF-03 acceptance: grouping + HAVING + ORDER BY compiles from RDF
    /// and executes on a real frozen-duckdb connection. Strengthened (lane
    /// N7): the COMPILED query is executed as-is via eval_ast_query and its
    /// OWN output rows are read back from the receipt — no twin SQL.
    #[test]
    fn group_having_order_by_select_executes_with_table_readback() {
        let doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList (
        [ sql:columnName "batch_id" ]
        [ sql:aliasName "total" ; sql:expression [
            sql:function <http://graphlaw.org/functions#SUM> ;
            sql:argument [ sql:columnName "latency_ms" ] ] ]
    ) ;
    sql:groupBy ( [ sql:columnName "batch_id" ] ) ;
    sql:having [
        sql:operator sql:GreaterThanOrEq ;
        sql:leftOperand [
            sql:function <http://graphlaw.org/functions#SUM> ;
            sql:argument [ sql:columnName "latency_ms" ] ] ;
        sql:rightOperand [ sql:literalValue "100"^^xsd:integer ]
    ] ;
    sql:orderBy ( [ sql:columnName "batch_id" ; sql:direction "DESC" ] ) .
"#;
        let compiled = compile_turtle(doc).unwrap();
        let sql_upper = compiled.sql.to_ascii_uppercase();
        for token in ["GROUP BY", "HAVING", "ORDER BY", "DESC"] {
            assert!(
                sql_upper.contains(token),
                "missing {token}: {}",
                compiled.sql
            );
        }
        assert_eq!(compiled.params.len(), 1);
        assert_eq!(
            compiled.params[0],
            graphlaw::sql_ast::SqlLiteral::Integer(100)
        );

        let db = db();
        let receipt = db
            .eval_ast_query(
                "http://example.org/queries#Q",
                &compiled,
                "http://example.org/data",
                "http://example.org/vocab",
            )
            .unwrap();

        // Exact rowcount: beta (10) fails HAVING; a HAVING-dropped mutation
        // would emit 3 rows, a total-inversion mutation 1 row.
        assert_eq!(receipt.rows_emitted, 2, "HAVING must filter beta out");
        // The compiled query's own row values (column readback, sorted).
        let ids = column_values(&receipt, "batch_id");
        assert_eq!(ids, vec!["alpha", "gamma"]);
        assert_eq!(column_values(&receipt, "total"), vec!["480", "900"]);
        assert!(
            !ids.contains(&"beta".to_string()),
            "a HAVING that stops filtering would re-admit beta"
        );
    }

    /// SQLRDF-07: an aggregate inside sql:whereClause is REFUSAL-03, before
    /// any DuckDB call.
    #[test]
    fn aggregate_in_where_is_refusal_03() {
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
        sql:leftOperand [
            sql:function <http://graphlaw.org/functions#SUM> ;
            sql:argument [ sql:columnName "latency_ms" ] ] ;
        sql:rightOperand [ sql:literalValue "100"^^xsd:integer ]
    ] .
"#;
        let err = compile_turtle(doc).unwrap_err().to_string();
        assert!(err.contains("REFUSAL-03"), "{err}");
        // The database is untouched: refusal happened at compile time.
        let db = db();
        let rows: i64 = db
            .raw_connection()
            .query_row("SELECT count(*)::BIGINT FROM telemetry_events", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(rows, 3);
    }

    /// SQLRDF-07 evasion shape: aggregate hidden under an sql:aliasName +
    /// sql:expression indirection inside WHERE.
    #[test]
    fn aggregate_hidden_under_alias_indirection_in_where_is_refusal_03() {
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
        sql:leftOperand [
            sql:aliasName "hidden" ;
            sql:expression [
                sql:function <http://graphlaw.org/functions#COUNT> ;
                sql:argument [ sql:columnName "batch_id" ] ] ] ;
        sql:rightOperand [ sql:literalValue "0"^^xsd:integer ]
    ] .
"#;
        let err = compile_turtle(doc).unwrap_err().to_string();
        assert!(err.contains("REFUSAL-03"), "{err}");
    }

    /// SQLRDF-07 evasion shape: COUNT(SUM(x)) nested aggregate in the
    /// projection list.
    #[test]
    fn nested_aggregate_is_refusal_03() {
        let doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList (
        [ sql:columnName "batch_id" ]
        [ sql:function <http://graphlaw.org/functions#COUNT> ;
          sql:argument [
            sql:function <http://graphlaw.org/functions#SUM> ;
            sql:argument [ sql:columnName "latency_ms" ] ] ]
    ) .
"#;
        let err = compile_turtle(doc).unwrap_err().to_string();
        assert!(err.contains("REFUSAL-03"), "{err}");
    }

    /// SQLRDF-07: an explicit sql:ProcessScope column in the projection list
    /// is REFUSAL-03; a sql:ViewScope column still executes.
    #[test]
    fn process_scope_column_in_select_list_is_refusal_03() {
        let doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList (
        [ sql:columnName "batch_id" ; sql:scope sql:ViewScope ]
        [ sql:columnName "latency_ms" ; sql:scope sql:ProcessScope ]
    ) .
"#;
        let err = compile_turtle(doc).unwrap_err().to_string();
        assert!(err.contains("REFUSAL-03"), "{err}");

        // The ViewScope twin compiles and runs.
        let ok = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ; sql:scope sql:ViewScope ] ) .
"#;
        let compiled = compile_turtle(ok).unwrap();
        let db = db();
        let executed = compiled.sql.replace("$1", "0");
        let mut stmt = db.raw_connection().prepare(&executed).unwrap();
        let n: Vec<String> = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(n.len(), 3);
    }

    /// SQLRDF-07 evasion shape: an aggregate in a GROUP BY item is
    /// REFUSAL-03 (the join-condition shape is covered by
    /// `join_condition_aggregate_is_refusal_03` below).
    #[test]
    fn aggregate_in_group_by_is_refusal_03() {
        let doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) ;
    sql:groupBy ( [ sql:function <http://graphlaw.org/functions#COUNT> ;
                    sql:argument [ sql:columnName "batch_id" ] ] ) .
"#;
        let err = compile_turtle(doc).unwrap_err().to_string();
        assert!(err.contains("REFUSAL-03"), "{err}");
    }

    /// SQLRDF-07 evasion shape: a property-stuffing node carrying both
    /// sql:columnName and sql:function must not silently extract as a
    /// ColumnRef (the aggregate would vanish) — REFUSAL-01.
    #[test]
    fn property_stuffing_node_is_refusal_01() {
        let doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ;
                       sql:function <http://graphlaw.org/functions#SUM> ;
                       sql:argument [ sql:columnName "latency_ms" ] ] ) .
"#;
        let err = compile_turtle(doc).unwrap_err().to_string();
        assert!(err.contains("REFUSAL-01"), "{err}");
    }

    /// SQLRDF-06: Connect/Disconnect are REFUSAL-02 typed engine refusals.
    #[test]
    fn connect_disconnect_carry_refusal_02() {
        for kind in ["ConnectStatement", "DisconnectStatement"] {
            let doc = format!(
                r#"@prefix sql: <http://graphlaw.org/sql#> .
<http://example.org/s> a sql:{kind} ; sql:statementIndex 1 ."#
            );
            let err = compile_turtle(&doc).unwrap_err().to_string();
            assert!(err.contains("REFUSAL-02"), "{kind}: {err}");
            assert!(err.contains("compile refusal"), "{kind}: {err}");
        }
    }

    /// SQLRDF-04: the section 7 analytical MERGE graph compiles to the
    /// section 7.1 parameterized statement with $1="SETTLED" extracted into
    /// the parameter buffer, and executes on real tables with readback.
    /// (The USING sub-select uses the SQLRDF-01 alias property names, which
    /// the current alias-window shapes require; clause semantics are the
    /// section 7 graph's.)
    #[test]
    fn section7_merge_compiles_to_parameterized_sql_with_params() {
        let doc = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix sql:  <http://graphlaw.org/sql#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .

_:mergeStatement a sql:MergeIntoStatement ;
    sql:statementIndex 1 ;
    sql:targetTable [
        sql:schemaName "main" ;
        sql:tableName "regional_sales"
    ] ;
    sql:usingSource [
        a sql:SelectStatement ;
        sql:statementIndex 1 ;
        sql:selectList (
            [ sql:columnName "region_id" ; sql:scope sql:ViewScope ]
            [ sql:aliasName "total_invoiced" ;
              sql:expression [
                sql:function <http://graphlaw.org/functions#SUM> ;
                sql:argument [ sql:columnName "amount" ; sql:scope sql:ProcessScope ] ] ;
              sql:scope sql:ViewScope ]
        ) ;
        sql:fromTable [
            sql:schemaName "staging" ;
            sql:tableName "invoices" ] ;
        sql:where [
            sql:operator "="^^<http://www.w3.org/2001/XMLSchema#string> ;
            sql:left [ sql:columnName "status" ; sql:scope sql:ProcessScope ] ;
            sql:right [ sql:literalValue "SETTLED" ; sql:datatype xsd:string ]
        ] ;
        sql:groupBy (
            [ sql:columnName "region_id" ; sql:scope sql:ProcessScope ]
        )
    ] ;
    sql:mergeOn [
        sql:operator "="^^<http://www.w3.org/2001/XMLSchema#string> ;
        sql:left [ sql:qualifier "main.regional_sales" ; sql:columnName "region_id" ; sql:scope sql:ProcessScope ] ;
        sql:right [ sql:qualifier "src" ; sql:columnName "region_id" ; sql:scope sql:ProcessScope ]
    ] ;
    sql:whenMatched [
        sql:setAssignments (
            [ sql:targetColumn "revenue" ;
              sql:valueExpression [
                sql:operator "+"^^<http://www.w3.org/2001/XMLSchema#string> ;
                sql:left [ sql:qualifier "main.regional_sales" ; sql:columnName "revenue" ; sql:scope sql:ProcessScope ] ;
                sql:right [ sql:qualifier "src" ; sql:columnName "total_invoiced" ; sql:scope sql:ProcessScope ] ] ]
        )
    ] .
"#;
        let compiled = compile_turtle(doc).unwrap();
        let norm = |s: &str| {
            s.chars()
                .map(|c| if c == '(' || c == ')' { ' ' } else { c })
                .collect::<String>()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .to_ascii_lowercase()
        };
        let expected = norm(
            "MERGE INTO main.regional_sales USING (SELECT region_id, SUM(amount) AS total_invoiced \
             FROM staging.invoices WHERE status = $1 GROUP BY region_id) AS src \
             ON main.regional_sales.region_id = src.region_id \
             WHEN MATCHED THEN UPDATE SET revenue = (main.regional_sales.revenue + src.total_invoiced);",
        );
        assert_eq!(
            norm(&compiled.sql),
            expected,
            "\nactual: {}\n",
            compiled.sql
        );
        assert_eq!(
            compiled.params,
            vec![graphlaw::sql_ast::SqlLiteral::String("SETTLED".into())]
        );

        // Real execution against real staging.invoices and main.regional_sales.
        let db = GraphLawDuckDB::open_in_memory().unwrap();
        db.raw_connection()
            .execute_batch(
                "CREATE SCHEMA staging;
                 CREATE TABLE staging.invoices (region_id VARCHAR, amount BIGINT, status VARCHAR);
                 INSERT INTO staging.invoices VALUES ('east', 100, 'SETTLED'), ('east', 50, 'OPEN'), ('west', 70, 'SETTLED');
                 CREATE TABLE main.regional_sales (region_id VARCHAR, revenue BIGINT);
                 INSERT INTO main.regional_sales VALUES ('east', 1000), ('west', 2000);",
            )
            .unwrap();
        let executed = compiled.sql.replace("$1", "'SETTLED'");
        db.raw_connection().execute_batch(&executed).unwrap();
        let revenue = |region: &str| -> i64 {
            db.raw_connection()
                .query_row(
                    "SELECT revenue FROM main.regional_sales WHERE region_id = ?",
                    [region],
                    |r| r.get::<_, i64>(0),
                )
                .unwrap()
        };
        // Only SETTLED invoices merge: east 1000+100=1100, west 2000+70=2070.
        assert_eq!(revenue("east"), 1100);
        assert_eq!(revenue("west"), 2070);

        // Receipt determinism (section 1.4, first formula): same graph and
        // same buffer hash identically; changing the literal changes the
        // digest. Hashing the SQL text alone does not bind the graph.
        use graphlaw::duckdb_eval::{admitted_statement_hash, canonical_ast_graph_bytes};
        let canon = canonical_ast_graph_bytes(doc).unwrap();
        let h1 = admitted_statement_hash(&canon, &compiled.params);
        let h2 = admitted_statement_hash(&canon, &compiled.params);
        assert_eq!(h1, h2);
        let mut changed = compiled.params.clone();
        changed[0] = graphlaw::sql_ast::SqlLiteral::String("OPEN".into());
        assert_ne!(admitted_statement_hash(&canon, &changed), h1);
    }

    /// SQLRDF-03 coverage hole closed: a schema-qualified sql:from rdf:List
    /// (rec section 3.3) compiles with the `schema.table` qualifier, executes
    /// on a real frozen-duckdb connection, and its scope-IRI projection is
    /// verified by readback of the materialized RDF statements.
    #[test]
    fn schema_qualified_from_list_compiles_and_executes() {
        let doc = r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex:  <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:from (
        [ sql:schemaName "main" ; sql:tableName "telemetry_events" ]
    ) ;
    sql:selectList (
        [ sql:columnName "batch_id" ; sql:scope sql:ViewScope ]
    ) .
"#;
        let compiled = compile_turtle(doc).unwrap();
        // The schema qualifier must survive compilation into the FROM clause.
        assert!(
            compiled
                .sql
                .to_ascii_uppercase()
                .contains("FROM MAIN.TELEMETRY_EVENTS"),
            "{}",
            compiled.sql
        );

        // Real table on the same connection. The sql-rdf compiler exposes no
        // CREATE statement kind (compile_command: Insert/Update/Delete only),
        // so the fixture is a real DDL batch on the raw frozen-duckdb
        // connection — the same pattern every other court in this file uses.
        let db = GraphLawDuckDB::open_in_memory().unwrap();
        db.raw_connection()
            .execute_batch(
                "CREATE TABLE main.telemetry_events (batch_id VARCHAR, latency_ms INTEGER);
                 INSERT INTO main.telemetry_events VALUES ('alpha', 480), ('beta', 10), ('gamma', 900);
                 CREATE TABLE main.shipments_events (batch_id VARCHAR, latency_ms INTEGER);
                 INSERT INTO main.shipments_events VALUES ('alpha', 480), ('beta', 10), ('gamma', 900);",
            )
            .unwrap();

        // Real execution through the typed eval path with scope IRIs.
        let subject_scope = "http://example.org/results";
        let predicate_scope = "http://example.org/vocab";
        let receipt = db
            .eval_ast_query(
                "http://example.org/queries#Q",
                &compiled,
                subject_scope,
                predicate_scope,
            )
            .unwrap();
        assert_eq!(
            receipt.rows_emitted, 3,
            "schema-qualified source must resolve all rows"
        );
        assert_ne!(receipt.query_hash, [0u8; 32]);
        // The receipt binds the canonical AST graph plus the (empty)
        // parameter buffer, not the SQL text alone.
        let canon = graphlaw::duckdb_eval::canonical_ast_graph_bytes(doc).unwrap();
        assert_eq!(
            receipt.query_hash,
            graphlaw::duckdb_eval::admitted_statement_hash(&canon, &compiled.params)
        );

        // Readback of the materialized RDF statements: the row subjects carry
        // the subject scope IRI and the ViewScope column is projected under
        // its plain SQL name as a predicate IRI under the predicate scope.
        assert!(!receipt.result_triples.is_empty());
        for triple in &receipt.result_triples {
            assert!(
                triple
                    .subject
                    .starts_with("http://example.org/results/row_"),
                "unexpected subject {}",
                triple.subject
            );
            assert_eq!(
                triple.predicate, "http://example.org/vocab/batch_id",
                "ViewScope column must project under its SQL column name"
            );
        }
        // Observed scope-storage behavior (documented): sql:scope is an
        // admission-time annotation only. It does not appear in the compiled
        // SQL and is not stored on the projected terms; the scope IRI lives
        // solely in the caller-supplied subject/predicate prefixes.
        assert!(!compiled.sql.contains("scope"));

        let values: Vec<String> = receipt
            .result_triples
            .iter()
            .map(|t| match &t.object {
                purrdf::TermValue::Literal { lexical_form, .. } => lexical_form.clone(),
                other => panic!("expected a literal, got {other:?}"),
            })
            .collect();
        assert_eq!(values.len(), 3);
        assert!(values.contains(&"alpha".to_string()));
        assert!(values.contains(&"beta".to_string()));
        assert!(values.contains(&"gamma".to_string()));

        // Anti-vacuity: the identical query shape aimed at a nonexistent
        // schema must produce a typed refusal, not a panic and not a raw
        // engine string surfaced to the caller.
        let ghost = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex:  <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:from (
        [ sql:schemaName "ghost_schema" ; sql:tableName "telemetry_events" ]
    ) ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) .
"#;
        let ghost_compiled = compile_turtle(ghost).unwrap();
        assert!(
            ghost_compiled
                .sql
                .to_ascii_uppercase()
                .contains("FROM GHOST_SCHEMA.TELEMETRY_EVENTS")
        );
        let err = db
            .eval_ast_query(
                "http://example.org/queries#Q",
                &ghost_compiled,
                subject_scope,
                predicate_scope,
            )
            .unwrap_err();
        assert!(
            matches!(err, graphlaw::duckdb_eval::EvaluatorError::DuckDb(_)),
            "expected a typed EvaluatorError, got {err:?}"
        );
        // The typed wrapper reports it as an engine failure through the
        // stable Display path; the table in the real schema is untouched.
        assert!(err.to_string().contains("DuckDB execution failure"));
        let rows: i64 = db
            .raw_connection()
            .query_row(
                "SELECT count(*)::BIGINT FROM main.telemetry_events",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(rows, 3);
    }
}

/// SQLRDF-07 evasion-shape court, join-condition lane: an aggregate inside
/// a sql:joinClause's sql:joinCondition must be REFUSAL-03 at compile time,
/// before any DuckDB call, leaving the backing tables untouched.
mod duckdb_join_condition_courts {
    use super::*;
    use graphlaw::duckdb_eval::GraphLawDuckDB;

    fn db() -> GraphLawDuckDB {
        let db = GraphLawDuckDB::open_in_memory().unwrap();
        db.raw_connection()
            .execute_batch(
                "CREATE TABLE telemetry_events (batch_id VARCHAR, latency_ms INTEGER);
                 INSERT INTO telemetry_events VALUES ('alpha', 480), ('beta', 10), ('gamma', 900);
                 CREATE TABLE batch_names (batch_id VARCHAR, label VARCHAR);
                 INSERT INTO batch_names VALUES ('alpha', 'Alpha'), ('gamma', 'Gamma');",
            )
            .unwrap();
        db
    }

    #[test]
    fn join_condition_aggregate_is_refusal_03() {
        let doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) ;
    sql:joinClause [
        sql:joinType sql:InnerJoin ;
        sql:joinTable [ sql:tableName "batch_names" ] ;
        sql:joinCondition [
            sql:operator sql:GreaterThan ;
            sql:leftOperand [
                sql:function <http://graphlaw.org/functions#SUM> ;
                sql:argument [ sql:columnName "latency_ms" ] ] ;
            sql:rightOperand [ sql:literalValue "0"^^xsd:integer ]
        ]
    ] .
"#;
        let err = compile_turtle(doc).unwrap_err().to_string();
        assert!(err.contains("REFUSAL-03"), "{err}");

        // The refusal happened at compile time: the real tables are unchanged.
        let db = db();
        let (events, names): (i64, i64) = db
            .raw_connection()
            .query_row(
                "SELECT (SELECT count(*)::BIGINT FROM telemetry_events), \
                        (SELECT count(*)::BIGINT FROM batch_names)",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(events, 3);
        assert_eq!(names, 2);
    }
}

/// SQLRDF-R1 determinism court (H_query, sql-rdf Rec. section 1.4 first
/// formula): the same admitted AST graph in five cosmetic variants must hash
/// identically through the PUBLIC compile+eval path, while a parameter change
/// or a semantic literal change must move the digest. The receipt must also
/// equal the hash recomputed directly from `canonical_ast_graph_bytes` +
/// params, i.e. it is independent of the pre-/post-R1 formula plumbing.
#[cfg(test)]
mod query_hash_determinism_court {
    use graphlaw::duckdb_eval::{
        GraphLawDuckDB, admitted_statement_hash, canonical_ast_graph_bytes,
    };
    use graphlaw::sql_ast::{SqlLiteral, compile_turtle};

    fn base_doc() -> String {
        r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:  <http://example.org/queries#> .

ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) ;
    sql:whereClause [
        sql:operator sql:GreaterThan ;
        sql:leftOperand [ sql:columnName "latency_ms" ] ;
        sql:rightOperand [ sql:literalValue "400"^^xsd:integer ]
    ] .
"#
        .to_string()
    }

    /// Cosmetic variant 1: prefix rename (`sql:` -> `q:`, `ex:` -> `qa:`).
    fn prefix_renamed() -> String {
        r#"
@prefix q:  <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix qa: <http://example.org/queries#> .

qa:Q a q:SelectStatement ;
    q:statementIndex 1 ;
    q:fromTable [ q:tableName "telemetry_events" ] ;
    q:selectList ( [ q:columnName "batch_id" ] ) ;
    q:whereClause [
        q:operator q:GreaterThan ;
        q:leftOperand [ q:columnName "latency_ms" ] ;
        q:rightOperand [ q:literalValue "400"^^xsd:integer ]
    ] .
"#
        .to_string()
    }

    /// Cosmetic variant 2: explicit blank-node labels (`_:f1`..) instead of
    /// anonymous `[...]` — graph-isomorphic, labels must not leak into Canon.
    fn blank_node_labels() -> String {
        r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:  <http://example.org/queries#> .

ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable _:f1 ;
    sql:selectList ( _:f2 ) ;
    sql:whereClause _:f3 .
_:f1 sql:tableName "telemetry_events" .
_:f2 sql:columnName "batch_id" .
_:f3 sql:operator sql:GreaterThan ;
    sql:leftOperand _:f4 ;
    sql:rightOperand _:f5 .
_:f4 sql:columnName "latency_ms" .
_:f5 sql:literalValue "400"^^xsd:integer .
"#
        .to_string()
    }

    /// Cosmetic variant 3: statement reorder (triples permuted).
    fn reordered() -> String {
        r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:  <http://example.org/queries#> .

ex:Q sql:whereClause [
        sql:operator sql:GreaterThan ;
        sql:leftOperand [ sql:columnName "latency_ms" ] ;
        sql:rightOperand [ sql:literalValue "400"^^xsd:integer ]
    ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:statementIndex 1 ;
    a sql:SelectStatement .
"#
        .to_string()
    }

    /// Cosmetic variant 4: whitespace/comment noise.
    fn whitespace_noisy() -> String {
        r#"
@prefix sql: <http://graphlaw.org/sql#> .



@prefix xsd:   <http://www.w3.org/2001/XMLSchema#> .
@prefix   ex:  <http://example.org/queries#> .

# a comment the parser must ignore
ex:Q   a   sql:SelectStatement ;
    sql:statementIndex   1 ;
    sql:fromTable   [   sql:tableName   "telemetry_events"   ]   ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) ;
    sql:whereClause [
        sql:operator   sql:GreaterThan ;
        sql:leftOperand   [ sql:columnName "latency_ms" ] ;
        sql:rightOperand   [ sql:literalValue "400"^^xsd:integer ]
    ] .
"#
        .to_string()
    }

    /// Cosmetic variant 5: quoting style (`'400'` single-quoted string form
    /// of the same xsd:integer-typed literal, long-form IRI for the type).
    fn quoting_variant() -> String {
        r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex:  <http://example.org/queries#> .

ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) ;
    sql:whereClause [
        sql:operator sql:GreaterThan ;
        sql:leftOperand [ sql:columnName "latency_ms" ] ;
        sql:rightOperand [ sql:literalValue '400'^^<http://www.w3.org/2001/XMLSchema#integer> ]
    ] .
"#
        .to_string()
    }

    /// Seeds the real table the query reads; no mocks, real frozen-duckdb.
    fn seeded_db() -> GraphLawDuckDB {
        let db = GraphLawDuckDB::open_in_memory().unwrap();
        db.raw_connection()
            .execute_batch(
                "CREATE TABLE main.telemetry_events (batch_id VARCHAR, latency_ms INTEGER);
                 INSERT INTO main.telemetry_events VALUES ('alpha', 480), ('beta', 10), ('gamma', 900);
                 CREATE TABLE main.shipments_events (batch_id VARCHAR, latency_ms INTEGER);
                 INSERT INTO main.shipments_events VALUES ('alpha', 480), ('beta', 10), ('gamma', 900);",
            )
            .unwrap();
        db
    }

    fn eval_hash(doc: &str) -> [u8; 32] {
        let compiled = compile_turtle(doc).unwrap();
        let receipt = seeded_db()
            .eval_ast_query(
                "http://example.org/queries#Q",
                &compiled,
                "urn:r:",
                "urn:p:",
            )
            .unwrap();
        receipt.query_hash
    }

    #[test]
    fn five_cosmetic_variants_hash_identically_through_public_path() {
        let baseline = eval_hash(&base_doc());
        for (name, doc) in [
            ("prefix rename", prefix_renamed()),
            ("blank-node labels", blank_node_labels()),
            ("statement reorder", reordered()),
            ("whitespace/comments", whitespace_noisy()),
            ("quoting style", quoting_variant()),
        ] {
            assert_eq!(
                eval_hash(&doc),
                baseline,
                "cosmetic variant '{name}' must not move H_query"
            );
        }
    }

    #[test]
    fn parameter_change_moves_the_digest() {
        let baseline = eval_hash(&base_doc());
        // Bind a typed parameter by hand on the same canonical graph: the
        // eval path hashes Canon(G_AST) || P_params, so a different buffer
        // must produce a different digest even for identical SQL text.
        let compiled = compile_turtle(&base_doc()).unwrap();
        let _changed = compiled.params.clone();
        // LANE BUMP minimal unblock: eval_ast_query takes `&CompiledQuery`,
        // so mutate the params ON the compiled query rather than passing a
        // bare `Vec<SqlLiteral>`.
        let mut changed = compiled;
        let p = changed
            .params
            .iter_mut()
            .find(|p| matches!(p, SqlLiteral::Integer(_)))
            .expect("the threshold literal is a bound parameter");
        *p = SqlLiteral::Integer(401);
        let receipt = seeded_db()
            .eval_ast_query("http://example.org/queries#Q", &changed, "urn:r:", "urn:p:")
            .unwrap();
        assert_ne!(receipt.query_hash, baseline);
    }

    #[test]
    fn semantic_literal_change_moves_the_digest() {
        let baseline = eval_hash(&base_doc());
        // Semantic (not cosmetic) edit: the table name literal differs, so
        // Canon(G_AST) differs. Params are unchanged — proving the graph
        // alone binds the receipt.
        let doc = base_doc().replace("telemetry_events", "shipments_events");
        let compiled = compile_turtle(&doc).unwrap();
        let receipt = seeded_db()
            .eval_ast_query(
                "http://example.org/queries#Q",
                &compiled,
                "urn:r:",
                "urn:p:",
            )
            .unwrap();
        assert_ne!(receipt.query_hash, baseline);
    }

    #[test]
    fn receipt_equals_recomputed_canon_plus_params_hash() {
        for doc in [
            base_doc(),
            prefix_renamed(),
            blank_node_labels(),
            reordered(),
        ] {
            let compiled = compile_turtle(&doc).unwrap();
            let receipt = seeded_db()
                .eval_ast_query(
                    "http://example.org/queries#Q",
                    &compiled,
                    "urn:r:",
                    "urn:p:",
                )
                .unwrap();
            let canon = canonical_ast_graph_bytes(&doc).unwrap();
            assert_eq!(
                receipt.query_hash,
                admitted_statement_hash(&canon, &compiled.params),
                "receipt must equal BLAKE3(Canon(G_AST) || P_params) recomputed from the public API"
            );
        }
    }
}

// ---- Fuzz round 2 courts (D1 / D5 / D4-adjacent) --------------------------

#[test]
fn d1_empty_select_list_is_refusal_01_not_select_star() {
    // rdf:nil projection head unrolls to zero items; the ontology says
    // projection is never zero — REFUSAL-01, never an implicit SELECT *.
    let alias_doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList () .
"#;
    for doc in [
        alias_doc,
        r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:select () .
"#,
    ] {
        let err = compile_turtle(doc).unwrap_err().to_string();
        // Both spellings refuse with the typed code before any engine call:
        // the alias form via the extract-path zero-projection guard, the
        // normative form via the SHACL pre-gate (the xone-level sh:message on
        // sql:SelectStatementShape carries the stable [REFUSAL-01] marker,
        // fuzz D1 — no generic "Node <Q>: violation" surfaces). The typed
        // extract guard stays as defense-in-depth for the zero-item case
        // that reaches the reader.
        assert!(err.contains("REFUSAL-01"), "{err}");
        assert!(admit_bytes_tiered(doc.as_bytes(), None, None, None).is_err());
    }
}

#[test]
fn d5_operator_dual_carry_string_and_iri_is_refusal_01() {
    // A BinaryExpression carrying BOTH a string token and an IRI operator
    // must refuse, not silently prefer the string token.
    let doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) ;
    sql:whereClause [
        sql:operator "="^^<http://www.w3.org/2001/XMLSchema#string> , sql:GreaterThan ;
        sql:leftOperand [ sql:columnName "batch_id" ] ;
        sql:rightOperand [ sql:literalValue "alpha" ]
    ] .
"#;
    let err = compile_turtle(doc).unwrap_err().to_string();
    assert!(err.contains("REFUSAL-01"), "{err}");
    assert!(err.contains("exactly one operator form"), "{err}");
}

#[test]
fn d4_direction_iri_object_is_refusal_01_not_silent_asc() {
    // sql:direction with an IRI object must REFUSAL-01, never fold to ASC.
    let doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) ;
    sql:orderBy ( [ sql:columnName "batch_id" ; sql:direction sql:Descending ] ) .
"#;
    let err = compile_turtle(doc).unwrap_err().to_string();
    assert!(err.contains("REFUSAL-01"), "{err}");
    assert!(err.contains("direction"), "{err}");
    assert!(admit_bytes_tiered(doc.as_bytes(), None, None, None).is_err());
}

// ---- L163 message-verbatim refusal courts ---------------------------------
// Lock the exact user-visible Display text per refusal class, as captured by
// `cargo run --example refusal_probe` (ALL REFUSAL PROBES MATCHED, exit 0).
// A message regression flips these courts. The ident-buffer truncation class
// pins the REFUSAL_IDENT_BYTES (192) truncated prefix verbatim.

/// Exact-message pin: REFUSAL-01 missing sql:statementIndex (fully
/// deterministic — the subject IRI interpolates, no ident truncation).
#[test]
fn l163_msg_missing_statement_index_is_verbatim() {
    let doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) .
"#;
    let err = compile_turtle(doc).unwrap_err();
    assert_eq!(
        err.to_string(),
        "SHACL AST structural violation: [REFUSAL-01] MalformedAstRefusal:statement \
         http://example.org/queries#Q is missing required sql:statementIndex \
         (section 4 StatementBaseShape)"
    );
}

/// Exact-message pin: LITERAL_NOT_INT. The aux ident is a generated
/// blank-node id, so the stable structural prefix and suffix are pinned; the
/// ident-buffer truncation class pins the REFUSAL_IDENT_BYTES prefix
/// verbatim (the buffer renders the first 192 bytes and nothing more).
#[test]
fn l163_msg_literal_not_int_is_verbatim_with_truncated_ident() {
    use graphlaw::sql_ast::REFUSAL_IDENT_BYTES;

    let head = concat!(
        "@prefix sql: <http://graphlaw.org/sql#> .\n",
        "@prefix ex: <http://example.org/queries#> .\n",
        "ex:Q a sql:SelectStatement ;\n",
        "    sql:statementIndex 1 ;\n",
        "    sql:fromTable [ sql:tableName \"telemetry_events\" ] ;\n",
        "    sql:selectList ( [ sql:columnName \"batch_id\" ] ) ;\n",
        "    sql:whereClause [\n",
        "        sql:operator sql:GreaterThan ;\n",
        "        sql:leftOperand [ sql:columnName \"latency_ms\" ] ;\n",
    );
    let tail = " ] .\n";
    let lit = |lexical: &str| {
        format!(
            "{head}        sql:rightOperand [ sql:literalValue \"{lexical}\"^^\
             <http://www.w3.org/2001/XMLSchema#integer> ]{tail}"
        )
    };

    // Structural pin: prefix + offending lexical + suffix (short lexical).
    let err = compile_turtle(&lit("not-a-number")).expect_err("must refuse");
    let msg = err.to_string();
    let prefix = "SHACL AST structural violation: [REFUSAL-01] MalformedAstRefusal:\
                  xsd:integer literal 'not-a-number' on ";
    assert!(msg.starts_with(prefix), "structural prefix: {msg}");
    assert!(
        msg.ends_with(" does not parse as an integer"),
        "structural suffix: {msg}"
    );

    // Truncation pin: a lexical form exceeding REFUSAL_IDENT_BYTES renders as
    // exactly the first REFUSAL_IDENT_BYTES bytes, verbatim.
    let long = "x".repeat(REFUSAL_IDENT_BYTES + 64);
    let msg = compile_turtle(&lit(&long))
        .expect_err("must refuse")
        .to_string();
    let expected_prefix = format!(
        "SHACL AST structural violation: [REFUSAL-01] MalformedAstRefusal:\
         xsd:integer literal '{}' on ",
        "x".repeat(REFUSAL_IDENT_BYTES)
    );
    assert!(
        msg.starts_with(&expected_prefix),
        "truncated 192-byte prefix must render verbatim: {}",
        &msg[..msg.len().min(expected_prefix.len() + 80)]
    );
}

/// Exact-message pin: LITERAL_NOT_FLOAT — stable structural prefix and
/// suffix of the xsd:double parse refusal (RefusalCode = 37).
#[test]
fn l163_msg_literal_not_float_is_verbatim() {
    let doc = format!(
        concat!(
            "@prefix sql: <http://graphlaw.org/sql#> .\n",
            "@prefix ex: <http://example.org/queries#> .\n",
            "ex:Q a sql:SelectStatement ;\n",
            "    sql:statementIndex 1 ;\n",
            "    sql:fromTable [ sql:tableName \"telemetry_events\" ] ;\n",
            "    sql:selectList ( [ sql:columnName \"batch_id\" ] ) ;\n",
            "    sql:whereClause [\n",
            "        sql:operator sql:GreaterThan ;\n",
            "        sql:leftOperand [ sql:columnName \"latency_ms\" ] ;\n",
            "        sql:rightOperand [ sql:literalValue \"{}\"^^\
             <http://www.w3.org/2001/XMLSchema#double> ] ] ."
        ),
        "1.2.3 garbage"
    );
    let err = compile_turtle(&doc).expect_err("non-double lexical must refuse");
    let msg = err.to_string();
    assert!(
        msg.starts_with(
            "SHACL AST structural violation: [REFUSAL-01] MalformedAstRefusal:\
             xsd:double literal '1.2.3 garbage' on "
        ),
        "structural prefix: {msg}"
    );
    assert!(
        msg.ends_with(" does not parse as a float"),
        "structural suffix: {msg}"
    );
}

/// Exact-message pin: JOIN_TYPE_IRI (sql:FullJoin) — fully deterministic.
/// ANTI-VACUITY MUTATION TWIN: the message-mutated join type (sql:InnerJoin)
/// compiles and renders real JOIN SQL, so the court flips exactly on the
/// refusal-message path — a refusal that never fires cannot satisfy this.
#[test]
fn l163_msg_join_type_iri_is_verbatim() {
    let doc_fmt = |join_type: &str| {
        format!(
            concat!(
                "@prefix sql: <http://graphlaw.org/sql#> .\n",
                "@prefix ex: <http://example.org/queries#> .\n",
                "ex:Q a sql:SelectStatement ;\n",
                "    sql:statementIndex 1 ;\n",
                "    sql:fromTable [ sql:tableName \"t1\" ] ;\n",
                "    sql:selectList ( [ sql:columnName \"a\" ] ) ;\n",
                "    sql:joinClause [\n",
                "        sql:joinType <{}> ;\n",
                "        sql:joinTable [ sql:tableName \"t2\" ] ;\n",
                "        sql:joinCondition [\n",
                "            sql:operator sql:Equals ;\n",
                "            sql:leftOperand [ sql:columnName \"a\" ] ;\n",
                "            sql:rightOperand [ sql:columnName \"b\" ] ] ] ."
            ),
            join_type
        )
    };
    let err = compile_turtle(&doc_fmt("http://graphlaw.org/sql#FullJoin"))
        .expect_err("unknown join-type IRI must refuse");
    assert_eq!(
        err.to_string(),
        "SHACL AST structural violation: [REFUSAL-01] MalformedAstRefusal:sql:joinType \
         IRI 'http://graphlaw.org/sql#FullJoin' is not sql:InnerJoin or sql:LeftJoin; \
         an unknown join-type IRI refuses typed, it never silently downgrades to Inner"
    );

    // Mutation twin: sql:InnerJoin compiles to real SQL — the pinned message
    // is unreachable for the mutated input (the court cannot pass vacuously).
    let ok =
        compile_turtle(&doc_fmt("http://graphlaw.org/sql#InnerJoin")).expect("InnerJoin compiles");
    assert!(
        ok.sql.contains(" JOIN "),
        "mutated join type must take the compiled path: {}",
        ok.sql
    );
}

/// Exact-message pin: REFUSAL-02 ConnectStatement — fully deterministic.
#[test]
fn l163_msg_connect_statement_is_verbatim() {
    let doc = "@prefix sql: <http://graphlaw.org/sql#> .\n\
               <http://example.org/s> a sql:ConnectStatement ; sql:statementIndex 1 .\n";
    let err = compile_turtle(doc).unwrap_err();
    assert_eq!(
        err.to_string(),
        "compile refusal: http://graphlaw.org/sql#ConnectStatement \
         [REFUSAL-02] UnimplementedStatementRefusal: not implemented by this DuckDB build"
    );
}

/// Exact-message pin: REFUSAL-03 aggregate-in-WHERE — fully deterministic.
#[test]
fn l163_msg_aggregate_in_where_is_verbatim() {
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
    let err = compile_turtle(doc).unwrap_err();
    assert_eq!(
        err.to_string(),
        "SHACL AST structural violation: [REFUSAL-03] ScopeIntegrityRefusal:aggregate \
         in a WHERE/JOIN/GROUP BY position is refused (section 1.3/5); re-scope the \
         expression"
    );
}

/// Exact-message pin: NO-CANONICAL-GRAPH — a sealed but graph-free
/// CompiledQuery refuses on eval with the exact user-visible message.
#[test]
fn l163_msg_no_canonical_graph_is_verbatim() {
    use graphlaw::duckdb_eval::GraphLawDuckDB;
    use graphlaw::sql_ast::{SelectStatement, SqlExpr, TableRef};

    let stmt = SelectStatement {
        // Minimal shared-compile unblock (compile-freeze SLA, window lane,
        // disclosed): new fields from the WITH/QUALIFY lanes default to empty
        // in this pre-existing court — semantics unchanged.
        ctes: Vec::new(),
        qualify: None,
        from: TableRef {
            name: "t".into(),
            alias: None,
            schema: None,
        },
        joins: Vec::new(),
        select_list: vec![SqlExpr::ColumnRef {
            column: "a".into(),
            table_alias: None,
        }],
        where_clause: None,
        group_by: Vec::new(),
        having: None,
        order_by: Vec::new(),
        limit: None,
        offset: None,
    };
    let q = stmt
        .compile()
        .expect("hand-built statement compiles sealed");
    let db = GraphLawDuckDB::open_in_memory().expect("db opens");
    let err = db
        .eval_ast_query("http://example.org/handbuilt", &q, "s", "v")
        .expect_err("graph-free query must refuse");
    assert_eq!(
        err.to_string(),
        "no canonical graph bytes — hand-built CompiledQuery must use compile_turtle \
         (query http://example.org/handbuilt)"
    );
}

/// Exact-message pin: admission-seal ReceiptError — tampering with the
/// compiled sql field after compile breaks the sole-admission seal.
#[test]
fn l163_msg_admission_seal_mismatch_is_verbatim() {
    use graphlaw::duckdb_eval::GraphLawDuckDB;

    let doc = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) .
"#;
    let mut q: graphlaw::sql_ast::CompiledQuery = compile_turtle(doc).expect("compiles");
    q.sql = q.sql.replace("batch_id", "tampered");
    let db = GraphLawDuckDB::open_in_memory().expect("db opens");
    let err = db
        .eval_ast_query("http://example.org/tampered", &q, "s", "v")
        .expect_err("seal must break on post-compile mutation");
    assert_eq!(
        err.to_string(),
        "Receipt generation error: admission seal mismatch on query \
         http://example.org/tampered: CompiledQuery is not in its admitted compile \
         state; re-compile via compile_turtle (sole-admission path)"
    );
}

// ---- CTQ-4 / REFUSAL-05 unbounded-mutation courts ---------------------------
// A WHERE-less UPDATE/DELETE would render SQL that mutates every row of the
// target table; it refuses at compile time with the stable [REFUSAL-05]
// marker (UnboundedMutationRefusal, refusal arm 38) before any engine call.
// Each refusal court carries an anti-vacuity twin: the same statement WITH
// sql:whereColumn + sql:whereValue compiles to real bounded SQL and executes
// on a real frozen-duckdb table — the court flips exactly on the refusal path.

fn unbounded_mutation_doc(kind: &str, with_where: bool) -> String {
    let where_clause = if with_where {
        " ; sql:whereColumn \"id\" ; sql:whereValue \"1\"^^<http://www.w3.org/2001/XMLSchema#integer>"
    } else {
        ""
    };
    // UPDATE carries an assignment; DELETE is predicate-only (extra assign
    // properties would change the shape surface under test).
    let assignments = if kind == "UpdateStatement" {
        "\n    sql:assignColumn \"latency_ms\" ;\n    sql:assignValue \"0\"^^<http://www.w3.org/2001/XMLSchema#integer>"
    } else {
        ""
    };
    format!(
        concat!(
            "@prefix sql: <http://graphlaw.org/sql#> .\n",
            "@prefix ex: <http://example.org/queries#> .\n",
            "ex:Q a sql:{kind} ;\n",
            "    sql:statementIndex 1 ;\n",
            "    sql:targetTable \"telemetry_events\" ;{assignments}{where_clause} ."
        ),
        kind = kind,
        assignments = assignments,
        where_clause = where_clause
    )
}

/// Exact-message pin (L163 convention): WHERE-less UPDATE refuses on the
/// public compile_turtle surface via the dialect-pack SHACL pre-gate. The
/// [REFUSAL-05] UnboundedMutationRefusal arm in src/sql_ast.rs::compile_command
/// sits behind this pre-gate (compile_command is private). Shape layer and
/// compiler are aligned on [REFUSAL-05] UnboundedMutationRefusal: both layers
/// emit the stable marker; the compiler's stricter law requires BOTH
/// sql:whereColumn AND sql:whereValue, matching the shape message below.
#[test]
fn refusal05_where_less_update_is_verbatim_and_pre_engine() {
    let doc = unbounded_mutation_doc("UpdateStatement", false);
    let err = compile_turtle(&doc).expect_err("WHERE-less UPDATE must refuse");
    let msg = err.to_string();
    assert!(
        msg.starts_with("SHACL AST structural violation: Node <<http://example.org/queries#Q>>: "),
        "prefix: {msg}"
    );
    assert!(
        msg.ends_with(
            "MUST carry a WHERE clause (requires BOTH sql:whereColumn and sql:whereValue)."
        ),
        "suffix: {msg}"
    );
    assert!(msg.contains("[REFUSAL-05] UnboundedMutationRefusal:"), "{msg}");
    assert!(admit_bytes_tiered(doc.as_bytes(), None, None, None).is_err());

    // Anti-vacuity twin: bounding the UPDATE compiles to real SQL and runs on
    // a real table, touching exactly one row.
    let twin = compile_turtle(&unbounded_mutation_doc("UpdateStatement", true))
        .expect("bounded UPDATE compiles");
    assert!(
        twin.sql.to_ascii_uppercase().contains(" WHERE ID = "),
        "twin must be bounded: {}",
        twin.sql
    );
    let db = graphlaw::duckdb_eval::GraphLawDuckDB::open_in_memory().unwrap();
    db.raw_connection()
        .execute_batch(
            "CREATE TABLE telemetry_events (id INTEGER, latency_ms INTEGER);
             INSERT INTO telemetry_events VALUES (1, 480), (2, 10);",
        )
        .unwrap();
    db.raw_connection().execute_batch(&twin.sql).unwrap();
    let zeros: i64 = db
        .raw_connection()
        .query_row(
            "SELECT count(*)::BIGINT FROM telemetry_events WHERE latency_ms = 0",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(zeros, 1, "bounded UPDATE mutates exactly the id=1 row");
}

/// Exact-message pin: WHERE-less DELETE — same pre-gate surface, same twin.
#[test]
fn refusal05_where_less_delete_is_verbatim_and_pre_engine() {
    let doc = unbounded_mutation_doc("DeleteStatement", false);
    let err = compile_turtle(&doc).expect_err("WHERE-less DELETE must refuse");
    let msg = err.to_string();
    assert!(
        msg.starts_with("SHACL AST structural violation: Node <<http://example.org/queries#Q>>: "),
        "prefix: {msg}"
    );
    assert!(
        msg.ends_with(
            "MUST carry a WHERE clause (requires BOTH sql:whereColumn and sql:whereValue)."
        ),
        "suffix: {msg}"
    );
    assert!(msg.contains("[REFUSAL-05] UnboundedMutationRefusal:"), "{msg}");
    assert!(admit_bytes_tiered(doc.as_bytes(), None, None, None).is_err());

    // Anti-vacuity twin: the bounded DELETE compiles and removes exactly one
    // row from a real frozen-duckdb table.
    let twin = compile_turtle(&unbounded_mutation_doc("DeleteStatement", true))
        .expect("bounded DELETE compiles");
    assert!(
        twin.sql.to_ascii_uppercase().contains(" WHERE ID = "),
        "twin must be bounded: {}",
        twin.sql
    );
    let db = graphlaw::duckdb_eval::GraphLawDuckDB::open_in_memory().unwrap();
    db.raw_connection()
        .execute_batch(
            "CREATE TABLE telemetry_events (id INTEGER, latency_ms INTEGER);
             INSERT INTO telemetry_events VALUES (1, 480), (2, 10);",
        )
        .unwrap();
    db.raw_connection().execute_batch(&twin.sql).unwrap();
    let rows: i64 = db
        .raw_connection()
        .query_row(
            "SELECT count(*)::BIGINT FROM telemetry_events",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(rows, 1, "bounded DELETE removes exactly the id=1 row");
}

/// SQL-in-RDF is a Turtle dataset on the same ABI path as the other RDF
/// dialects: parse returns a canonical id, and shacl/law accept the dialect.
#[test]
fn sql_rdf_joins_the_rdf_dataset_path() {
    let parsed = graphlaw::abi::call_json(&serde_json::json!({
        "op": "parse",
        "text": SELECT,
        "dialect": "sql",
    }));
    assert_eq!(parsed["ok"], true, "{parsed}");
    assert!(
        parsed["id"].as_str().unwrap_or("").starts_with("blake3:"),
        "{parsed}"
    );
    assert!(parsed["quads"].as_u64().unwrap_or(0) > 0, "{parsed}");

    let broken = graphlaw::abi::call_json(&serde_json::json!({
        "op": "parse",
        "text": "@prefix sql: <http://graphlaw.org/sql#> .\n<urn:q> a sql:SelectStatement .\n",
        "dialect": "sql",
    }));
    assert_eq!(broken["ok"], false, "{broken}");
    assert!(broken.get("id").is_none(), "{broken}");

    let shacl = graphlaw::abi::call_json(&serde_json::json!({
        "op": "shacl",
        "data": {"text": SELECT, "dialect": "sql"},
        "shapes": include_str!("../ontologies/sql-ast.ttl"),
    }));
    let shacl_msg = shacl
        .pointer("/error/message")
        .and_then(|m| m.as_str())
        .unwrap_or("");
    assert_ne!(
        shacl_msg, "dialect does not denote an RDF dataset",
        "{shacl}"
    );
    assert!(
        shacl.get("conforms").is_some() || shacl["ok"] == true,
        "{shacl}"
    );

    let law = graphlaw::abi::call_json(&serde_json::json!({
        "op": "law",
        "data": {"text": SELECT, "dialect": "sql"},
        "steps": [{"step": "shacl", "shapes": include_str!("../ontologies/sql-ast.ttl")}],
    }));
    assert_eq!(law["ok"], true, "{law}");
    let nquads = law["nquads"].as_str().unwrap_or("");
    assert!(
        nquads.contains("http://graphlaw.org/sql#SelectStatement"),
        "{law}"
    );
}
