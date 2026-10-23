# Thesis — State of SQL-RDF and the 1000× ERRC Horizon

- **Filed:** 2026-10-22
- **Written against tree:** `/Users/sac/graphlaw` @ `ed3be811fc6136facf68755b6bf6bb1d5dd902ca`
  (`ed3be81`, "chore: extend wasm_boundary_fuzz to 1,100 iterations + refusal-class
  histogram (GWS-15 closure …)"), commit time 2026-10-21 20:01:29 -0700, with the
  v26.10.22 working tree dirty (65 modified/untracked paths — the in-flight
  v26.10.22 sql-rdf wave).
- **Follow-up backlog:** [SQLRDF-16](SQLRDF-16.md) (moonshot rows + falsifiers +
  dependency order).
- **Provenance:** the operator's verbatim thesis prose was not present in the
  dispatch payload; the coordinator holds it. The body below is **reconstructed
  from the operator's dispatch; wording per dispatch summary** — structure and
  content claims are the coordinator's authoritative summary, not quoted
  operator text. Where a sentence would otherwise read as invented prose, it is
  a summary-form rendering of the dispatch content.

## Part I — The achieved surface, and what the wave teaches

### 1.1 The achieved surface

The v26.10.22 wave landed a complete, receipted sql-rdf surface. Eight items
constitute it (per dispatch summary; receipts in [INDEX.md](INDEX.md) and
[RECEIPTS.md](RECEIPTS.md)):

1. **Normative vocabulary** — the sql-rdf surface is stated in the canonical
   graph (`ontologies/sql-ast.ttl`), not in code comments or prose; the
   Recommendation text (`docs/reference/sql-rdf-rec.md`) cites by section and
   its W3C status claims are document text, not facts this repo asserts.
2. **Closed-world SHACL admission** — graphs admit through the executable law
   (`src/law.rs`) against closed-world SHACL shapes; a shape not satisfied is a
   typed refusal, never a warning.
3. **Expression-level compiler** — `src/sql_ast.rs` compiles SQL-in-RDF to a
   typed AST, including expression-level structure, not just statement shells.
4. **Scope lattice** — the admission tiers (`src/tier.rs`) form a lattice;
   capability is scoped, not global.
5. **Typed refusal regime** — every refusal carries a stable code
   (`docs/refusals.md`, REFUSAL-01..); no generic DB errors or crashes where a
   typed refusal belongs.
6. **§1.4 receipt** — the receipt form specified in §1.4 of the Recommendation
   (BLAKE3(Canon(G_AST) || P_params) via `purrdf::try_canonicalize`) is
   implemented and courted.
7. **One wasm unit** — `graphlaw-wasm` links the same crate as the host,
   including frozen-duckdb. SQL-in-RDF admits through `compile_turtle` inside
   that module. `REFUSAL-04` is the non-document-backed builtin reader.
   frozen-duckdb 2.0.2 emits `static=duckdb` for wasm32-wasip1. The link is witnessed (2026-10-22); the recorded pin `332ea218…` was not replaced. `WASI_SDK_PATH` must be a wasi-sdk root (`bin/clang`, `share/cmake/wasi-sdk-p1.cmake`). Without it, frozen-duckdb-sys `build.rs` fails typed (`WASI_SDK_PATH required`).
8. **Courts over everything** — every claim above is backed by an execution
   court (`tests/chicago_validation.rs`, `tests/sql_rdf_dialect.rs`,
   `tests/duckdb_statement_kinds.rs`, `tests/trust_attest.rs`) plus the
   anti-vacuity sweep (`scripts/mutation_court.sh`).

**Honest residue.** The wave does not claim a clean surface. The E2–E5 escape
class remains; the `sh:class` closures were withheld; `StatementBaseShape`
remains documentary rather than load-bearing; statement classes are flat rather
than structured; VERIFIED-EXTERNAL parity is not achieved; and Σ_PQC is unmet.
These are filed, not hidden: [SQLRDF-13](SQLRDF-13.md),
[SQLRDF-14](SQLRDF-14.md), [SQLRDF-15](SQLRDF-15.md), and the L2 backlog
([SQLRDF-16](SQLRDF-16.md)).

### 1.2 What the wave teaches

Four lessons, per dispatch summary:

- **Correctness was produced by courts, not coordinator pre-thinking.** No
  defect was caught by the coordinator reasoning ahead of execution; every
  material defect was caught by a court run.
- **Every material defect was caught by an execution ladder** — the cheapest
  rung that answers the question, escalating on failure (`cargo check` →
  single court file → full suite → mutation court → wasm boundary).
- **The entropy-cascade law held**: unverified claims compound at generation
  rate; verification compounds at execution rate. The wave's verification rate
  exceeded its generation rate, so defects surfaced instead of compounding.
- **Any 1000× proposal that cannot be courted is prose.** Each Part II
  moonshot therefore carries a falsifier that a real command can flip.

## Part II — The ERRC 1000× phase-change moonshots

Eight moonshots, one per ERRC family, each with its one-line thesis
characterization (per dispatch summary; full rows + falsifiers in
[SQLRDF-16](SQLRDF-16.md)):

| Id | Moonshot | Thesis characterization |
| --- | --- | --- |
| E1 | Eliminate the alias window | Remove the window in which an alias binding can be observed inconsistent with the canon — the escape class closes by construction, not by check. |
| E2 | Shapes as single description; generate the compiler | The SHACL shapes become the one description of the surface; `src/sql_ast.rs` is generated from them, so the compiler cannot drift from the law. |
| E3 | Executable conformance | The Recommendation's conformance clauses execute as courts; the document and the test suite are one artifact. |
| R1 | Incremental canon | Canonicalization becomes incremental, so unchanged subgraphs do not pay re-canon cost on every admission. |
| R2 | Capability query | Capability becomes a query over the registry/scopes rather than a compile-time feature flag — ask the lattice what is admissible. |
| A1 | Scope as type | The admission scope lattice becomes a type-level fact; out-of-scope programs are unrepresentable, not rejected at runtime. |
| C1 | Standing-addressed cache | Admission results are addressed by the standing (digest) of their inputs; unchanged inputs verify instead of re-admit. |
| C2 | SQL-RDF as the only wire format | No raw-SQL admission path survives; the graph is the sole ingress, so the laundering corpus is refused by construction. |

Falsifier shorthand as carried (see [SQLRDF-16](SQLRDF-16.md) for the bound
rows): E1 — E-corpus refusing by construction; R1 — canon-same receipts
cheaper; A1 — cache hit on identical digests; C1 — conformance green = the
document; C2 — zero raw-SQL admissions. E2/E3/R2 falsifiers are bound in the
SQLRDF-16 rows (assignments not carried verbatim in the dispatch).

## Part III — Sequencing

Dependency order (per dispatch summary), each edge carrying its falsifier
inside it:

- **E1 → A1 → E2 → E3.** The alias window must close before scope can be a
  type (a scope over an open window is not a type); scope-as-type must hold
  before the compiler can be generated from shapes (the generated artifact
  needs the scope discipline to be enforceable); generated compiler before
  executable conformance (the conformance courts execute the generated
  artifact). Falsifier: any edge admitting while its predecessor's falsifier
  still flips.
- **R1 → C1.** Incremental canon first — a standing-addressed cache is only
  sound if canon-same inputs are cheaply identified; the cache falsifier
  (cache hit on identical digests) presupposes the incremental-canon falsifier
  (canon-same receipts cheaper).
- **R2 → C2.** Capability query first — SQL-RDF-as-only-wire-format is
  admissible only once capability is queryable, so "zero raw-SQL admissions"
  is checkable as a query result rather than an audit.

Standing: all moonshots **BLOCKED-BY: v26.10.22 integration commit**
([SQLRDF-16](SQLRDF-16.md)).
