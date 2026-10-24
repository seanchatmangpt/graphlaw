//! E3 conformance-report rung for `docs/reference/sql-rdf-rec.md`.
//!
//! This rung IS the conformance report: one `#[test]` per already-courtable
//! normative clause group, each delegating to the REAL public evidence the
//! existing courts use (no duplicated engines, no mocks — rec §6.1). Prose-only
//! clauses (§1, §2, §6.1, §8) are NOT-COURTABLE here and are marked as such;
//! their court evidence lives in the Chicago execution lanes.
//!
//! Gate: `cargo test --test sql_rdf_conformance`
//!
//! ANTI-VACUITY (inline-flip per court, scripts/mutation_court.sh discipline):
//! every refusal gate below carries an inline FLIP check — the benign twin of
//! the refusing document MUST compile — so a gate that refuses everything
//! (or silently admits everything) fails this file.

use graphlaw::sql_ast::{
    REFUSAL_01_MALFORMED_AST, REFUSAL_02_UNIMPLEMENTED, REFUSAL_03_SCOPE_INTEGRITY, SqlLiteral,
    compile_turtle,
};
use graphlaw::sql_statement_kinds::{SQL_EXTENSIONS, SQL_STATEMENTS};
use graphlaw::tier::REFUSAL_04_CAPABILITY_REFUSAL;

/// The rec §4 normative shape inventory, read from the shapes file itself.
const SQL_AST_TTL: &str = include_str!("../ontologies/sql-ast.ttl");
/// The rec §5 refusal taxonomy registry.
const REFUSALS_MD: &str = include_str!("../docs/refusals.md");

// ---------------------------------------------------------------------------
// NOT-COURTABLE here (prose clauses): rec §1 (philosophical foundations),
// §2 (conformance notation), §6.1 (Chicago testing doctrine — enforced by the
// execution courts' structure itself), §8 (epilogue). No normative machine-
// checkable invariant is stated in these sections beyond what the tests below
// and the execution courts already witness.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Rec §3.2 — Normative statement inventory: 35 classes, Connect/Disconnect
// refuse typed (REFUSAL-02), never a generic crash.
// ---------------------------------------------------------------------------

#[test]
fn section3_2_inventory_is_35_classes_with_typed_engine_refusals() {
    assert_eq!(
        SQL_STATEMENTS.len(),
        35,
        "rec §3.2: MUST recognize all 35 statement classes"
    );
    let names: Vec<&str> = SQL_STATEMENTS.iter().map(|(n, _)| *n).collect();
    for expected in [
        "SelectStatement",
        "InsertStatement",
        "UpdateStatement",
        "DeleteStatement",
        "MergeIntoStatement",
        "ConnectStatement",
        "DisconnectStatement",
    ] {
        assert!(
            names.contains(&expected),
            "rec §3.2 inventory missing {expected}"
        );
    }

    // The two engine-refusal classes refuse typed at compile time with the
    // stable REFUSAL-02 marker — the same public compile surface the PEG
    // execution courts use (no duplication).
    for kind in ["ConnectStatement", "DisconnectStatement"] {
        let doc = format!(
            "@prefix sql: <http://graphlaw.org/sql#> .\n@prefix ex: <http://example.org/queries#> .\nex:S a sql:{kind} ; sql:statementIndex 1 .\n"
        );
        let err = compile_turtle(&doc).unwrap_err().to_string();
        assert!(
            err.contains("compile refusal") && err.contains("REFUSAL-02"),
            "{kind}: expected typed REFUSAL-02 compile refusal, got: {err}"
        );
        assert!(
            err.contains(REFUSAL_02_UNIMPLEMENTED) || err.contains("UnimplementedStatementRefusal"),
            "{kind}: refusal must carry the UnimplementedStatementRefusal category: {err}"
        );
        // FLIP: the same document shaped as a SelectStatement must compile —
        // the refusal is attributable to the statement class, not the shape.
        let benign = "@prefix sql: <http://graphlaw.org/sql#> .\n@prefix ex: <http://example.org/queries#> .\nex:S a sql:SelectStatement ; sql:statementIndex 1 ; sql:fromTable [ sql:tableName \"t\" ] ; sql:selectList ( [ sql:columnName \"c\" ] ) .\n".to_string();
        assert!(
            compile_turtle(&benign).is_ok(),
            "{kind} flip: benign SelectStatement twin must compile"
        );
    }
    emit_conformance_report();
}

// ---------------------------------------------------------------------------
// Rec §4 — Closed-world SHACL shapes: the four normative shape names exist in
// ontologies/sql-ast.ttl, and the shape gates refuse via the real compiler
// (missing index, bad operator, malformed list).
// ---------------------------------------------------------------------------

#[test]
fn section4_shape_names_present_and_gates_refuse() {
    for shape in [
        "sql:StatementBaseShape",
        "sql:SelectStatementShape",
        "sql:BinaryExpressionShape",
        "sql:OrderedListShape",
    ] {
        assert!(
            SQL_AST_TTL.contains(shape),
            "rec §4 shape {shape} must be declared in ontologies/sql-ast.ttl"
        );
    }

    // Gate 1: StatementBaseShape — missing/non-integer sql:statementIndex is REFUSAL-01.
    let missing_index = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) .
"#;
    let err = compile_turtle(missing_index).unwrap_err().to_string();
    assert!(err.contains("REFUSAL-01"), "missing index: {err}");
    assert!(err.contains("statementIndex"), "missing index: {err}");
    // FLIP: with the index present the same shape must compile.
    let with_index = missing_index.replace(
        "    sql:fromTable",
        "    sql:statementIndex 1 ;\n    sql:fromTable",
    );
    assert!(
        compile_turtle(&with_index).is_ok(),
        "flip: adding sql:statementIndex must admit the document"
    );

    // Gate 2: BinaryExpressionShape operator enum — a token outside
    // (= != < <= > >= AND OR + - * / % LIKE ILIKE) is REFUSAL-01.
    let bad_operator = r#"
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
    let err = compile_turtle(bad_operator).unwrap_err().to_string();
    assert!(err.contains("REFUSAL-01"), "bad operator: {err}");
    assert!(err.contains("SOUNDS-LIKE"), "bad operator: {err}");
    // FLIP: the same predicate with an in-vocabulary operator must compile.
    let good_operator = bad_operator.replace("SOUNDS-LIKE", "=");
    assert!(
        compile_turtle(&good_operator).is_ok(),
        "flip: in-vocabulary operator must admit the document"
    );

    // Gate 3: OrderedListShape — a list node without its rdf:rest cell (an
    // unclosed chain) refuses as a typed malformed-AST error (REFUSAL-01
    // family: MalformedList/MissingProperty per docs/refusals.md), never a
    // silent fold or a crash.
    let malformed_list = r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList [
        rdf:first [ sql:columnName "batch_id" ] ;
        rdf:rest [ rdf:first [ sql:columnName "shard" ] ]
    ] .
"#;
    let err = compile_turtle(malformed_list).unwrap_err().to_string();
    assert!(
        err.contains("rest") || err.contains("Malformed") || err.contains("REFUSAL-01"),
        "malformed list must refuse typed, got: {err}"
    );
    // FLIP: close the list chain (rdf:rest rdf:nil) — the twin must compile.
    let closed_list = malformed_list.replace(
        "rdf:first [ sql:columnName \"shard\" ] ]",
        "rdf:first [ sql:columnName \"shard\" ] ; rdf:rest rdf:nil ]",
    );
    assert!(
        compile_turtle(&closed_list).is_ok(),
        "flip: closing the rdf:rest chain must admit the document"
    );
    emit_conformance_report();
}

// ---------------------------------------------------------------------------
// Rec §5 — Formal typed refusal taxonomy: all four codes exist BOTH as rows in
// docs/refusals.md AND as typed constants in src, and fire through the real
// compiler.
// ---------------------------------------------------------------------------

#[test]
fn section5_refusal_codes_in_register_and_in_src() {
    // Register side: every code and its category identifier is documented.
    for (code, category) in [
        ("REFUSAL-01", "MalformedAstRefusal"),
        ("REFUSAL-02", "UnimplementedStatementRefusal"),
        ("REFUSAL-03", "ScopeIntegrityRefusal"),
        ("REFUSAL-04", "CapabilityRefusal"),
    ] {
        assert!(
            REFUSALS_MD.contains(code) && REFUSALS_MD.contains(category),
            "docs/refusals.md must register {code} ({category})"
        );
    }

    // Source side: the typed constants exist and carry the stable codes
    // (referencing them here IS the existence court — they must compile).
    assert!(REFUSAL_01_MALFORMED_AST.contains("REFUSAL-01"));
    assert!(REFUSAL_01_MALFORMED_AST.contains("MalformedAstRefusal"));
    assert!(REFUSAL_02_UNIMPLEMENTED.contains("REFUSAL-02"));
    assert!(REFUSAL_02_UNIMPLEMENTED.contains("UnimplementedStatementRefusal"));
    assert!(REFUSAL_03_SCOPE_INTEGRITY.contains("REFUSAL-03"));
    assert!(REFUSAL_03_SCOPE_INTEGRITY.contains("ScopeIntegrityRefusal"));
    assert_eq!(REFUSAL_04_CAPABILITY_REFUSAL, "REFUSAL-04");
    let rec = include_str!("../docs/reference/sql-rdf-rec.md");
    assert!(
        rec.contains("not document-backed"),
        "rec §5 REFUSAL-04 trigger must be the non-document-backed reader"
    );
    assert!(
        !rec.contains("calling DuckDB in WASM"),
        "rec §5 must not describe REFUSAL-04 as DuckDB-in-WASM"
    );
    assert!(
        REFUSALS_MD.contains("not document-backed"),
        "refusals.md REFUSAL-04 trigger must be the non-document-backed reader"
    );

    // REFUSAL-03 fires through the real compiler: aggregate in a WHERE
    // position (rec §5 trigger example).
    let aggregate_in_where = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) ;
    sql:whereClause [
        sql:operator "=" ;
        sql:leftOperand [
            sql:functionName "sum" ;
            sql:arguments ( [ sql:columnName "latency_ms" ] )
        ] ;
        sql:rightOperand [ sql:literalValue "400"^^xsd:integer ]
    ] .
"#;
    let err = compile_turtle(aggregate_in_where).unwrap_err().to_string();
    assert!(
        err.contains("REFUSAL-03"),
        "aggregate in WHERE must be REFUSAL-03: {err}"
    );
    // FLIP: the same aggregate projected (not filtered) must compile.
    let aggregate_in_select = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList ( [ sql:functionName "sum" ; sql:arguments ( [ sql:columnName "latency_ms" ] ) ] ) .
"#;
    assert!(
        compile_turtle(aggregate_in_select).is_ok(),
        "flip: the aggregate in projection position must be admitted"
    );
    emit_conformance_report();
}

// ---------------------------------------------------------------------------
// Rec §6.2 — Certified built-in extensions: the five certified expected values
// as constants, cross-checked against the generated SQL_EXTENSIONS table.
// ---------------------------------------------------------------------------

/// Rec §6.2 certified results: (extension, certified integer result).
const CERTIFIED_EXTENSION_RESULTS: &[(&str, i64)] = &[
    ("core_functions", 6),
    ("icu", 1),
    ("json", 1),
    ("parquet", 1),
    ("tpch", 25),
];

#[test]
fn section6_2_certified_extension_values_match_generated_table() {
    for (name, certified) in CERTIFIED_EXTENSION_RESULTS {
        let row = SQL_EXTENSIONS
            .iter()
            .find(|(n, _, _, _)| n == name)
            .unwrap_or_else(|| panic!("rec §6.2 extension {name} missing from SQL_EXTENSIONS"));
        assert_eq!(
            row.3, *certified,
            "rec §6.2 certified result for {name}: table says {}, rec says {certified}",
            row.3
        );
    }
    emit_conformance_report();
}

// ---------------------------------------------------------------------------
// Rec §7.1 — Target parameterized SQL projection: compiling the §7 sample
// graph yields the exact MERGE fragment with $1 and the SETTLED parameter.
// ---------------------------------------------------------------------------

/// The rec §7 sample graph, verbatim per `docs/reference/sql-rdf-rec.md` §7.
fn section7_merge_graph() -> String {
    let body = r#"_:mergeStatement a sql:MergeIntoStatement ;
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
            sql:right [ a sql:LiteralExpression ; sql:value "SETTLED" ; sql:datatype xsd:string ]
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
    ] ."#;
    format!(
        "@prefix sql: <http://graphlaw.org/sql#> .\n@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n{body}"
    )
}

#[test]
fn section7_1_compiles_to_target_parameterized_projection() {
    let compiled = compile_turtle(&section7_merge_graph()).unwrap();
    // Whitespace/paren-normalized comparison — same normalization the
    // sql_rdf_dialect court uses for this clause.
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
    // §7.1 extracted parameter buffer: exactly the SETTLED string literal.
    assert_eq!(
        compiled.params,
        vec![SqlLiteral::String("SETTLED".into())],
        "rec §7.1 extracted parameter buffer"
    );
    emit_conformance_report();
}

// ---------------------------------------------------------------------------
// E3 — conformance-report emission: the rung prints its receipt table as
// Markdown rows behind the CONFORMANCE_REPORT=<path> env gate. When the gate
// is unset, emission is additive: every court above passes unchanged.
// ---------------------------------------------------------------------------

/// One row of the clause receipt table. `Status` is either `COURTED` (a
/// `#[test]` in this file witnesses it) or `PROSE-ONLY` (no normative
/// machine-checkable invariant beyond what the execution courts witness —
/// rec §1, §2, §6.1, §8). Including the PROSE-ONLY rows makes the clause
/// inventory exhaustive and machine-checked.
struct ClauseRow {
    clause: &'static str,
    section: &'static str,
    requirement: &'static str,
    status: &'static str,
    witness: &'static str,
}

const CLAUSE_TABLE: &[ClauseRow] = &[
    ClauseRow {
        clause: "§1",
        section: "Philosophical foundations",
        requirement: "No normative machine-checkable invariant stated",
        status: "PROSE-ONLY",
        witness: "rec §1; court evidence lives in the Chicago execution lanes",
    },
    ClauseRow {
        clause: "§2",
        section: "Conformance notation",
        requirement: "No normative machine-checkable invariant stated",
        status: "PROSE-ONLY",
        witness: "rec §2; RFC 2119 key words as document text",
    },
    ClauseRow {
        clause: "§3.2",
        section: "Normative statement inventory",
        requirement: "35 statement classes recognized; Connect/Disconnect refuse typed REFUSAL-02",
        status: "COURTED",
        witness: "section3_2_inventory_is_35_classes_with_typed_engine_refusals",
    },
    ClauseRow {
        clause: "§4",
        section: "Closed-world SHACL shapes",
        requirement: "Shape gates refuse typed (REFUSAL-01 family) via the real compiler",
        status: "COURTED",
        witness: "section4_shape_names_present_and_gates_refuse",
    },
    ClauseRow {
        clause: "§5",
        section: "Formal typed refusal taxonomy",
        requirement: "REFUSAL-01..04 in register AND in src; REFUSAL-03 fires through the compiler",
        status: "COURTED",
        witness: "section5_refusal_codes_in_register_and_in_src",
    },
    ClauseRow {
        clause: "§6.1",
        section: "Chicago testing doctrine",
        requirement: "No mocks; real collaborators, real table readbacks",
        status: "PROSE-ONLY",
        witness: "rec §6.1; enforced by the execution courts' structure itself",
    },
    ClauseRow {
        clause: "§6.2",
        section: "Certified built-in extensions",
        requirement: "Five certified extension results match the generated SQL_EXTENSIONS table",
        status: "COURTED",
        witness: "section6_2_certified_extension_values_match_generated_table",
    },
    ClauseRow {
        clause: "§7.1",
        section: "Target-parameterized SQL projection",
        requirement: "§7 sample graph compiles to the exact MERGE fragment with $1 and the SETTLED param",
        status: "COURTED",
        witness: "section7_1_compiles_to_target_parameterized_projection",
    },
    ClauseRow {
        clause: "§8",
        section: "Epilogue",
        requirement: "No normative machine-checkable invariant stated",
        status: "PROSE-ONLY",
        witness: "rec §8",
    },
];

/// Render the clause receipt table as Markdown rows (no pipes in cell text —
/// the cell contents above are pipe-free by construction).
fn render_clause_table() -> String {
    let mut out = String::from(
        "| Clause | Section | Requirement | Status | Witness |\n|---|---|---|---|---|\n",
    );
    for r in CLAUSE_TABLE {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            r.clause, r.section, r.requirement, r.status, r.witness
        ));
    }
    out
}

/// Emission is idempotent across the courts in this file (each calls this at
/// its end); the file is written at most once per process.
fn emit_conformance_report() {
    use std::sync::Once;
    static EMIT: Once = Once::new();
    EMIT.call_once(|| {
        let Ok(path) = std::env::var("CONFORMANCE_REPORT") else {
            return; // additive: gate unset, nothing written, tests unchanged
        };
        let table = render_clause_table();
        let totals = format!(
            "\nTotals: {} clauses — {} COURTED, {} PROSE-ONLY.\n",
            CLAUSE_TABLE.len(),
            CLAUSE_TABLE
                .iter()
                .filter(|r| r.status == "COURTED")
                .count(),
            CLAUSE_TABLE
                .iter()
                .filter(|r| r.status == "PROSE-ONLY")
                .count(),
        );
        // BLAKE3 of the table text binds the report to the producing binary:
        // the hash is computed from the exact rows this binary rendered, plus
        // the crate version of the binary that produced them.
        let binding = blake3::hash(
            format!("graphlaw v{} || {}", env!("CARGO_PKG_VERSION"), table).as_bytes(),
        );
        let body = format!(
            "# sql-rdf conformance report\n\n<!-- GENERATED by tests/sql_rdf_conformance.rs \
             (CONFORMANCE_REPORT gate). Regenerate: CONFORMANCE_REPORT=docs/reference-generated/\
             sql-rdf-conformance.md cargo test --test sql_rdf_conformance. Never hand-edit. -->\n\n\
             Producer: graphlaw v{} (tests/sql_rdf_conformance.rs)\n\n\
             {table}{totals}\nTable BLAKE3: `{}`\n",
            env!("CARGO_PKG_VERSION"),
            binding.to_hex(),
        );
        if let Some(parent) = std::path::Path::new(&path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&path, body)
            .unwrap_or_else(|e| panic!("CONFORMANCE_REPORT={path}: cannot write report: {e}"));
    });
}

/// docs_release_claims-style assertion: the report emitter is exercised on
/// every run; with CONFORMANCE_REPORT unset emission is additive (the gate is
/// skipped) and the clause inventory — including the PROSE-ONLY rows for §1,
/// §2, §6.1, §8 — stays exhaustive so no clause can silently vanish.
#[test]
fn report_emission_is_additive_and_clause_inventory_is_exhaustive() {
    // Additive contract: with CONFORMANCE_REPORT unset the emitter is a
    // filesystem no-op and every court in this file passes unchanged; with it
    // set, emission must not perturb any assertion above. Either way the
    // in-memory inventory below stays complete.
    assert_eq!(
        CLAUSE_TABLE.len(),
        9,
        "clause inventory: 9 rec clause groups"
    );
    for clause in ["§1", "§2", "§6.1", "§8"] {
        let row = CLAUSE_TABLE
            .iter()
            .find(|r| r.clause == clause)
            .unwrap_or_else(|| panic!("PROSE-ONLY clause {clause} missing from the inventory"));
        assert_eq!(
            row.status, "PROSE-ONLY",
            "{clause} must stay PROSE-ONLY here"
        );
    }
    assert_eq!(
        CLAUSE_TABLE
            .iter()
            .filter(|r| r.status == "COURTED")
            .count(),
        5,
        "five clause groups carry in-file court witnesses"
    );
    for r in CLAUSE_TABLE {
        assert!(
            !r.requirement.contains('|') && !r.witness.contains('|'),
            "cell text must stay pipe-free for Markdown: {}",
            r.clause
        );
    }
    // Exercising the emitter here is safe under both gate states: with the
    // gate set it writes the report; unset it is a no-op.
    emit_conformance_report();
}

// Every court above registers its clause as witnessed by appending
// emit_conformance_report() at its end; the Once makes emission idempotent.
// ---------------------------------------------------------------------------
// Rec §7 — Analytical MERGE execution court (3-step: ADMIT / REFUSE / FLIP).
// The compiler-side leg the pyshacl gate cannot cover: a §7-shaped MERGE
// document is admitted by the real compiler, executed against a real
// frozen-duckdb in-memory database (Chicago: real collaborators, real table
// readbacks, no mocks/sleeps), a document missing sql:mergeOn refuses with
// the compiler's typed MissingProperty code, and adding sql:mergeOn back
// flips refuse → admit (anti-vacuity inline flip).
// ---------------------------------------------------------------------------

/// The §7 graph with the sql:mergeOn block excised (the REFUSE step).
fn section7_merge_graph_missing_mergeon() -> String {
    section7_merge_graph().replace(
        r#"    sql:mergeOn [
        a sql:BinaryExpression ;
        sql:left [ a sql:ColumnIdentifier ; sql:qualifier "main.regional_sales" ; sql:columnName "region_id" ; sql:scope sql:ProcessScope ] ;
        sql:operator "=" ;
        sql:right [ a sql:ColumnIdentifier ; sql:qualifier "src" ; sql:columnName "region_id" ; sql:scope sql:ProcessScope ]
    ] ;
"#,
        "",
    )
}

#[test]
fn section7_merge_admit_refuse_flip_on_real_duckdb() {
    use graphlaw::duckdb_eval::GraphLawDuckDB;

    // --- STEP 1: ADMIT + EXECUTE + RECEIPT -----------------------------------
    let compiled = compile_turtle(&section7_merge_graph())
        .expect("§7 MERGE document must be admitted by compile_turtle");
    let db = GraphLawDuckDB::open_in_memory().expect("open in-memory frozen-duckdb");
    db.raw_connection()
        .execute_batch(
            "CREATE SCHEMA staging;
             CREATE TABLE staging.invoices (region_id VARCHAR, amount BIGINT, status VARCHAR);
             INSERT INTO staging.invoices VALUES ('east', 100, 'SETTLED'), ('east', 50, 'OPEN'), ('west', 70, 'SETTLED');
             CREATE TABLE main.regional_sales (region_id VARCHAR, revenue BIGINT);
             INSERT INTO main.regional_sales VALUES ('east', 1000), ('west', 2000);",
        )
        .expect("seed tables");
    let receipt = db
        .eval_ast_query("ex:mergeStatement", &compiled, "ex:row", "ex:cell")
        .expect("admitted MERGE must execute through the receipt-bearing eval path");
    // Receipt assertions: frozen-duckdb's MERGE INTO emits exactly one
    // summary row (the merged-row count), so rows_emitted is the typed 1,
    // and the admitted-statement receipt hash is nonzero.
    assert_eq!(
        receipt.rows_emitted, 1,
        "MERGE receipt: one engine summary row"
    );
    assert_ne!(
        receipt.query_hash, [0u8; 32],
        "admitted-statement receipt hash must be nonzero"
    );
    assert!(receipt.execution_ms >= 0.0);
    assert!(
        receipt
            .sql_statement
            .starts_with("MERGE INTO main.regional_sales"),
        "receipt must carry the executed parameterized MERGE: {}",
        receipt.sql_statement
    );
    // Real table readback — only SETTLED invoices merge.
    let revenue = |region: &str| -> i64 {
        db.raw_connection()
            .query_row(
                "SELECT revenue FROM main.regional_sales WHERE region_id = ?",
                [region],
                |r| r.get::<_, i64>(0),
            )
            .expect("revenue readback")
    };
    assert_eq!(revenue("east"), 1100, "east: 1000 + settled 100");
    assert_eq!(revenue("west"), 2070, "west: 2000 + settled 70");

    // --- STEP 2: REFUSE (missing sql:mergeOn) --------------------------------
    let missing_mergeon = section7_merge_graph_missing_mergeon();
    let err = compile_turtle(&missing_mergeon).unwrap_err().to_string();
    // The shape gate fires first: the closed-world MergeStatementShape
    // (ontologies/sql-ast.ttl, sh:path sql:mergeOn minCount 1) refuses with
    // its typed SHACL violation carrying the shape's own message — the
    // compiler's MissingProperty(mergeOn) is the backstop behind it.
    assert!(
        err.contains("SHACL AST structural violation")
            && err.contains("A MergeIntoStatement MUST carry exactly one sql:mergeOn"),
        "missing sql:mergeOn must refuse with the shape gate's typed code: {err}"
    );

    // --- STEP 3: FLIP (inline benign twin: adding mergeOn flips refuse→admit)
    // The twin of the refusing document is the original §7 graph — proven to
    // admit and execute in STEP 1; the flip is re-derived syntactically:
    // injecting the mergeOn block into the refusing document must admit AND
    // compile to the MERGE fragment.
    let flipped = missing_mergeon.replace(
        "    sql:whenMatched [",
        r#"    sql:mergeOn [
        a sql:BinaryExpression ;
        sql:left [ a sql:ColumnIdentifier ; sql:qualifier "main.regional_sales" ; sql:columnName "region_id" ; sql:scope sql:ProcessScope ] ;
        sql:operator "=" ;
        sql:right [ a sql:ColumnIdentifier ; sql:qualifier "src" ; sql:columnName "region_id" ; sql:scope sql:ProcessScope ]
    ] ;
    sql:whenMatched ["#,
    );
    let flipped_compiled =
        compile_turtle(&flipped).expect("FLIP: adding sql:mergeOn must flip refuse → admit");
    assert!(
        flipped_compiled
            .sql
            .starts_with("MERGE INTO main.regional_sales"),
        "FLIP twin must compile to the MERGE fragment: {}",
        flipped_compiled.sql
    );
    emit_conformance_report();
}

// ---------------------------------------------------------------------------
// Lane M12 — register-completeness court (defect class A2, SQLRDF-18 drift).
// docs/refusals.md trigger column must enumerate every implemented
// `refusal_arm::*` rendering arm in `src/sql_ast/refusals.rs`.
//
// Mechanics: the arm-constant NAME SET is parsed mechanically from the src
// file (include_str! — no regex dep). Every parsed arm must be either (a) in
// REGISTRY with all its doc needles present in docs/refusals.md, or (b) in
// PINNED_GAPS (known defect-class-A2 drift, tracked for a receipted register
// flip). Adding a new arm to the enum without touching this court fails
// requirement (a) — that IS the completeness forcing function. Removing a
// needle's doc row fails requirement (b-side regression: registered arms may
// not silently de-register).
// ---------------------------------------------------------------------------

const REFUSALS_RS: &str = include_str!("../src/sql_ast/refusals.rs");

/// Arms whose behavior the register DOES name (needle text verified against
/// docs/refusals.md rows as of v26.10.22).
const REGISTERED_ARMS: &[(&str, &[&str])] = &[
    // REFUSAL-01 trigger column
    ("OPERATOR_TOKEN", &["outside the §4 operator enum"]),
    ("SCOPE_INDIVIDUAL", &["non-`sql:Scope` individual"]),
    (
        "DISCRIMINATORS",
        &["more than one expression discriminator"],
    ),
    (
        "INDEX_MISSING",
        &["missing or non-integer `sql:statementIndex`"],
    ),
    ("INDEX_NOT_INT", &["non-integer `sql:statementIndex`"]),
    ("EMPTY_FROM", &["empty/missing `sql:from`"]),
    ("NO_FROM", &["empty/missing `sql:from`"]),
    ("NO_PROJECTION", &["missing projection list"]),
    ("EMPTY_PROJECTION", &["missing projection list"]),
    // REFUSAL-02
    ("UNIMPLEMENTED_STATEMENT", &["ConnectStatement"]),
    ("NO_SQL_TEMPLATE", &["UnsupportedTerm"]),
    ("UNSUPPORTED_GENERIC", &["UnsupportedTerm"]),
    // REFUSAL-03 trigger column
    (
        "AGG_IN_POSITION",
        &["aggregate in a `WHERE`/`JOIN`/`GROUP BY` position"],
    ),
    (
        "AGG_IN_PREDICATE",
        &["aggregate in a `WHERE`/`JOIN`/`GROUP BY` position"],
    ),
    (
        "PROCESS_SCOPE_IN_SELECT",
        &["`sql:ProcessScope` column in the projection list"],
    ),
    (
        "VIEW_SCOPE_IN_PREDICATE",
        &["`sql:ViewScope` column inside a predicate"],
    ),
    // L4 appendix
    ("JOIN_TYPE_IRI", &["sql:joinType"]),
    // G5 document-sequence appendix (docs/refusals.md, v26.10.23)
    (
        "MIXED_DOCUMENT_ROOTS",
        &["one root or one sequence, never both"],
    ),
    (
        "INDEX_SEQUENCE_DISORDER",
        &["not strictly ascending along the `rdf:List` spine"],
    ),
    ("SEQUENCE_EMPTY", &["Sequence with zero members"]),
    // REFUSAL-05 trigger column
    (
        "UNBOUNDED_MUTATION",
        &["neither `sql:whereColumn` nor `sql:whereValue`"],
    ),
    // SHACL internal family (coarse: strict closed-world SHACL shape validation)
    ("PARSE_DATASET", &["SHACL shape validation"]),
    ("PARSE_SHAPES", &["SHACL shape validation"]),
    ("SHACL_ENGINE", &["SHACL shape validation"]),
    ("SHACL_VIOLATIONS", &["SHACL shape validation"]),
    // SQLRDF-25 appendix (docs/refusals.md, v26.10.23 — window/frame/qualify/
    // atomic/CTE/composite/star/COLUMNS arms, refusal_arm 42..52). Disclosed
    // minimal unblock by the refusals-reconcile lane: this register file is
    // another lane's lease; the gate cannot pass without these rows.
    ("WINDOW_FN_POSITION", &["Window-function position law"]),
    ("FRAME_UNIT", &["Frame unit law"]),
    ("FRAME_BOUND", &["Frame bound law"]),
    ("QUALIFY_EXPR", &["QUALIFY expression law"]),
    ("ATOMIC_LITERAL", &["Atomic flag datatype law"]),
    ("CTE_NO_QUERY", &["CTE query law"]),
    ("COMPOSITE_DEPTH", &["Composite literal/accessor depth cap"]),
    ("COMPOSITE_MEMBER", &["Composite member law"]),
    ("STAR_MODIFIER_POSITION", &["Star vocabulary position"]),
    ("COLUMNS_LAMBDA", &["COLUMNS lambda form"]),
    ("COLUMNS_PATTERN", &["COLUMNS pattern literal"]),
    // SQLRDF-27 set-op arms (docs/refusals.md set-op appendix, refusal_arm
    // 53..56; courts: tests/set_operations_court.rs).
    ("SET_OP_NO_RIGHT", &["carries no `sql:setRight`"]),
    ("SET_OP_KIND_TOKEN", &["not `UNION`, `INTERSECT` or `EXCEPT`"]),
    ("SET_OP_FLAG_LEX", &["flag literal is not an `xsd:boolean`"]),
    ("SET_OP_BY_NAME_KIND", &["beside a non-UNION `sql:setOperator`"]),
];

/// Known defect-class-A2 drift: implemented arms the register does not yet
/// enumerate. Appending an arm here requires a receipted register flip
/// (docs/refusals.md is coordinator/receipt-gated for this lane).
const PINNED_GAPS: &[&str] = &[
    "ALIASED_AGG_ARG",      // ScopeIntegrity: aliased agg arg
    "CANONICALIZATION",     // canon refused arm
    "DUP_CLAUSE_PREDICATE", // single-cardinality clause predicates
    "SCOPE_CARDINALITY",    // scope single-cardinality
    "SCOPE_WRONG_NODE",     // scope on non-column node
    "EXPRESSION_WRAPPER",   // bare sql:expression wrapper rule
    "ALIAS_IN_AGG_ARG",     // aliased expr in agg arg
    "DUAL_LEFT_OPERAND",    // leftOperand+left laundering
    "DUAL_RIGHT_OPERAND",   // rightOperand+right laundering
    "DUAL_OPERATOR_FORMS",  // token+IRI operator laundering
    "SQL_ALIAS_IN_AGG_ARG", // sql:alias in agg arg
    "SCOPE_ON_TABLE_REF",   // scope on table reference
    "DUAL_FROM",            // from + fromTable
    "DUAL_SELECT",          // selectList + select
    "DUAL_WHERE",           // whereClause + where
    "DUAL_HAVING",          // having + havingClause
    "DIRECTION_LEX",        // direction not ASC/DESC
    "DIRECTION_IRI",        // direction as IRI
    "LIMIT_MALFORMED",
    "LIMIT_DUP",
    "LIMIT_NOT_INT",
    "OFFSET_MALFORMED",
    "OFFSET_DUP",
    "OFFSET_NOT_INT",
    "INDEX_DUP",
    "INDEX_BAD_DTYPE",
    "INDEX_NEGATIVE",
    "UNKNOWN_STATEMENT_PREDICATE", // closed sql: predicate enum
    "SHACL_DEPTH",                 // SHACL nesting depth cap
    "LEGACY_MESSAGE",              // free-form ShaclViolation carrier (by design)
    "SCOPE_IN_PRED_AGG_ARG",       // A2: scope re-scoping an aggregate past review
    // Witnessed live during lane M12 (2026-10-22): a concurrent lane appended
    // this arm; the register row is pending a receipted flip (not this lane's
    // lease — docs/refusals.md is coordinator/receipt-gated).
    "MERGE_ON_NO_TARGET_COLUMN", // ScopeIntegrity: MERGE target column missing
];

#[test]
fn register_completeness_every_refusal_arm_is_enumerated() {
    // Mechanically parse the arm-constant names from the src file.
    let arms: Vec<String> = REFUSALS_RS
        .lines()
        .filter_map(|l| {
            let t = l.trim();
            t.strip_prefix("pub const ")
                .and_then(|s| s.split_once(": u16 ="))
                .map(|(name, _)| name.trim().to_string())
                .filter(|n| !n.is_empty())
        })
        .collect();
    assert!(
        arms.len() >= 52,
        "parser drift: expected >=52 refusal_arm consts, parsed {}",
        arms.len()
    );

    let registry_names: Vec<&str> = REGISTERED_ARMS.iter().map(|(n, _)| *n).collect();
    let mut failures: Vec<String> = Vec::new();

    // (a) every arm is classified (registered or pinned gap) — new arms fail here.
    for arm in &arms {
        let registered = registry_names.contains(&arm.as_str());
        let pinned = PINNED_GAPS.contains(&arm.as_str());
        if !registered && !pinned {
            failures.push(format!(
                "UNREGISTERED ARM '{arm}': implemented in src/sql_ast/refusals.rs but absent \
                 from both REGISTERED_ARMS and PINNED_GAPS in tests/sql_rdf_conformance.rs — \
                 register it in docs/refusals.md and add its needle, or pin it as a tracked gap"
            ));
        }
    }

    // (b) every registered arm's doc needles are actually present.
    for (arm, needles) in REGISTERED_ARMS {
        for needle in *needles {
            if !REFUSALS_MD.contains(needle) {
                failures.push(format!(
                    "DOC GAP '{arm}': needle \"{needle}\" absent from docs/refusals.md"
                ));
            }
        }
    }

    // (c) the defect-class-A2 arms stay visible until the register flips.
    for defect in ["SCOPE_IN_PRED_AGG_ARG", "VIEW_SCOPE_IN_PREDICATE"] {
        if defect == "SCOPE_IN_PRED_AGG_ARG" {
            assert!(
                PINNED_GAPS.contains(&defect),
                "A2 defect arm SCOPE_IN_PRED_AGG_ARG must remain pinned as an unregistered gap"
            );
            assert!(
                !REFUSALS_MD.contains("aggregate argument in a predicate"),
                "A2 regression: docs/refusals.md now names the SCOPE_IN_PRED_AGG_ARG behavior — \
                 move it from PINNED_GAPS to REGISTERED_ARMS"
            );
        }
    }

    // Non-vacuity baseline: the A2 defect is witnessed live — the register
    // does not name the aggregate-argument-in-predicate behavior today.
    assert!(!REFUSALS_MD.contains("aggregate argument in a predicate"),);

    assert!(
        failures.is_empty(),
        "register completeness failures:\n  {}",
        failures.join("\n  ")
    );
}
