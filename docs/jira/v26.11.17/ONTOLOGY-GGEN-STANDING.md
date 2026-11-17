# Ontology and ggen standing (2026-11-17)

Lane 13. Static inspection at dd6a80e. No files changed.

## 1. Size (ontologies/ = 88M; plan said 87M)
| Path | Size | Note |
|---|---|---|
| industry/research | 31M | doid.owl 28.4M |
| industry/healthcare | 28M | doid-disease-ontology.owl 28.4M |
| industry/financial | 18M | fibo-master 18M |
| schema-org | 4.9M | nt + rdf + ttl serializations |
| industry/energy | 1.8M | another schema.org copy (1.5M) |
| rest | ~3.5M | core, provenance, catalogs, shapes... |

doid.owl and doid-disease-ontology.owl are byte-identical (cmp): ~28M duplication. 453 files are git-tracked. Grep consumers before any dedupe (not done).

## 2. Authored vs vendored
Authored: ontologies/sql-ast.ttl, graphlaw-wasm.ttl, castle-capability-intake.ttl, biopharma-ocel.ttl, catalog.ttl; shapes/{ast_governance, process-conformance.shacl, sa2a-diataxis-shapes}.ttl; ggen/sql-dialect inputs; docs/DIALECTS.md.
Vendored (do not edit): ontologies/core/*, shapes/{shacl,shacl-shacl,shex}, industry/*, schema-org/*, foaf, dublin-core.

## 3. Projections (never hand-edit)
ggen/sql-dialect (mode Overwrite): sql_statement_kinds.rs, sql_extensions.rs, statement_classes.ttl. Regenerate with `ggen sync run` in ggen/sql-dialect/. `src/sql_statement_kinds.rs` is an authored wrapper that `include!`s them.
Unverified: ontologies/graphlaw-wasm.ttl renders wasm/src/{abi_meta.rs,ffi.rs} via a second pack.

## 4. Gates
scripts/shape_mutation_court.sh [M1|M2|all] (TC-MUT-01): exits 0 flipped, 3 vacuous, 4 restore failed, 5 not applied. M1 is E2 coverage only; M2 is the real flip class. Mutates in place; run serially.

## 5. Open items
Dedupe DOID and schema.org copies; confirm consumers; verify wasm generated headers; consider CI regenerate-and-diff for ggen.
