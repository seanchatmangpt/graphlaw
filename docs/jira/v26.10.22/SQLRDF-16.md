# SQLRDF-16 — Moonshot backlog: the 1000× ERRC horizon

- **Id:** SQLRDF-16
- **Title:** Moonshot backlog — the 1000× ERRC horizon (thesis Part II)
- **Status:** Open (follow-up family, not part of the v26.10.22 acceptance set)
- **Standing:** **BLOCKED-BY: v26.10.22 integration commit** — no moonshot row
  starts before the v26.10.22 sql-rdf wave is integrated and its tickets/courts
  are flipped by the coordinator.
- **Thesis:** [THESIS.md](THESIS.md) ("State of SQL-RDF and the 1000× ERRC
  Horizon"). Body landed 2026-10-22 (reconstructed from the operator's
  dispatch, provenance-marked in the file); the row table below is the
  operative content, corrected per the skeptic review noted in the Rows
  section.

## Scope

Eight moonshots forming the follow-up family beyond v26.10.22 acceptance
(SQLRDF-01..SQLRDF-12). This ticket registers the backlog; each row becomes its
own ticket in a later window, unblocked only by the v26.10.22 integration
commit.

## Rows

> Corrected 2026-10-22 per the skeptic-lane review: the initial filing
> garbled row-name→falsifier assignments and two falsifiers were vacuous
> as filed. The table below carries the dispatch name-set with
> skeptic-hardened falsifiers.

| Row | Moonshot | Falsifier (a real observation that kills the moonshot claim) | Depends on |
| --- | --- | --- | --- |
| E1 | Eliminate the alias window — single normative names (`sql:select`/`from`/`where`/`having`, `left`/`right`, §4 operator strings only) | After the migration, a laundering document (benign normative + hostile alias, per `tests/sql_rdf_laundering_corpus.rs`) still admits — i.e. removing the second name did NOT make the escape class unrepresentable; or an external consumer is found that the window was admitting (grep receipt cited) | v26.10.22 integration commit |
| A1 | Scope from annotation to type — validation-scoped RDFS closure (SQLRDF-13) + per-position scope shapes; REFUSAL-03 becomes a category the parser cannot construct | With the closure live, a scope-crossing document still requires a compiler-side check to refuse (the shapes alone admit it) — scope-as-type did not subsume REFUSAL-03 | E1 |
| E2 | Generate the compiler surface from the shapes (ggen pipeline: shapes + reader-hint annotations → reader/emission module) | Deleting a property from the shapes does not change the generated reader or fail a coverage court (drift survives); or the annotation layer needed is as large as the hand-written code it replaces (measured, not asserted) | A1 |
| E3 | Executable conformance — rec MUSTs as assertions; `cargo test --test sql_rdf_conformance` IS the conformance report | A clause is changed in the rec text and no court flips (courts are not pinned to the filed text); or the coverage is only the already-courted subset and "green = the document" is true by construction of the extraction (vacuous-coverage check built into the rung) | E2. Legal posture: courts assert repo behavior, NOT the rec's W3C status claims — the disclaimed-text boundary is preserved |
| R1 | Reduce receipt cost — canonicalize once at admission, carry canon bytes on `CompiledQuery` | Canon-same receipts are NOT cheaper than the current parse-per-receipt path, measured end-to-end (benchmark receipt required; no profiling exists in-tree today) | v26.10.22 integration commit (implementation in flight this window) |
| C1 | Content-addressed eval cache — H_query as the cache key | Two identical graphs on different connections fork the store, or one flipped literal collides — the key is not injective; OR a cached result diverges from a fresh execution after ATTACH/schema mutation (staleness boundary must be typed, not assumed — ATTACH/session state is NOT in H_query) | R1 + **E1** (skeptic finding: alias-variant canonicalization means legacy vs normative forms of one statement hash differently today — the key is not stable across vocabulary until E1) |
| R2 | Capability route table — `capabilities()` ABI op; admission routes by registered engine, not build flag | The route table cannot express a real case the cfg-split handles (table-driven court diverges from actual admission), or the wasm row is unfalsifiable because wasm has no receipts (skeptic: as filed this falsifier was vacuous — wasm must gain at least the typed-refusal receipt surface for the row to be testable) | v26.10.22 integration commit |
| C2 | SQL-RDF as the only wire format — zero raw-SQL admissions from any external entry point | Baseline is already positive today: `src/duckdb_op.rs:203-217` accepts `ast.text` raw SQL on the wire (the C2 hole, filed as SQLRDF-17). The row's kill condition is: after closing P1 + gating `raw_connection`, the census court (static allowlist grep + dynamic typed-refusal probe on every ABI op) passes while a hand-crafted raw-SQL payload still executes — i.e. the boundary cannot be closed even with the census green | R2 + SQLRDF-17 |

## Dependency order

```
E1 → A1 → E2 → E3     (one vocabulary, one closure regime, one generator, then executable text)
R1 → C1               (cheap canon first, then the cache keyed on it; E1 → C1 edge added per skeptic)
R2 → C2               (route table formalizes the boundary, then the boundary closes)
```

All chains start **BLOCKED-BY: v26.10.22 integration commit**.

## Acceptance

This is a backlog registration ticket. Its own acceptance is:

- Real run: `test -f docs/jira/v26.10.22/SQLRDF-16.md` and
  `test -f docs/jira/v26.10.22/THESIS.md`, plus the INDEX.md row below
  (appended by this filing).
- Each row's falsifier becomes a runnable court when its row is spun into a
  ticket in a later window; a row with no executable falsifier at spin-out is
  FAIL for that spin-out.
- Anti-vacuity inherits the window law: `bash scripts/mutation_court.sh` must
  flip any court built from these rows.
