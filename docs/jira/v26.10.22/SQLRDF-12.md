# SQLRDF-12 — File the conformance text

- **Id:** SQLRDF-12
- **Title:** File the conformance text
- **Status:** Open

## Conformance target

The pasted sql-rdf Recommendation is the conformance text for v26.10.22. Tickets SQLRDF-01 through SQLRDF-11 cite it by section. This ticket files that text at a stable path so those citations have a target in the tree.

## Current behavior

The Recommendation text is not in the repository. [`docs/reference/`](../../reference/) has no `sql-rdf-rec.md`. [`docs/jira/v26.10.22/INDEX.md`](INDEX.md) is the map of this ticket set and points at the compiler that exists today.

Admission of a `sql` document is implemented in [`src/tier.rs`](../../../src/tier.rs) `admit_sql_rdf` and courted by [`tests/sql_rdf_dialect.rs`](../../../tests/sql_rdf_dialect.rs). Filing a markdown document does not change that path.

## Required behavior

File the pasted Recommendation at [`docs/reference/sql-rdf-rec.md`](../../reference/sql-rdf-rec.md). The file’s status line says it is the conformance text for v26.10.22 and not a fetched W3C publication.

This ticket-writing pass does not create that file.

## Acceptance

Once the file is in place, `cargo test --test sql_rdf_dialect` stays green, because the file is documentation and admission is unchanged. The status line is present in `docs/reference/sql-rdf-rec.md`.
