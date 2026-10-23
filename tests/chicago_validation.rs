//! Every PEG statement class compiles from RDF and runs on a real frozen-duckdb
//! connection, or is refused at compile time when this DuckDB build cannot run it.

use graphlaw::duckdb_eval::GraphLawDuckDB;
use graphlaw::receipt_chain::frame::{
    DenialPolarity, OcelCausalFrame, OcelCausalReceipt, PackedObjRef,
};
use graphlaw::sql_ast::{SqlLiteral, compile_turtle};
use graphlaw::sql_statement_kinds::SQL_STATEMENTS;

const KEY: &str = "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAt9+KeDkWJa7SiBb7AMdXAPV2qpRunztoC2c7DtRmN9+Lq2YQm4rMQmmAQcwyxTxcK8vwPKOiE4mLMwmQMvTty9tL+buREfvg5KJ+FY3Bv05cZYsf7VuYgUDcw41zSEE5W1thh5PiafcrMn7CPF0jN0oFfVvAHB/fAM0yaZGPKs3WYqnHAHa6v/cVMeiWZ/lQORCBqqL2hii1fEiuIqw6AOEoZVsN0/lqm7GfnD899zrzLOPRuP4YFg46drL1UyNdg+4TquJ2OV9kR4CAfdGRexqHFWLmoxkTfnUxJ4yL+gkV57K1H+smEdKrDhDX68pEFM/34E2eVKZoRU/0paxDZQIDAQAB";

fn doc(body: &str) -> String {
    format!(
        "@prefix sql: <http://graphlaw.org/sql#> .\n@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n{body}\n<http://example.org/s> sql:statementIndex 1 ."
    )
}

fn turtle(kind: &str) -> String {
    let body = match kind {
        "SelectStatement" => r#"<http://example.org/s> a sql:SelectStatement ; sql:select ( [ a sql:ColumnIdentifier ; sql:columnName "id" ; sql:scope sql:ViewScope ] ) ; sql:from ( [ a sql:TableIdentifier ; sql:tableName "t" ] ) ."#.into(),
        "InsertStatement" => r#"<http://example.org/s> a sql:InsertStatement ; sql:targetTable "t" ; sql:assignColumn "id" ; sql:assignValue "2"^^xsd:integer ."#.into(),
        "UpdateStatement" => r#"<http://example.org/s> a sql:UpdateStatement ; sql:targetTable "t" ; sql:assignColumn "name" ; sql:assignValue "z" ; sql:whereColumn "id" ; sql:whereValue "1"^^xsd:integer ."#.into(),
        "DeleteStatement" => r#"<http://example.org/s> a sql:DeleteStatement ; sql:targetTable "t" ; sql:whereColumn "id" ; sql:whereValue "1"^^xsd:integer ."#.into(),
        "CreateStatement" => r#"<http://example.org/s> a sql:CreateStatement ; sql:targetTable "t2" ; sql:assignColumn "id" ."#.into(),
        "DropStatement" => r#"<http://example.org/s> a sql:DropStatement ; sql:targetTable "t2" ."#.into(),
        "AlterStatement" => r#"<http://example.org/s> a sql:AlterStatement ; sql:targetTable "t" ; sql:assignColumn "extra" ."#.into(),
        "TruncateStatement" => r#"<http://example.org/s> a sql:TruncateStatement ; sql:targetTable "t" ."#.into(),
        "AnalyzeStatement" => r#"<http://example.org/s> a sql:AnalyzeStatement ; sql:targetTable "t" ."#.into(),
        "CopyStatement" => r#"<http://example.org/s> a sql:CopyStatement ; sql:targetTable "t" ."#.into(),
        "ExplainStatement" => r#"<http://example.org/s> a sql:ExplainStatement ; sql:targetTable "t" ; sql:assignColumn "id" ."#.into(),
        // §7-legal MERGE: usingSource SelectStatement + mergeOn + whenMatched
        // (rec section 7; compiler-MissingProperty and shape-104 both require them).
        "MergeIntoStatement" => r#"<http://example.org/s> a sql:MergeIntoStatement ; sql:targetTable [ a sql:TableIdentifier ; sql:tableName "t" ] ; sql:usingSource [ a sql:SelectStatement ; sql:statementIndex 1 ; sql:select ( [ a sql:ColumnIdentifier ; sql:columnName "id" ; sql:scope sql:ViewScope ] ) ; sql:from ( [ a sql:TableIdentifier ; sql:tableName "t" ] ) ] ; sql:mergeOn [ a sql:BinaryExpression ; sql:left [ a sql:ColumnIdentifier ; sql:qualifier "t" ; sql:columnName "id" ; sql:scope sql:ProcessScope ] ; sql:operator "=" ; sql:right [ a sql:ColumnIdentifier ; sql:qualifier "src" ; sql:columnName "id" ; sql:scope sql:ProcessScope ] ] ; sql:whenMatched [ a sql:MergeUpdateAction ; sql:setAssignments ( [ a sql:Assignment ; sql:targetColumn "id" ; sql:valueExpression [ a sql:ColumnIdentifier ; sql:qualifier "src" ; sql:columnName "id" ; sql:scope sql:ProcessScope ] ] ) ] ."#.into(),
        "CommentStatement" => r#"<http://example.org/s> a sql:CommentStatement ; sql:targetTable "t" ; sql:assignValue "graphlaw" ."#.into(),
        "SetStatement" => r#"<http://example.org/s> a sql:SetStatement ; sql:targetTable "threads" ; sql:assignValue "1"^^xsd:integer ."#.into(),
        "ResetStatement" => r#"<http://example.org/s> a sql:ResetStatement ; sql:targetTable "threads" ."#.into(),
        "PragmaStatement" => r#"<http://example.org/s> a sql:PragmaStatement ; sql:targetTable "version" ."#.into(),
        "CallStatement" => r#"<http://example.org/s> a sql:CallStatement ; sql:targetTable "pragma_database_size" ."#.into(),
        "PrepareStatement" => r#"<http://example.org/s> a sql:PrepareStatement ; sql:targetTable "q" ; sql:assignValue "1"^^xsd:integer ."#.into(),
        "ExecuteStatement" => r#"<http://example.org/s> a sql:ExecuteStatement ; sql:targetTable "q" ; sql:assignValue "1"^^xsd:integer ."#.into(),
        "DeallocateStatement" => r#"<http://example.org/s> a sql:DeallocateStatement ; sql:targetTable "q" ."#.into(),
        "LoadStatement" => r#"<http://example.org/s> a sql:LoadStatement ; sql:targetTable "icu" ."#.into(),
        // Env-derived extensions dir: machine-independent fixture (lane M09).
        "InstallStatement" => format!(
            r#"<http://example.org/s> a sql:InstallStatement ; sql:targetTable "tpch" ; sql:assignValue "{}" ."#,
            std::env::temp_dir().join("graphlaw-duckdb-extensions").display()
        ),
        "AttachStatement" => r#"<http://example.org/s> a sql:AttachStatement ; sql:targetTable "mem" ."#.into(),
        "UseStatement" => r#"<http://example.org/s> a sql:UseStatement ; sql:targetTable "memu" ."#.into(),
        "DetachStatement" => r#"<http://example.org/s> a sql:DetachStatement ; sql:targetTable "memd" ."#.into(),
        "ExportStatement" => r#"<http://example.org/s> a sql:ExportStatement ; sql:targetTable "exp" ."#.into(),
        "ImportStatement" => r#"<http://example.org/s> a sql:ImportStatement ; sql:targetTable "impsrc" ."#.into(),
        "ExtensionRepositoryStatement" => format!(
            r#"<http://example.org/s> a sql:ExtensionRepositoryStatement ; sql:targetTable "repo" ; sql:assignColumn "/tmp/graphlaw-ext-repo" ; sql:assignValue "{KEY}" ."#
        ),
        other => format!(r#"<http://example.org/s> a sql:{other} ."#),
    };
    doc(&body)
}

#[test]
fn every_peg_statement_compiles_and_runs_or_refuses() {
    let mut failures = Vec::new();
    for (kind, _) in SQL_STATEMENTS {
        let compiled = compile_turtle(&turtle(kind));
        if *kind == "ConnectStatement" || *kind == "DisconnectStatement" {
            match compiled {
                Err(e) if e.to_string().contains("compile refusal") => {}
                other => failures.push(format!("{kind}: expected compile refusal, got {other:?}")),
            }
            continue;
        }
        let sql = match compiled {
            Ok(q) => q.sql,
            Err(e) => {
                failures.push(format!("{kind}: compile: {e}"));
                continue;
            }
        };
        let db = GraphLawDuckDB::open_in_memory_allowing_unsigned_extensions().unwrap();
        db.raw_connection()
            .execute_batch(
                "CREATE TABLE t(id INTEGER, name VARCHAR); INSERT INTO t VALUES (1, 'a');",
            )
            .unwrap();
        if let Err(e) = db.raw_connection().execute_batch(&sql) {
            failures.push(format!("{kind}: run: {e}\n  sql: {sql}"));
        }
    }
    assert!(
        failures.is_empty(),
        "unfinished statement classes:\n{}",
        failures.join("\n")
    );
}

#[test]
fn dml_changes_row_counts_on_a_real_database() {
    let db = GraphLawDuckDB::open_in_memory().unwrap();
    db.raw_connection()
        .execute_batch("CREATE TABLE t(id INTEGER, name VARCHAR); INSERT INTO t VALUES (1, 'a');")
        .unwrap();
    let count = |sql: &str| {
        let q = compile_turtle(&turtle(sql)).unwrap();
        db.raw_connection().execute_batch(&q.sql).unwrap();
        db.raw_connection()
            .query_row("SELECT count(*)::BIGINT FROM t", [], |r| r.get::<_, i64>(0))
            .unwrap()
    };
    assert_eq!(count("InsertStatement"), 2);
    assert_eq!(count("UpdateStatement"), 2);
    assert_eq!(count("DeleteStatement"), 1);
}

#[test]
fn hostile_identifier_is_refused_and_does_not_change_rows() {
    let db = GraphLawDuckDB::open_in_memory().unwrap();
    db.raw_connection()
        .execute_batch("CREATE TABLE t(id INTEGER, name VARCHAR); INSERT INTO t VALUES (1, 'a');")
        .unwrap();
    let hostile = doc(
        r#"<http://example.org/s> a sql:InsertStatement ; sql:targetTable "t; drop table t" ; sql:assignColumn "id" ; sql:assignValue "9"^^xsd:integer ."#,
    );
    assert!(compile_turtle(&hostile).is_err());
    let rows: i64 = db
        .raw_connection()
        .query_row("SELECT count(*)::BIGINT FROM t", [], |r| r.get(0))
        .unwrap();
    assert_eq!(rows, 1);
}

fn seed(db: &GraphLawDuckDB) {
    db.raw_connection()
        .execute_batch("CREATE TABLE t(id INTEGER, name VARCHAR); INSERT INTO t VALUES (1, 'a');")
        .unwrap();
}

/// BLAKE3 of the executed SQL plus the ordered table readback. The SQL text
/// alone is not the receipt.
fn state_receipt(kind: &str) -> [u8; 32] {
    let db = GraphLawDuckDB::open_in_memory().unwrap();
    seed(&db);
    let sql = compile_turtle(&turtle(kind)).unwrap().sql;
    db.raw_connection().execute_batch(&sql).unwrap();
    let mut stmt = db
        .raw_connection()
        .prepare("SELECT id, COALESCE(name, '') FROM t ORDER BY id, name")
        .unwrap();
    let rows: Vec<(i64, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    let mut hasher = blake3::Hasher::new();
    hasher.update(sql.as_bytes());
    hasher.update(&[0xff]);
    for (id, name) in &rows {
        hasher.update(&id.to_le_bytes());
        hasher.update(name.as_bytes());
        hasher.update(&[0]);
    }
    *hasher.finalize().as_bytes()
}

#[test]
fn dml_receipts_bind_readback_and_distinguish_statements() {
    let insert = state_receipt("InsertStatement");
    let insert_again = state_receipt("InsertStatement");
    let update = state_receipt("UpdateStatement");
    let delete = state_receipt("DeleteStatement");
    let merge = state_receipt("MergeIntoStatement");
    assert_eq!(insert, insert_again);
    assert_ne!(insert, [0u8; 32]);
    assert_ne!(update, insert);
    assert_ne!(delete, insert);
    assert_ne!(delete, update);
    assert_ne!(merge, insert);
    assert_ne!(merge, update);
    assert_ne!(merge, delete);
}

// ---------------------------------------------------------------------------
// SQLRDF-04 — rec §7 analytical MERGE + §1.4 H_query = BLAKE3(Canon(G_AST) || P_params)
// ---------------------------------------------------------------------------

/// The section 7 sample graph, verbatim per `docs/reference/sql-rdf-rec.md` §7
/// except the status literal, which is a court parameter for the mutation check.
fn section7_merge_graph(status_literal: &str) -> String {
    let body = format!(
        r#"_:mergeStatement a sql:MergeIntoStatement ;
    sql:statementIndex 1 ;
    sql:targetTable [
        a sql:TableIdentifier ;
        sql:schemaName "main" ;
        sql:tableName "regional_sales"
    ] ;
    sql:usingSource [
        a sql:SelectStatement ;
        sql:statementIndex 1 ;
        sql:select (
            [ a sql:ColumnIdentifier ;
              sql:columnName "region_id" ;
              sql:scope sql:ViewScope ]
            [ a sql:FunctionExpression ;
              sql:functionName "sum" ;
              sql:arguments ( [ a sql:ColumnIdentifier ; sql:columnName "amount" ; sql:scope sql:ProcessScope ] ) ;
              sql:alias "total_invoiced" ;
              sql:scope sql:ViewScope ]
        ) ;
        sql:from (
            [ a sql:TableIdentifier ;
              sql:schemaName "staging" ;
              sql:tableName "invoices" ]
        ) ;
        sql:where [
            a sql:BinaryExpression ;
            sql:left [ a sql:ColumnIdentifier ; sql:columnName "status" ; sql:scope sql:ProcessScope ] ;
            sql:operator "=" ;
            sql:right [ a sql:LiteralExpression ; sql:value "{status_literal}" ; sql:datatype xsd:string ]
        ] ;
        sql:groupBy (
            [ a sql:ColumnIdentifier ; sql:columnName "region_id" ; sql:scope sql:ProcessScope ]
        )
    ] ;
    sql:mergeOn [
        a sql:BinaryExpression ;
        sql:left [ a sql:ColumnIdentifier ; sql:qualifier "main.regional_sales" ; sql:columnName "region_id" ; sql:scope sql:ProcessScope ] ;
        sql:operator "=" ;
        sql:right [ a sql:ColumnIdentifier ; sql:qualifier "src" ; sql:columnName "region_id" ; sql:scope sql:ProcessScope ]
    ] ;
    sql:whenMatched [
        a sql:MergeUpdateAction ;
        sql:setAssignments (
            [ a sql:Assignment ;
              sql:targetColumn "revenue" ;
              sql:valueExpression [
                  a sql:BinaryExpression ;
                  sql:left [ a sql:ColumnIdentifier ; sql:qualifier "main.regional_sales" ; sql:columnName "revenue" ; sql:scope sql:ProcessScope ] ;
                  sql:operator "+" ;
                  sql:right [ a sql:ColumnIdentifier ; sql:qualifier "src" ; sql:columnName "total_invoiced" ; sql:scope sql:ProcessScope ]
              ]
            ]
        )
    ] ."#
    );
    format!(
        "@prefix sql: <http://graphlaw.org/sql#> .\n@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n{body}"
    )
}

/// BLAKE3 over the RDFC-1.0 canonical form of the AST graph (rec §1.4 Canon(G_AST)).
fn canon_bytes(turtle: &str) -> Vec<u8> {
    let dataset = graphlaw::rdf::parse_dataset(turtle.as_bytes(), "text/turtle", None)
        .expect("AST graph must parse");
    let canon = graphlaw::rdf::try_canonicalize(&dataset).expect("AST graph must canonicalize");
    canon.nquads.as_bytes().to_vec()
}

/// Ordered parameter buffer P_params (rec §1.4): type-tagged little-endian bytes.
fn params_buffer(params: &[SqlLiteral]) -> Vec<u8> {
    let mut buf = Vec::new();
    for p in params {
        match p {
            SqlLiteral::String(s) => {
                buf.push(0x01);
                buf.extend_from_slice(s.as_bytes());
            }
            SqlLiteral::Integer(i) => {
                buf.push(0x02);
                buf.extend_from_slice(&i.to_le_bytes());
            }
            SqlLiteral::Float(f) => {
                buf.push(0x03);
                buf.extend_from_slice(&f.to_bits().to_le_bytes());
            }
            SqlLiteral::Boolean(b) => {
                buf.push(0x04);
                buf.extend_from_slice(&[*b as u8]);
            }
            SqlLiteral::Null => buf.push(0x00),
        }
    }
    buf
}

/// H_query per rec §1.4, computed independently of the engine.
fn h_query(turtle: &str, params: &[SqlLiteral]) -> [u8; 32] {
    // TC-HOT-01 two-node reference: h_ast = BLAKE3(Canon(G_AST)), then
    // H_query = BLAKE3(h_ast || P_params) (rec section 1.4 first formula).
    let h_ast = *blake3::hash(&canon_bytes(turtle)).as_bytes();
    let mut hasher = blake3::Hasher::new();
    hasher.update(&h_ast);
    hasher.update(&params_buffer(params));
    *hasher.finalize().as_bytes()
}

/// Seeds the two real tables the §7 MERGE reads and writes.
fn seed_section7(db: &GraphLawDuckDB, third_invoice_settled: bool) {
    let third = if third_invoice_settled {
        "SETTLED"
    } else {
        "PENDING"
    };
    db.raw_connection()
        .execute_batch(&format!(
            "CREATE SCHEMA staging;
             CREATE TABLE staging.invoices(region_id INTEGER, amount DECIMAL, status VARCHAR);
             INSERT INTO staging.invoices VALUES
               (1, 30, 'SETTLED'), (1, 12, 'SETTLED'), (2, 99, 'PENDING'),
               (3, 7, '{third}');
             CREATE TABLE main.regional_sales(region_id INTEGER PRIMARY KEY, revenue DECIMAL);
             INSERT INTO main.regional_sales VALUES (1, 100), (2, 50), (3, 0);"
        ))
        .unwrap();
}

/// Runs the §7 MERGE on real tables and returns the engine receipt plus the
/// region→revenue readback.
fn run_section7_merge(
    status_literal: &str,
    third_invoice_settled: bool,
) -> (graphlaw::duckdb_eval::ExecutionReceipt, Vec<(i64, f64)>) {
    let graph = section7_merge_graph(status_literal);
    let compiled = compile_turtle(&graph).expect("section 7 graph must compile");
    for (fragment, present) in [
        ("MERGE INTO main.regional_sales", true),
        ("FROM staging.invoices", true),
        ("WHERE (status = $1)", true),
        ("GROUP BY region_id", true),
        ("WHEN MATCHED THEN UPDATE SET revenue", true),
        ("WHEN NOT MATCHED", false),
    ] {
        assert_eq!(
            compiled.sql.contains(fragment),
            present,
            "sql: {}\nfragment: {fragment}",
            compiled.sql
        );
    }
    assert_eq!(
        compiled.params,
        vec![SqlLiteral::String(status_literal.into())]
    );

    let db = GraphLawDuckDB::open_in_memory().unwrap();
    seed_section7(&db, third_invoice_settled);
    let receipt = db
        .eval_ast_query("http://example.org/merge7", &compiled, "urn:r:", "urn:p:")
        .expect("section 7 MERGE must execute");
    let mut stmt = db
        .raw_connection()
        .prepare("SELECT region_id, revenue::FLOAT8 FROM main.regional_sales ORDER BY region_id")
        .unwrap();
    let rows: Vec<(i64, f64)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    (receipt, rows)
}

#[test]
fn section7_merge_upserts_regional_sales_on_real_tables() {
    let (_, rows) = run_section7_merge("SETTLED", true);
    assert_eq!(
        rows,
        vec![(1, 142.0), (2, 50.0), (3, 7.0)],
        "MERGE must add SETTLED invoice sums into regional_sales"
    );

    // Anti-vacuity (rec §6.1.2): mutating one data row must flip the outcome.
    let (_, mutated) = run_section7_merge("SETTLED", false);
    assert_eq!(
        mutated,
        vec![(1, 142.0), (2, 50.0), (3, 0.0)],
        "flipping region 3's only SETTLED invoice to PENDING must leave its revenue at 0"
    );
}

#[test]
fn merge_receipt_is_blake3_of_canonical_graph_and_params_not_sql_text() {
    // Replay: same graph + same buffer → same receipt.
    let (receipt, _) = run_section7_merge("SETTLED", true);
    let (replayed, _) = run_section7_merge("SETTLED", true);
    let graph = section7_merge_graph("SETTLED");
    let expected = h_query(&graph, &[SqlLiteral::String("SETTLED".into())]);
    assert_eq!(
        receipt.query_hash, expected,
        "receipt must be BLAKE3(Canon(G_AST) || P_params)"
    );
    assert_eq!(receipt.query_hash, replayed.query_hash, "replay must match");

    // Mutation: changing the literal changes the receipt.
    let (other, _) = run_section7_merge("REFUNDED", true);
    assert_ne!(receipt.query_hash, other.query_hash);

    // Hashing the SQL text alone FAILS the court (rec §1.4).
    assert_ne!(
        receipt.query_hash,
        *blake3::hash(receipt.sql_statement.as_bytes()).as_bytes(),
        "query_hash must not be a hash of the SQL text"
    );

    // The state readback witness (SQL + table readback) stays a SEPARATE digest
    // from H_query — SQLRDF-09 keeps both witnesses distinct.
    assert_ne!(receipt.query_hash, {
        let mut hasher = blake3::Hasher::new();
        hasher.update(receipt.sql_statement.as_bytes());
        hasher.update(&[0xff]);
        *hasher.finalize().as_bytes()
    });
}

// ---------------------------------------------------------------------------
// SQLRDF-07 — rec §1.3/§5, REFUSAL-03 ScopeIntegrityRefusal
// ---------------------------------------------------------------------------

fn scoped_select_graph(select_scope: &str, where_expr: Option<&str>) -> String {
    let where_clause = match where_expr {
        Some(w) => format!(" ;\n        sql:where {w}"),
        None => String::new(),
    };
    format!(
        "@prefix sql: <http://graphlaw.org/sql#> .\n@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n_:s a sql:SelectStatement ;\n    sql:statementIndex 1 ;\n    sql:select ( [ a sql:ColumnIdentifier ; sql:columnName \"id\" ; sql:scope sql:{select_scope} ] ) ;\n    sql:from ( [ a sql:TableIdentifier ; sql:tableName \"t\" ] ){where_clause} ."
    )
}

const AGGREGATE_WHERE: &str = "[ a sql:BinaryExpression ; sql:operator \"=\" ; \
     sql:left [ a sql:FunctionExpression ; sql:functionName \"sum\" ; \
       sql:arguments ( [ a sql:ColumnIdentifier ; sql:columnName \"amount\" ; sql:scope sql:ProcessScope ] ) ] ; \
     sql:right [ a sql:LiteralExpression ; sql:value \"1\" ; sql:datatype xsd:integer ] ]";

const PLAIN_WHERE: &str = "[ a sql:BinaryExpression ; sql:operator \"=\" ; \
     sql:left [ a sql:ColumnIdentifier ; sql:columnName \"id\" ; sql:scope sql:ProcessScope ] ; \
     sql:right [ a sql:LiteralExpression ; sql:value \"1\" ; sql:datatype xsd:integer ] ]";

fn refusal03(graph: &str) {
    let err = compile_turtle(graph)
        .expect_err("scope violation must be refused at compile time, before any engine call");
    let msg = err.to_string();
    assert!(
        msg.contains("REFUSAL-03") || msg.contains("ScopeIntegrity"),
        "refusal must be typed REFUSAL-03 ScopeIntegrityRefusal, got: {msg}"
    );
}

#[test]
fn aggregate_in_where_is_refusal_03_and_database_unchanged() {
    let db = GraphLawDuckDB::open_in_memory().unwrap();
    seed(&db);
    refusal03(&scoped_select_graph("ViewScope", Some(AGGREGATE_WHERE)));
    let rows: i64 = db
        .raw_connection()
        .query_row("SELECT count(*)::BIGINT FROM t", [], |r| r.get(0))
        .unwrap();
    assert_eq!(rows, 1, "refused graph must not touch the database");

    // Anti-vacuity: replacing the aggregate with a plain column predicate flips
    // the court — the same graph shape compiles and executes.
    let db2 = GraphLawDuckDB::open_in_memory().unwrap();
    seed(&db2);
    let compiled = compile_turtle(&scoped_select_graph("ViewScope", Some(PLAIN_WHERE)))
        .expect("plain WHERE must compile");
    let receipt = db2
        .eval_ast_query("http://example.org/plain", &compiled, "urn:r:", "urn:p:")
        .expect("plain WHERE must execute");
    assert_eq!(receipt.rows_emitted, 1);
}

#[test]
fn process_scope_in_select_list_is_refusal_03_but_view_scope_executes() {
    let db = GraphLawDuckDB::open_in_memory().unwrap();
    seed(&db);
    refusal03(&scoped_select_graph("ProcessScope", None));
    let rows: i64 = db
        .raw_connection()
        .query_row("SELECT count(*)::BIGINT FROM t", [], |r| r.get(0))
        .unwrap();
    assert_eq!(rows, 1, "refused graph must not touch the database");

    // Anti-vacuity: the ViewScope twin of the same graph executes on the real db.
    let compiled = compile_turtle(&scoped_select_graph("ViewScope", None))
        .expect("ViewScope select list must compile");
    let receipt = db
        .eval_ast_query("http://example.org/view", &compiled, "urn:r:", "urn:p:")
        .expect("ViewScope select list must execute");
    assert_eq!(receipt.rows_emitted, 1);
}

fn object_refs(object: &str) -> [PackedObjRef; 8] {
    let digest = blake3::hash(object.as_bytes());
    let bytes = digest.as_bytes();
    let mut refs = [PackedObjRef(0); 8];
    for i in 0..8 {
        let word = u32::from_le_bytes(bytes[i * 4..i * 4 + 4].try_into().unwrap());
        refs[i] = PackedObjRef(word);
    }
    refs
}

fn push_frame(
    receipt: &mut OcelCausalReceipt,
    tick: u64,
    activity: u16,
    object: &str,
    denial: DenialPolarity,
) {
    let frame = OcelCausalFrame {
        instruction_id: tick,
        fired_mask: denial.to_fired_mask(),
        denial,
        obj_refs: object_refs(object),
        ts_ns: tick,
        activity_idx: activity,
        node_kind: u8::from(!denial.is_admitted()),
        pad: [0; 5],
        prior_hash: receipt.chain_hash,
    };
    receipt.chain(&frame);
}

fn statement_object(kind: &str) -> (DenialPolarity, String) {
    match compile_turtle(&turtle(kind)) {
        Ok(q) if kind == "ConnectStatement" || kind == "DisconnectStatement" => {
            panic!("{kind} compiled: {}", q.sql)
        }
        Err(e) if kind == "ConnectStatement" || kind == "DisconnectStatement" => {
            (DenialPolarity::CONFORMANCE_GATE_FAILED, e.to_string())
        }
        Ok(q) => (DenialPolarity::ADMITTED, q.sql),
        Err(e) => panic!("{kind} refused: {e}"),
    }
}

fn sealed_trace(drop_hostile: bool, rewrite_first_activity: bool) -> [u8; 32] {
    let mut receipt = OcelCausalReceipt::genesis([0u8; 32]);
    for (i, (kind, _)) in SQL_STATEMENTS.iter().enumerate() {
        let (denial, object) = statement_object(kind);
        let activity = if rewrite_first_activity && i == 0 {
            0xFFFE
        } else {
            u16::try_from(i).unwrap()
        };
        push_frame(&mut receipt, i as u64 + 1, activity, &object, denial);
    }
    if !drop_hostile {
        let hostile = doc(
            r#"<http://example.org/s> a sql:InsertStatement ; sql:targetTable "t; drop table t" ; sql:assignColumn "id" ; sql:assignValue "9"^^xsd:integer ."#,
        );
        let refusal = compile_turtle(&hostile).unwrap_err().to_string();
        push_frame(
            &mut receipt,
            SQL_STATEMENTS.len() as u64 + 1,
            0xFFFF,
            &refusal,
            DenialPolarity::CONFORMANCE_GATE_FAILED,
        );
    }
    receipt.chain_hash
}

#[test]
fn ocel_trace_seals_every_statement_and_the_hostile_refusal() {
    let sealed = sealed_trace(false, false);
    let replay = sealed_trace(false, false);
    let dropped = sealed_trace(true, false);
    let rewritten = sealed_trace(false, true);
    assert_eq!(sealed, replay);
    assert_ne!(sealed, dropped);
    assert_ne!(sealed, rewritten);
    assert_ne!(sealed, [0u8; 32]);

    let db = GraphLawDuckDB::open_in_memory().unwrap();
    seed(&db);
    let hostile = doc(
        r#"<http://example.org/s> a sql:InsertStatement ; sql:targetTable "t; drop table t" ; sql:assignColumn "id" ; sql:assignValue "9"^^xsd:integer ."#,
    );
    assert!(compile_turtle(&hostile).is_err());
    let rows: i64 = db
        .raw_connection()
        .query_row("SELECT count(*)::BIGINT FROM t", [], |r| r.get(0))
        .unwrap();
    assert_eq!(rows, 1);
}

// ---------------------------------------------------------------------------
// Lane M07 (GUARD BUILDER) — K1 H2 rowcount-receipt strengthening.
// K1's named defect class: a `rows_emitted >= 1` floor cannot detect a
// dropped filter (WHERE removed -> 3 rows) or an inverted predicate
// (> 400 -> < 400 leaves only beta). These courts assert the EXACT row
// count AND the exact values read back from the COMPILED query's own
// output rows, projected by eval_ast_query into ExecutionReceipt
// .result_triples — no twin SQL, no hand-written second statement.
// ---------------------------------------------------------------------------
mod rowcount_receipt_courts {
    use graphlaw::duckdb_eval::ExecutionReceipt;
    use graphlaw::tier::{Admitted, admit_bytes_tiered};

    use super::*;

    fn telemetry_db() -> GraphLawDuckDB {
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
    /// hand-written twin of the query).
    fn column_values(receipt: &ExecutionReceipt, column: &str) -> Vec<String> {
        let suffix = format!("/{column}");
        let mut values: Vec<String> = receipt
            .result_triples
            .iter()
            .filter(|t| t.predicate.ends_with(&suffix))
            .filter_map(|t| match &t.object {
                purrdf::TermValue::Literal { lexical_form, .. } => Some(lexical_form.clone()),
                // Non-literal objects carry no column value; skip (compile-freeze
                // unblock, lane M08 disclosure: K-lane in-flight file, minimal fix).
                _ => None,
            })
            .collect();
        values.sort();
        values
    }

    /// (a) Strengthened `admitted_ast_executes_on_frozen_duckdb`: the admitted
    /// SELECT (latency_ms > 400) emits EXACTLY rows_emitted == 2 and EXACTLY
    /// batch_id values [alpha, gamma]. The dropped-filter mutation (no WHERE)
    /// would give 3 values including beta; the inverted-filter mutation
    /// (latency_ms < 400) would give only beta — both now fail loudly.
    #[test]
    fn admitted_select_exact_rowcount_and_exact_values_readback() {
        let doc = r#"
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
        let compiled = match admit_bytes_tiered(doc.as_bytes(), None, None, None).unwrap() {
            Admitted::Query(q) => q,
            other => panic!("expected a compiled query, got {other:?}"),
        };
        assert_eq!(compiled.params, vec![SqlLiteral::Integer(400)]);

        let db = telemetry_db();
        let receipt = db
            .eval_ast_query(
                "http://example.org/queries#Q",
                &compiled,
                "http://example.org/data",
                "http://example.org/vocab",
            )
            .unwrap();
        assert_ne!(receipt.query_hash, [0u8; 32]);
        // Exact rowcount floor: dropped WHERE -> 3, inverted predicate -> 1.
        assert_eq!(receipt.rows_emitted, 2, "exact rowcount, not a >= 1 floor");
        // Exact values: beta (10) must be filtered OUT by the compiled WHERE.
        let ids = column_values(&receipt, "batch_id");
        assert_eq!(ids, vec!["alpha", "gamma"]);
        assert!(
            !ids.contains(&"beta".to_string()),
            "inverted/dropped filter would re-admit beta"
        );
    }

    /// (b) Strengthened `group_having_order_by_select_executes_with_table_
    /// readback`: the COMPILED query (GROUP BY + HAVING SUM >= $1 + ORDER BY
    /// batch_id DESC) is executed as-is via eval_ast_query, and its OWN output
    /// rows are read back from the receipt — the previous twin-SQL readback
    /// could pass while the compiled statement diverged from the hand-written
    /// copy. Expected output rows: (alpha, 480) and (gamma, 900); beta (10)
    /// fails HAVING.
    #[test]
    fn compiled_group_having_order_by_reads_back_its_own_rows() {
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
        assert_eq!(compiled.params, vec![SqlLiteral::Integer(100)]);

        let db = telemetry_db();
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
}

// ---------------------------------------------------------------------------
// CTQ-4 / REFUSAL-05 end-to-end: a WHERE-less DELETE refuses at compile time
// (pre-engine) and the real in-memory database is byte-identical afterwards;
// the predicate-bound twin mutates exactly one row.
// ---------------------------------------------------------------------------

#[test]
fn where_less_delete_refuses_pre_engine_and_database_is_byte_identical() {
    let refused = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
<http://example.org/s> a sql:DeleteStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "t" .
"#;
    let bounded = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
<http://example.org/s> a sql:DeleteStatement ;
    sql:statementIndex 1 ;
    sql:targetTable "t" ;
    sql:whereColumn "id" ;
    sql:whereValue "1"^^xsd:integer .
"#;
    let readback = |db: &GraphLawDuckDB| -> Vec<(i64, String)> {
        let mut stmt = db
            .raw_connection()
            .prepare("SELECT id, COALESCE(name, '') FROM t ORDER BY id, name")
            .unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .map(|r| r.unwrap())
            .collect()
    };

    // Refusal path: compile refuses typed, before any engine contact, and the
    // seeded database readback is byte-identical after the attempt.
    let db = GraphLawDuckDB::open_in_memory().unwrap();
    seed(&db);
    let before = readback(&db);
    let err = compile_turtle(refused).expect_err("WHERE-less DELETE must refuse");
    assert!(
        err.to_string().contains("MUST carry a WHERE clause"),
        "{err}"
    );
    assert_eq!(
        readback(&db),
        before,
        "a pre-engine refusal must leave the real database byte-identical"
    );

    // Twin: the bounded DELETE mutates exactly one row (3 -> 2, id=1 gone).
    let q = compile_turtle(bounded).expect("bounded DELETE compiles");
    db.raw_connection().execute_batch(&q.sql).unwrap();
    let after = readback(&db);
    assert_eq!(after.len(), before.len() - 1, "{after:?}");
    assert!(
        !after.iter().any(|(id, _)| *id == 1),
        "bounded DELETE must remove exactly the id=1 row: {after:?}"
    );
}
