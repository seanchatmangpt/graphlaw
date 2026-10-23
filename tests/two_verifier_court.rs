//! WO-08 — two-verifier `H_query` agreement court (TC-MEGA-FIBO-01, serves M6).
//!
//! Two independent verifier instances agree byte-for-byte on
//! `H_query = Fold(h_ast, P_params)` (sql-rdf Rec. section 1.4 first formula,
//! two-node form):
//!
//! * **Verifier A** compiles a real document via `compile_turtle`, executes it
//!   on a real frozen-duckdb instance, and extracts `H_query` from the
//!   `ExecutionReceipt`.
//! * **Verifier B** is an INDEPENDENT `GraphLawDuckDB` instance (separate
//!   connection, separate prepared-statement and C1 caches) which re-derives
//!   `h_ast = BLAKE3(Canon(G_AST))` from the carried canon bytes alone and
//!   recomputes `H_query` with a FRESH `blake3::Hasher` per fold
//!   (`fold_params_anchored`) — it never reads A's hasher, receipt, or cache.
//! * **Cross-check**: both digests must equal the reference formula
//!   `admitted_statement_hash(Canon(G_AST), P_params)` (the same formula the
//!   in-file bit-identity tests in `src/duckdb_eval.rs` pin), and B's own
//!   eval receipt must also carry it.
//! * **Negative twin (falsifier)**: mutate ONE bound parameter between A and
//!   B → the digests DIVERGE. Non-repudiation at this layer means divergence
//!   is DETECTABLE: no party can substitute a different parameter set without
//!   the receipt hash changing and both verifiers disagreeing.
//!
//! ## Honest scope boundary (determinism, not adversarial non-repudiation)
//!
//! This court witnesses DETERMINISM and REPLAY agreement between two
//! verifier instances in one process. It does NOT witness non-repudiation
//! against an adversarial party: there are no signatures, no network, no
//! adversary. The Σ_PQC / Ed25519 signature layers (`sign_bytes` /
//! `sign_receipt` over the receipt payload, `src/attest.rs`) are SQLRDF-13
//! follow-ups and are deliberately out of scope here. The negative twin
//! proves the two-node hash is a real function of `P_params` (a mutated
//! param is detectable), not that a signer cannot lie.

use graphlaw::duckdb_eval::{GraphLawDuckDB, admitted_statement_hash, fold_params_anchored};
use graphlaw::sql_ast::{SqlLiteral, compile_turtle};

/// The §4 SELECT document (rec §4, `docs/reference/sql-rdf-rec.md`), fixed
/// bytes on every call — one bound string parameter.
fn select_document() -> String {
    r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex: <http://example.org/queries#> .
ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "telemetry_events" ] ;
    sql:selectList ( [ sql:columnName "batch_id" ] ) ;
    sql:whereClause [
        sql:operator "="^^<http://www.w3.org/2001/XMLSchema#string> ;
        sql:leftOperand [ sql:columnName "batch_id" ] ;
        sql:rightOperand [ sql:literalValue "beta" ]
    ] .
"#
    .to_string()
}

fn telemetry_ddl() -> &'static str {
    "CREATE TABLE telemetry_events (batch_id VARCHAR, latency_ms INTEGER);
     INSERT INTO telemetry_events VALUES ('alpha', 480), ('beta', 10), ('gamma', 900);"
}

/// An independent verifier instance: fresh in-memory frozen-duckdb with the
/// same DDL applied — no shared connection, cache, or hasher with any other
/// verifier.
fn verifier_instance() -> GraphLawDuckDB {
    let db = GraphLawDuckDB::open_in_memory().expect("in-memory duckdb must open");
    db.raw_connection()
        .execute_batch(telemetry_ddl())
        .expect("DDL must apply");
    db
}

/// WO-08 leg 1: verifier A and verifier B agree byte-for-byte on H_query.
#[test]
fn wo08_two_verifiers_agree_on_h_query() {
    let doc = select_document();

    // --- Verifier A: real compile + real eval, H_query from the receipt. ---
    let compiled_a = compile_turtle(&doc).expect("document must compile");
    let db_a = verifier_instance();
    let receipt_a = db_a
        .eval_ast_query(
            "http://example.org/queries#Q",
            &compiled_a,
            "http://example.org/data",
            "http://example.org/vocab",
        )
        .expect("verifier A eval must succeed");
    let h_a = receipt_a.query_hash;
    assert_ne!(h_a, [0u8; 32], "verifier A H_query must be non-trivial");
    assert!(
        receipt_a.rows_emitted >= 1,
        "verifier A must execute real rows, got {}",
        receipt_a.rows_emitted
    );

    // --- Verifier B: independent instance, recompute from canon+params
    // alone with a FRESH hasher; A's hasher/cache/receipt are not consulted.
    let db_b = verifier_instance();
    let canon_b = compiled_a
        .canon_graph_bytes
        .clone()
        .expect("graph-backed compile carries canon bytes");
    // B re-derives h_ast from the canonical bytes themselves.
    let h_ast_b = *blake3::hash(&canon_b).as_bytes();
    let h_b = fold_params_anchored(&h_ast_b, &compiled_a.params);
    assert_eq!(
        h_a, h_b,
        "verifier B's fresh-hash recompute must be byte-identical to A's receipt H_query"
    );

    // Reference-formula cross-check (the two-node formula's own in-file
    // bit-identity target): BLAKE3(Canon(G_AST) || P_params) directly.
    let h_ref = admitted_statement_hash(&canon_b, &compiled_a.params);
    assert_eq!(h_a, h_ref, "both digests must match the reference formula");

    // B also evaluates on its OWN instance and extracts H_query from its own
    // receipt — a second, fully independent receipt path must agree.
    let receipt_b = db_b
        .eval_ast_query(
            "http://example.org/queries#Q",
            &compiled_a,
            "http://example.org/data",
            "http://example.org/vocab",
        )
        .expect("verifier B eval must succeed");
    assert_eq!(
        h_a, receipt_b.query_hash,
        "verifier B's own receipt H_query must be byte-identical to A's"
    );

    // Print the agreed digest for the receipt (hex via attest helper).
    println!(
        "[WO08] H_query (A == B == reference) = {}",
        graphlaw::attest::hex_encode(&h_a)
    );
}

/// WO-08 leg 2 (falsifier): mutating ONE bound parameter between A and B
/// makes the digests DIVERGE on every comparison path.
#[test]
fn wo08_one_mutated_param_makes_digests_diverge() {
    let doc = select_document();

    // Verifier A: lawful compile + eval.
    let compiled_a = compile_turtle(&doc).expect("document must compile");
    let db_a = verifier_instance();
    let receipt_a = db_a
        .eval_ast_query(
            "http://example.org/queries#Q",
            &compiled_a,
            "http://example.org/data",
            "http://example.org/vocab",
        )
        .expect("verifier A eval must succeed");
    let h_a = receipt_a.query_hash;

    // One-param mutation between A and B: same admitted statement, different
    // bound value. Params are unsealed by design (binding different params to
    // the same admitted statement is lawful), so the mutation compiles the
    // seal intact — the DIVERGENCE is carried entirely by the two-node fold.
    let mut compiled_b = compiled_a.clone();
    let mutated = SqlLiteral::String("gamma".into());
    assert_ne!(
        compiled_b.params[0], mutated,
        "mutation must actually change the parameter"
    );
    compiled_b.params[0] = mutated;

    let db_b = verifier_instance();
    let receipt_b = db_b
        .eval_ast_query(
            "http://example.org/queries#Q",
            &compiled_b,
            "http://example.org/data",
            "http://example.org/vocab",
        )
        .expect("verifier B eval must succeed");

    assert_ne!(
        receipt_a.query_hash, receipt_b.query_hash,
        "receipt H_query must diverge on a one-param mutation"
    );

    // Fold-path divergence with a fresh hasher on B's side.
    let h_ast = *blake3::hash(
        compiled_b
            .canon_graph_bytes
            .as_deref()
            .expect("canon bytes present"),
    )
    .as_bytes();
    let h_b_fold = fold_params_anchored(&h_ast, &compiled_b.params);
    assert_ne!(
        h_a, h_b_fold,
        "fresh-hash fold must diverge from A on the mutated param"
    );

    // The reference formula agrees with B's mutated fold — divergence is in
    // the param, not in a verifier implementation bug.
    assert_eq!(
        h_b_fold,
        admitted_statement_hash(
            compiled_b.canon_graph_bytes.as_deref().unwrap(),
            &compiled_b.params
        ),
        "B's mutated fold must still match the reference formula"
    );

    println!(
        "[WO08] diverged: A = {}, B(mutated param) = {}",
        graphlaw::attest::hex_encode(&h_a),
        graphlaw::attest::hex_encode(&h_b_fold)
    );
}

/// WO-08 leg 3: formula-consistency cross-check — the shipped
/// `fold_params_anchored` two-node fold is bit-identical to the reference
/// `admitted_statement_hash` on BOTH param-buffer branches (inline <= 64B
/// encoded and larger-than-inline), pinning this court to the in-file
/// bit-identity tests in `src/duckdb_eval.rs`.
#[test]
fn wo08_formula_consistency_inline_and_large_param_branches() {
    let doc = select_document();
    let compiled = compile_turtle(&doc).expect("document must compile");
    let canon = compiled
        .canon_graph_bytes
        .clone()
        .expect("graph-backed compile carries canon bytes");
    let h_ast = *blake3::hash(&canon).as_bytes();

    // Inline branch: the document's own single short-string param.
    let h_inline = fold_params_anchored(&h_ast, &compiled.params);
    assert_eq!(
        h_inline,
        admitted_statement_hash(&canon, &compiled.params),
        "inline param branch: two-node fold must match the reference formula"
    );

    // Large branch: params whose encoding exceeds the 64-byte inline budget.
    let large_params = vec![SqlLiteral::String("x".repeat(200))];
    let h_large = fold_params_anchored(&h_ast, &large_params);
    assert_eq!(
        h_large,
        admitted_statement_hash(&canon, &large_params),
        "large param branch: two-node fold must match the reference formula"
    );
    assert_ne!(
        h_inline, h_large,
        "different param sets under the same h_ast must produce different H_query"
    );
}
