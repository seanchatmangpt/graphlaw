# RECEIPTS — moonshot layer, v26.10.22

Manufacturing receipt for the moonshot layer (SQLRDF-16/SQLRDF-17 family) of
the v26.10.22 sql-rdf wave. Subject: `/Users/sac/graphlaw` @ `ed3be81` +
uncommitted working set (coordinator owns git). Companion to the window-level
[RECEIPTS.md](../RECEIPTS.md) (which carries the SQLRDF-01..12 verdicts);
this file carries the moonshot layer only. As-of: 2026-10-22, 12:52–12:58 PDT,
all commands re-executed by this lane (cwd `/Users/sac/graphlaw`).

## 1. Lanes dispatched (15 per the coordinator's wave ledger)

| # | Lane | Product | Standing |
| --- | --- | --- | --- |
| 1 | Thesis filing | [THESIS.md](../THESIS.md) skeleton (verbatim body PENDING — dispatch did not carry the prose) | LANDED (skeleton) |
| 2 | E1 spec | [E1-alias-elimination.md](E1-alias-elimination.md) | LANDED |
| 3 | E1 court | census court spec inside E1 design | LANDED |
| 4 | E1 census | alias-window census (grep receipt) | LANDED |
| 5 | E2 spec | [E2-shapes-reader.md](E2-shapes-reader.md) | LANDED |
| 6 | E3 spec | [E3-conformance.md](E3-conformance.md) | LANDED |
| 7 | C1 spec | [C1-cache.md](C1-cache.md) | LANDED |
| 8 | C2 spec | C2 inventory (folded into SQLRDF-17) | LANDED |
| 9 | A1 spec | [A1-scope-as-type.md](A1-scope-as-type.md) | LANDED |
| 10 | R2 spec | [R2-capability-routes.md](R2-capability-routes.md) | LANDED |
| 11 | R1 implementation + court | `CompiledQuery.canon_graph_bytes` + counter court + gates | LANDED (code) |
| 12 | E3 skeleton | [tests/sql_rdf_conformance.rs](../../../tests/sql_rdf_conformance.rs) rung skeleton | LANDED (4/5 green, 1 live flip failure — see §4) |
| 13 | Skeptic | [SKEPTIC-thesis-review.md](SKEPTIC-thesis-review.md) (row-name→falsifier corrections folded into SQLRDF-16) | LANDED |
| 14 | Moonshot backlog ticket | [SQLRDF-16.md](../SQLRDF-16.md) (8 rows, falsifiers, dependency order) | LANDED |
| 15 | C2 spin-out ticket | [SQLRDF-17.md](../SQLRDF-17 Current-behavior pointers re-verified on disk 12:52 PDT | LANDED |

Count reconciliation: the ledger says 15; the table itemizes 15 rows; the
dispatch prose named 12 dispatches (E1 spec+court+census counted as three).
Row 15's standing is as-reported by the dispatch; its Current-behavior
pointers were re-verified on disk by this lane (§4).

## 2. What LANDED

- **SQLRDF-16 corrected table** —
  [`docs/jira/v26.10.22/SQLRDF-16.md`](../SQLRDF-16.md), verified on disk
  12:43 mtime, read 12:52 PDT. Eight rows (E1/A1/E2/E3/R1/R2/A1/C1/C2 family)
  with skeptic-hardened falsifiers, dependency order
  `E1→A1→E2→E3`, `R1→C1 (+E1→C1)`, `R2→C2`, all BLOCKED-BY the v26.10.22
  integration commit. Corrected-per-skeptic banner present (SQLRDF-16.md:26-29).
- **SQLRDF-17 C2-hole ticket** —
  [`docs/jira/v26.10.22/SQLRDF-17.md`](../SQLRDF-17.md), verified on disk
  12:43 mtime, read 12:52 PDT: the C2 hole P1 with Current-behavior pointers
  at `src/duckdb_op.rs:198-219`, `src/duckdb_eval.rs:183-188` / `:155`,
  `src/dialect.rs:740-758`; pointer re-verification in §4 below.
- **THESIS skeleton with PENDING-VERBATIM banner** —
  [`docs/jira/v26.10.22/THESIS.md`](../THESIS.md): filed against `ed3be81`
  (65 dirty paths), verbatim-body status PENDING banner at lines 12-19, moonshot
  id/falsifier-shorthand table, Part III sequencing. No operator text quoted.
- **designs/ 8 files** — [INDEX.md](INDEX.md) + 7 design records
  (E1-alias-elimination, A1-scope-as-type, C1-cache, E2-shapes-reader,
  R2-capability-routes, E3-conformance, SKEPTIC-thesis-review), verified on
  disk 12:52 PDT.
- **Laundering corpus 11/11 with live E4 flip witnesses** — real run, this
  lane, 12:58 PDT:
  ```
  $ cargo test --test sql_rdf_laundering_corpus
  running 11 tests
  test benign_skeleton_compiles ... ok
  test e1_mixed_projection_select_item ... ok
  test e2_hostile_from_under_benign_fromtable ... ok
  test e3_duplicate_where_clause ... ok
  test e4_malformed_limit ... ok
  test e4_malformed_offset ... ok
  test e5_left_leftoperand_dual_key ... ok
  test process_scope_column_in_select_refuses_03 ... ok
  test view_scope_column_in_where_refuses_03 ... ok
  test dual_scope_on_column_refuses ... ok
  test scope_on_operator_node_refuses ... ok
  test result: ok. 11 passed; 0 failed; ... finished in 0.07s
  ```
  Anti-vacuity is inline-flip per court (`assert_flip_compiles`; receipts/sql-rdf-laundering-corpus-2026-10-22.md — re-executed 2026-10-22 by lane V-L17: current corpus 23/0, the 11/11 record above is as-of-date historical),
  tests/sql_rdf_laundering_corpus.rs:39); the E4 courts carry live flips
  ("e4-limit flip", "e4-offset flip", lines 163/174).
- **Conformance rung skeleton** — `tests/sql_rdf_conformance.rs` exists;
  real run, this lane, 12:58 PDT:
  ```
  $ cargo test --test sql_rdf_conformance
  test section4_shape_names_present_and_gates_refuse ... FAILED
  thread '...' panicked at tests/sql_rdf_conformance.rs:175:5:
  flip: closing the rdf:rest chain must admit the document
  test result: FAILED. 4 passed; 1 failed; ... finished in 0.10s
  ```
  Standing: **4/5 — the dispatch claim "5/5 skeleton" is NOT observed on the
  current tree.** The failing assertion is itself an anti-vacuity flip check;
  it aborts instead of flipping, so the §4 court's flip witness is currently
  red. This is a live defect in the skeleton (§4), not an environmental
  failure. (receipts/sql-rdf-conformance-2026-10-22.md — repaired same-day: 7/0 as of 2026-10-22; this 4/1 record is as-of-date historical)
- **R1 canon-once implementation complete, receipts bit-identical** —
  `CompiledQuery.canon_graph_bytes` landed
  (`src/duckdb_eval.rs:171-178`, carry-and-reuse at 372-374); canonical
  bytes computed once per compile with a thread-local counter witness
  (per window RECEIPTS.md, 12:57 PDT entry). Gates re-executed by this
  lane 12:58 PDT, all exit 0:
  ```
  $ cargo test --test chicago_validation   # 9 passed; 0 failed (2.30s)
  $ cargo test --test sql_rdf_dialect      # 25 passed; 0 failed (0.31s)
  $ cargo test --lib                       # 17 passed; 0 failed (0.06s)
  ```
  Receipts bit-identical across the canon-once change (receipts/chicago-validation-2026-10-22.md, receipts/sql-rdf-dialect-2026-10-22.md, receipts/kernel-lib-courts-2026-10-22.md — re-executed 2026-10-22 by lane V-L17; lib suite now 34/0; counter test +
  unchanged receipt hashes, per window RECEIPTS.md 12:57 entry; the two
  receipt-carrying courts above are green on the post-R1 tree).

## 3. DESIGNED-NOT-IMPLEMENTED (all BLOCKED-BY: v26.10.22 integration commit)

Per [SQLRDF-16](../SQLRDF-16.md) rows; designs exist, code does not:

- **E1 migration** — alias-window elimination. Design only
  ([E1-alias-elimination.md](E1-alias-elimination.md)); the alias window is
  still live in `ontologies/sql-ast.ttl` and the compiler.
- **A1 closure** — validation-scoped RDFS closure + positional scope shapes
  ([A1-scope-as-type.md](A1-scope-as-type.md)); currently the SQLRDF-13 plan
  only; `sh:class` entailment withheld at `ontologies/sql-ast.ttl:315-330`.
- **E2 generator** — shapes-driven reader generation
  ([E2-shapes-reader.md](E2-shapes-reader.md)); the reader is still
  hand-written `src/sql_ast.rs`.
- **C1 cache** — H_query content-addressed eval cache
  ([C1-cache.md](C1-cache.md)); no cache exists; additionally E1-gated per
  the skeptic (alias-variant canonicalization makes the key non-stable until
  E1).
- **C2 closure** — zero raw-SQL admissions; the hole is open and filed as
  SQLRDF-17 (§4).

## 4. Defects found by this layer

- **E4 `get_single_object` literal-blindness — the biggest kernel bug of the
  window.** Found by the laundering-corpus court (E4 cases): the reader's
  `get_single_object` (`src/sql_ast.rs:575`) does not surface literal nodes,
  so malformed *literal* clauses (`sql:limit "ten"^^xsd:integer`,
  negative `sql:offset`) previously laundered through as silent accepts.
  Closed by the corpus court + fixes; flips witnessed live ("e4-limit flip",
  "e4-offset flip", `tests/sql_rdf_laundering_corpus.rs:163,174`).
- **statementIndex gaps** — nested/duplicate/datatype gaps in the
  `sql:statementIndex` fixtures/courts; resolved via the `doc()` wrapper
  backfill (window RECEIPTS.md Blockers, resolved 12:30 PDT entry).
- **R2's `compile_turtle` admission bypass → SQLRDF-17.** The R2 spec lane
  found the ABI duckdb op admits raw SQL text
  (`src/duckdb_op.rs:198-219`), hashed via the legacy `(None, None)` arm
  (`src/duckdb_eval.rs:183-188`), with a sniff misroute (`src/dialect.rs:740-758`)
  and `pub raw_connection()` (`src/duckdb_eval.rs:155`). Filed as
  [SQLRDF-17](../SQLRDF-17.md); Current-behavior pointers re-verified by
  this lane on disk 12:52 PDT (grep receipts above: `canon_graph_bytes`
  match arm at `src/duckdb_eval.rs:178`, `get_single_object` at
  `src/sql_ast.rs:575`).
- **Conformance-skeleton flip failure (introduced by this layer's E3
  skeleton lane, observed live 12:58 PDT)** —
  `cargo test --test sql_rdf_conformance` → 4 passed / 1 failed,
  [receipts/sql-rdf-conformance-2026-10-22.md — re-run 2026-10-22: current standing 7 passed / 0 failed; the 4/1 flip-fixture failure was repaired same-day, this failure kept visible]
  `tests/sql_rdf_conformance.rs:175` panics "flip: closing the rdf:rest
  chain must admit the document". The court's own anti-vacuity flip aborts
  instead of flipping. Falsifier: fixing the flip fixture (or the §4 shape
  it exercises) must turn the rung 5/5 with the flip witnessing; until then
  the rung is PARTIAL, and the dispatch's "5/5 skeleton" claim is corrected
  here.

## 5. Receipt fields (operating-doctrine)

- **Identity**: tree `ed3be81` + working set; every claim above carries its
  gate command + as-of; commands run cwd `/Users/sac/graphlaw`, 12:52–12:58
  PDT 2026-10-22.
- **Authority**: docs lane; file ownership = this file only
  (`docs/jira/v26.10.22/designs/RECEIPTS.md`).
- **Consequence**: 15 lanes dispatched, 14 products landed (1 skeleton with a
  live red flip), 5 designed-not-implemented, 4 defects registered.
- **Replay**: rerun the four gate commands in §2; expected counts as printed.
  `sql_rdf_conformance` fails identically until the flip fixture is repaired.
- **Standing**: SQLRDF-16/17 tickets **Open**, BLOCKED-BY integration commit;
  layer artifacts CONFORMANT-ON-OBSERVED-TREE except the conformance rung
  (PARTIAL, live red flip at `tests/sql_rdf_conformance.rs:175`).
- **Falsifiers**: (a) repairing the conformance flip fixture must yield 5/5
  with a witnessed flip — if the failure persists after fixture repair, the
  defect is in the §4 court logic, not the fixture; (b) the corpus courts
  must fail if any E4-class literal laundering re-admits (inline flips are
  the witness); (c) THESIS.md verbatim-body landing must leave the PENDING
  banner removed and no operator text invented — presence of the banner
  after body landing is FAIL for that dispatch.
