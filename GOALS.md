# GraphLaw production goals

## Invariants

1. **Zero duplicated standards semantics.** RDF, SPARQL, SHACL, ShEx, Datalog, OWL/RDFS and N3 executable semantics belong to upstream libraries.
2. **Fail closed.** There is no legacy parser/reasoner fallback. Unsupported features are explicit refusals.
3. **Pinned authority.** Direct semantic dependencies are pinned to an exact release or Git revision.
4. **One law-state vocabulary.** GraphLaw-specific ontologies, packs and queries may compose upstream engines; they must not fork their semantics.
5. **Upstream before local.** Missing standards functionality is implemented in the owning library or remains unsupported.
6. **Executable verification.** CI proves native parsing/query/reasoning/validation plus a wasm32 library build.

## Authority map

- PurRDF 3.0.1 (git-sourced: [seanchatmangpt/purrdf](https://github.com/seanchatmangpt/purrdf), rev-pinned): RDF 1.2, SPARQL 1.1/1.2, SHACL, ShEx 2.1, Datalog, RDF/RDFS/OWL-RL entailment, RDF events.
- purrdf::n3 3.0.1 (purrdf crates/n3 in the [seanchatmangpt/purrdf](https://github.com/seanchatmangpt/purrdf) fork): Notation3 parsing, reasoning, built-ins, resource limits and proofs.
- DuckDB 2.0 (`frozen-duckdb 2.0.0-dev`): Vectorized columnar execution, Arrow C ABI integration, Parquet scanning, zone-map filtering, SIMD aggregations.

## Non-goals

- Maintaining a hidden fallback of the retired historical standards engine.
- Preserving internal implementation APIs whose purpose was to implement a standard now owned upstream.
- Recreating DRed/RSP/window/query algorithms locally when no production-grade admitted library has standing. Such capabilities stay absent or external until an authority is admitted.
- Building a relational execution engine or SQL parser inside GraphLaw. Tabular storage, columnar pushdown, and Parquet scans belong to `frozen-duckdb`.

GraphLaw should grow primarily by adding semantic assets, admission policy, provenance/receipt composition, and adapters that do not duplicate upstream semantics.
