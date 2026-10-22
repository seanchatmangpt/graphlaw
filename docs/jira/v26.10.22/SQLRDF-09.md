# SQLRDF-09 — Admitted-statement receipt

- **Id:** SQLRDF-09
- **Title:** Admitted-statement receipt
- **Status:** Open

## Conformance target

The first formula in section 1.4 of the pasted sql-rdf Recommendation, the conformance text for v26.10.22:

`H_query = BLAKE3(Canon(G_AST) || P_params)`

`Canon(G_AST)` is the canonical byte form of the admitted AST graph. `P_params` is the parameter buffer. The digest is the admitted-statement receipt.

## Current behavior

`ExecutionReceipt.query_hash` in [`src/duckdb_eval.rs`](../../../src/duckdb_eval.rs) (lines 56–60) is documented as a BLAKE3 hash across the serialized SQL and its bound parameter vector. `eval_ast_query` (lines 111–137) updates a `blake3::Hasher` with `compiled.sql` bytes and then with a type tag plus each parameter’s bytes. The hash is stored on the receipt returned to the caller.

`state_receipt` in [`tests/chicago_validation.rs`](../../../tests/chicago_validation.rs) (lines 127–151) hashes the SQL text, a `0xff` separator, and the ordered table readback. `dml_receipts_bind_readback_and_distinguish_statements` (lines 154–168) treats that digest as the DML witness.

The crate canonicalizes RDF graphs with `purrdf::try_canonicalize` (RDFC-1.0), used from [`src/law.rs`](../../../src/law.rs). `Dialect::SqlRdf` does not use it: `media_type()` is `None`, so admission in [`src/tier.rs`](../../../src/tier.rs) `admit_sql_rdf` returns a `CompiledQuery` and no canonical graph bytes.

## Required behavior

The admitted-statement receipt exposed on the execution result is BLAKE3 of the canonical AST graph concatenated with the parameter buffer, the first formula in section 1.4.

The Chicago readback hash remains a separate state witness. It stays the witness courted by `dml_receipts_bind_readback_and_distinguish_statements`. It is a separate digest from `H_query`.

## Acceptance

`cargo test --test sql_rdf_dialect -- admitted_ast_executes_on_frozen_duckdb` and `cargo test --test chicago_validation -- dml_receipts_bind_readback_and_distinguish_statements`.
