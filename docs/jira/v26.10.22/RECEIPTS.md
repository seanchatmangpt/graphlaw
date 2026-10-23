# RECEIPTS — v26.10.22

One row per ticket: the acceptance gate command, the latest observed verdict
with its as-of timestamp, and the blocking observation where the verdict is
NOT-DONE. Verdicts are transcribed from the wave ledger as reported; ticket
Status fields stay **Open** until the coordinator flips them at integration.
Early-verdict note: the first gate rerun attempt (`cargo test --test
trust_attest`, 11:40 PDT) failed with `ENOSPC: no space left on device`; disk
was freed (99 GiB, `df -h` 12:20 PDT) and the Wave-receipt section carries
this lane's re-executed verdicts. Early rows below are as-reported, not
re-executed at that time.

| Id | Acceptance gate | Latest observed verdict | As-of (2026-10-22, PDT) | Observation |
| --- | --- | --- | --- | --- |
| SQLRDF-01 | `cargo test --test sql_rdf_dialect` | NOT-DONE (compiler-side) | 11:38 | ontology landed (L1); ontology-surface court green; 5 remaining reds are compiler-side (L2), not ontology gaps |
| SQLRDF-02 | `cargo test --test sql_rdf_dialect` | DONE (re-court pending) | 11:40 | DoD-02 court passed; re-court pending after L1's `xone` fixes land |
| SQLRDF-03 | `cargo test --test sql_rdf_dialect` | NOT-DONE | 11:38 | extraction-missing: clauses not extracted from the graph |
| SQLRDF-04 | `cargo test --test chicago_validation` | NOT-DONE | 11:38 | blocked-compile, then courts-present (courts exist, subject does not compile) |
| SQLRDF-05 | `cargo test --test duckdb_statement_kinds` | NOT-DONE | 11:38 | 0/29 classes on expression-level template paths |
| SQLRDF-06 | `cargo test --test sql_rdf_dialect` | NOT-DONE | 11:38 | carrier-mismatch: refusal exists without the stable REFUSAL-02 code path |
| SQLRDF-07 | `cargo test --test chicago_validation` | NOT-DONE | 11:38 | SHACL gate fires generically, not on the scoped shapes |
| SQLRDF-08 | wasm boundary build + `cargo test -p graphlaw-wasm` | PARTIAL | 11:40 | boundary build and code-path courts PASS; explicit refusal assertion unexecuted (test link failed with ENOSPC) |
| SQLRDF-09 | `cargo test --test chicago_validation` | PARTIAL | 11:38 | formula implemented in `src/duckdb_eval.rs` (BLAKE3 over canonical AST graph + typed params); gate courts fixture-blocked |
| SQLRDF-10 | `cargo test --test trust_attest` | DONE | 11:38 | 10 passed / 0 failed | (receipts/trust-attest-2026-10-22.md)
| SQLRDF-11 | `cargo test --test chicago_validation` | NOT-DONE | 11:38 | compile-abort: exit 1, 0 tests executed (not a failure count) |
| SQLRDF-12 | `test -f docs/reference/sql-rdf-rec.md` | DONE (gate pending) | 11:40 | file verified on disk; gate rerun pending tree-green (disk ENOSPC) |

## Blockers

Current state as-of 2026-10-22 12:57 PDT unless noted. Resolution record (each verified, not assumed): **ENOSPC** (freed 99 GiB 12:20, gates re-executed 12:20–12:35); **fixture `statementIndex` backfill** (`doc()` wrapper; chicago 9/0 12:30); **MERGE USING missing `SELECT`** (chicago 9/0 12:30); **select-list REFUSAL-03 gap** (`process_scope_column_in_select_list_is_refusal_03`, dialect 18/0 12:26); **escape-audit E1/E3/E4/E5 closed by L2's never-both + typed-parse + cardinality fixes** with courts, **E6 refuted** (not a defect), **E7 declared by L1** (12:45); **gws4 / wasm_duckdb_op stale test targets fixed** (12:45).

Still open:

- **generated_kinds `.gram` environmental failure** (12:30): `generated_kinds_match_the_peg_statement_rule` in duckdb_statement_kinds panics — `/Users/sac/frozen-duckdb/target/wasm-build/` absent on this host. The parse-parity surface it guards is VERIFIED-EXTERNAL 21/21 against the upstream snapshot. Loud, typed environmental failure; blocks nothing else. Falsifier: restore the frozen wasm-build checkout and rerun — must go green or yield a code-level diagnosis.
- **Frozen-pin checkout missing (SQLRDF-15)** (12:22): regeneration of the tpch hand-patch retiree BLOCKED on restoring the frozen-duckdb pinned sources; tpch sf=0/sf=0.01 adjudication UNKNOWN for the pin.
- **Anti-vacuity full sweep UNMET** (12:33; SPC/shape supplement 14:1x): `bash scripts/mutation_court.sh` aborts pre-mutation at `tests/test_cold_chain_ocel.rs:61` (`missing object_count in impl RdfAstGraphReader`), cargo-mutants exit 4, zero mutants tested. Per-court inversions remain the local anti-vacuity evidence. **Supplemental anti-vacuity courts landed post-abort**: (a) `scripts/spc_court.sh` landed with an **honest-FAIL receipt** — `receipts/spc-2026-10-22.json` + `-inject.json` (both n=1,000,000, tc_hot_01 clone_path, SMOKE): cpk −0.0018 / −0.0025 < 1.33, p99 375ns > 250ns USL, verdict FAIL both legs including the outlier-injected leg — the TC-HOT-01 perf budget is genuinely unmet, and the flip-to-FAIL is degenerate (both legs fail pre- and post-injection) until the perf closure; (b) `scripts/shape_mutation_court.sh` landed (mtime 13:40, usage `bash scripts/shape_mutation_court.sh [M1|M2|all]`): **M1-documented** (operator-enum removal is E2 coverage, not a flip court, per the script header at `scripts/shape_mutation_court.sh:11`), **M2-FLIP** (sh:minCount removal must flip the from-clause court to failure, exit 3 on non-flip). Falsifier: fixing that line must let the sweep reach mutant testing; a still-abort is a new defect.
- **In-flight at 12:57** (docs-lane refresh): laundering-corpus case e4 (`tests/sql_rdf_laundering_corpus.rs`) adjudication in flight; conformance anti-vacuity repair alongside it; `iri_object_count` trait sweep on external `RdfAstGraphReader` impls (same surface as the mutation-court abort); wasm artifact re-pin in flight (superseded 13:5x — re-pinned `332ea218…`, see Successor §2); final solo mutation_court run pending the above.
- **R1 canon-once — LANDED** (12:57, superseding the 12:45 in-flight entry): `CompiledQuery.canon_graph_bytes` computes canonical graph bytes once per compile (prior re-canonicalization witnessed at `tests/sql_rdf_dialect.rs:495-497`); +1-canonicalization witnessed by a thread-local counter test; receipts bit-identical. Gates: chicago 9/9, lib 17/17, dialect 25/25, trust 10/0. On-disk: `src/sql_ast.rs:367` (field), `:383` (doc), `:511` (initializer).
- **Moonshot layer** (12:45): SQLRDF-16/17 filed and verified on disk; designs persisted under `docs/jira/v26.10.22/designs/` (A1, C1, E1, E2, E3, R2, SKEPTIC-thesis-review); R2's compile_turtle-bypass finding folded into SQLRDF-17.

---

# Wave receipt (final)

As-of 2026-10-22 12:20–13:10 PDT, lane lane-docs (append-only to this file). Subject: tree @ `ed3be81` + uncommitted working set (coordinator owns git). This lane re-executed the gates after ENOSPC cleared; the early table above is retained unmodified. A concurrent `cargo-mutants` run was live on the shared tree (transient compile breaks at 13:04 in `src/abi.rs`/`src/sql_ast.rs`, self-restored by 13:06 — disclosed per the compile-freeze SLA, not absorbed into any PASS claim).

## Standing per ticket (FINAL — courts re-executed, 12:20–13:10 PDT)

| Id | Gate command (cwd /Users/sac/graphlaw) | Final verdict — court command, count, as-of (2026-10-22 PDT) |
| --- | --- | --- |
| SQLRDF-01 | `cargo test --test sql_rdf_dialect` | DONE — re-executed by this lane: 25 passed / 0 failed, exit 0, 13:0x | (receipts/sql-rdf-dialect-2026-10-22.md — re-run 2026-10-22: 33 passed / 0 failed, exit 0)
| SQLRDF-02 | shape-probe + hostile-court rungs (`sql_rdf_dialect` + `sql_rdf_laundering_corpus`) | DONE — dialect 25/0 (13:0x, this lane); hostile corpus 11/11 as-reported by the corpus lane (`designs/RECEIPTS.md` §2, 12:58 PDT, embedded output there) |
| SQLRDF-03 | `cargo test --test sql_rdf_dialect` | DONE — 25/0, exit 0, 13:0x (this lane; HAVING court + schema-qualified-from courts green in the run) |
| SQLRDF-04 | `cargo test --test chicago_validation` | DONE — re-executed: 9 passed / 0 failed, exit 0, 13:0x (§7.1 MERGE + BLAKE3(Canon(G_AST)\|\|P_params) receipt courts green) | (receipts/chicago-validation-2026-10-22.md)
| SQLRDF-05 | `cargo test --test duckdb_statement_kinds` | DONE with standing environmental exception — re-executed: 5 passed / 1 failed, exit 1, 13:0x; sole failure is the `.gram` pin-source absence (see Known-environmental exceptions) | (receipts/duckdb-statement-kinds-2026-10-22.md — re-run 2026-10-22: 6 passed / 0 failed; the .gram environmental exception has cleared, legacy 5/1 kept as history)
| SQLRDF-06 | `docs/refusals.md` register row + `cargo test --test sql_rdf_dialect` | DONE — REFUSAL-02 row confirmed in `docs/refusals.md` (grep 13:0x, `UnimplementedStatementRefusal`); refusal court green inside dialect 25/0 |
| SQLRDF-07 | `cargo test --test chicago_validation` | DONE — re-executed: 9/0, exit 0, 13:0x (evasion set + join/refusal-03 courts green in the run) |
| SQLRDF-08 | wasm boundary build + `cargo test -p graphlaw-wasm` | DONE — host contract re-executed: 7/0, exit 0, 13:0x; boundary build re-executed green lane-isolated (`CARGO_TARGET_DIR=target-lane-docs`, `wasm` profile 4m25s — shared `target/wasm` unusable, see environment notes); artifact re-pinned, digest verified (Successor §2) |
| SQLRDF-09 | `sql_rdf_dialect` + `chicago_validation` | DONE — re-executed: 25/0 + 9/0, 13:0x; `H_query = BLAKE3(Canon(G_AST) \|\| P_params)` via `purrdf::try_canonicalize`, incl. the 5-variant cosmetic-invariance set (receipt identical across comment/whitespace/alias variants) |
| SQLRDF-10 | `cargo test --test trust_attest` | DONE — re-executed: 10 passed / 0 failed, exit 0, 0.06s, 13:0x | (receipts/trust-attest-2026-10-22.md)
| SQLRDF-11 | `cargo test --test sql_rdf_conformance` | DONE — §6.2 court 5/5 as-reported 13:1x (Successor §3, embedded output); this lane's own re-run at 13:0x was compile-blocked mid-mutation + ENOSPC recurrence, so the citation stands per the receipt rule (receipt with embedded output = receipt) |
| SQLRDF-12 | `docs/reference/sql-rdf-rec.md` on disk + cited | DONE — file present (verified 12:22 and 13:0x); cited from the conformance rung and §6.2 court text |
| SQLRDF-13/14/15 | none (follow-ups) | Filed + verified on disk 12:22; not in v26.10.22 acceptance set |

### Final-verification environment notes (13:0x PDT)

- **Live mutation court on the shared tree**: `src/abi.rs:48` observed mid-mutation (`16 * 1024 / 1024` with the `~ changed by cargo-mutants ~` marker, 13:04), transiently breaking the wasm-boundary const assert (`wasm/src/lib.rs:32`, E0080); restored to `16 * 1024 * 1024` by 13:06. Not a tree defect — disclosed so the transient E0080 is not misread as a boundary failure.
- **Shared `target/wasm` unusable for fresh build scripts**: newly compiled build-script binaries lacked the exec bit (`Permission denied`, os error 13, two packages); `chmod +x` did not hold across cargo's copy steps. Boundary build re-executed lane-isolated (green, 4m25s). Root cause not diagnosed — noted environment hazard.
- **ENOSPC recurrence** (~13:10): volume returned to zero-free during the concurrent mutation run (`/tmp` writes failing); the §6.2 conformance re-run was the gate lost to it.

## Known-environmental exceptions

- **generated_kinds `.gram` absent** — fails only because `/Users/sac/frozen-duckdb/target/wasm-build/` is gone; guarded parse-parity surface VERIFIED-EXTERNAL 21/21 (upstream snapshot comparison, per SQLRDF-15's frozen-pin analysis). Loud, typed environmental failure — not a pass, not a silent skip.
- **frozen-duckdb pinned sources missing** — same root cause; BLOCKS SQLRDF-15 regeneration and the tpch sf=0/sf=0.01 adjudication (UNKNOWN).

## Disclosed hand-patches (regeneration-away-from-a-regression)

- `ggen/sql-dialect/sql_extensions.rs:22-26` — tpch `dbgen(sf = 0.01)` replacing generator-emitted `sf = 0`; witnessed 2026-10-22, contradicted by upstream `dbgen.cpp:905-910`/`1259-1271`. Retired via follow-up SQLRDF-15.

## Follow-ups filed

- SQLRDF-13 — §4 `sh:class` entailment (validation-scoped RDFS closure)
- SQLRDF-14 — §3.1 letter conformance, re-parent 35 PEG leaves under six family classes
- SQLRDF-15 — §6.2 tpch sf=0.01 hand-patch retirement (see above)

## Open defects accepted this window

- **Compiler-hardening escapes E2/E3/E4/E5 + fuzz finding 4**: ON RECORD as OPEN per Blockers' earlier revision ("E1–E7: open, unaddressed", read 12:22 PDT); no L2 final receipt closing them existed at that read. Subsequently (12:45) E1/E3/E4/E5 closed by L2 fixes with courts, E6 refuted, E7 declared (see Blockers resolution record); E2 remains the one escape without a stated close receipt in this file.
- **Mutation court aborts pre-mutation** (12:33): see Blockers; full-sweep anti-vacuity UNMET this window.
- MERGE USING `SELECT` keyword + select-list REFUSAL-03: superseded by the 12:26/12:30 court runs — resolved-by-court, pending D-lane confirmation.

## Aggregate

- Receipt formula witnessed: `H_query = BLAKE3(Canon(G_AST) || P_params)` — chicago_validation 9/0 green (12:30), canonicalization via `purrdf::try_canonicalize` (RDFC-1.0).
- `Σ_PQC` UNMET — registered in `docs/refusals.md` (SQLRDF-10, gate `cargo test --test trust_attest` 10/0, 12:24).
- REFUSAL-01..04 typed and stable in `docs/refusals.md`; courts green (dialect 25/0 final 13:0x; was 18/0 at 12:26 before L1/L2 additions).

## Receipt fields (operating-doctrine)

- **Identity**: tree `ed3be81` + uncommitted working set; receipts embedded above with command + exit + timestamp.
- **Authority**: docs lane, file ownership = this file (append-only).
- **Consequence**: 12 acceptance tickets (SQLRDF-01..12) landed + courted on the final tree (final counts in the table above); 3 follow-ups filed; **9 code defects found + fixed** (fixture `statementIndex` backfill, MERGE USING missing `SELECT`, select-list REFUSAL-03 gap, escapes E1/E3/E4/E5, gws4/wasm_duckdb_op stale targets ×2) plus 1 refuted (E6) and 1 declared-by-design (E7); environmental failures loud and typed (`.gram`, frozen-pin checkout, ENOSPC recurrence); full anti-vacuity sweep UNMET (per-court inversions remain local evidence).
- **Replay**: from a checkout at `ed3be81` + the working set, run the per-row gate commands; expected counts: dialect 25/0, chicago 9/0, kinds 5/1 env, trust 10/0, wasm contract 7/0, boundary build green, conformance 5/0. The `.gram`-dependent test fails identically on any host without `/Users/sac/frozen-duckdb/target/wasm-build/`.
- **Standing**: all 12 tickets DONE — CONFORMANT-ON-OBSERVED-TREE for SQLRDF-01..04, 06..10, 12; DONE-with-loud-environmental-exception for SQLRDF-05/11; ticket Status fields stay **Open** pending coordinator flip.
- **Falsifiers**: (a) restore the frozen wasm-build checkout and rerun duckdb_statement_kinds — must go green or yield a code-level diagnosis; (b) fix `tests/test_cold_chain_ocel.rs:61` — mutation_court must reach mutant testing, else the abort is a new defect; (c) any court passing with the §6.2 tpch hand-patch deleted but the generator un-patched is FAIL per SQLRDF-15.
- **BLOCKED/UNSUPPORTED**: SQLRDF-15 regeneration BLOCKED on frozen-pin checkout restore; `Σ_PQC` UNSUPPORTED this window (registered conformance note, Ed25519 path only); `.gram` parse-parity VERIFIED-EXTERNAL 21/21.

---

# Successor program layer (appended 13:1x PDT, docs lane)

What this session landed BEYOND the 12 acceptance tickets. Claims re-verified on disk at append time unless marked as-of-date.

## 1. frozen-duckdb 2.0.0 registry switch

`Cargo.lock:1182-1186` — `name = "frozen-duckdb"`, `version = "2.0.0"`, crates.io registry source, `checksum = "f9eae23d5949bbab32c26cfebc1784239c52b1bb0c95d33fed63f1b04db66399"`. The path/checkout pin is gone. `Cargo.toml:35` — `frozen-duckdb = { version = "2.0", optional = true }`. Authority gate (trust_attest, previously 10/0 at 12:24): claim is **as-of-date** — a re-run at append time was compile-blocked by another lane's in-flight `src/` edits (see §6), not re-witnessed by this lane. (receipts/frozen-duckdb-pin-2026-10-22.md — re-verified 2026-10-22: live Cargo.lock:1197-1200 pins 2.0.2 / checksum `7b2cd958…5692`; the 2.0.0 / `f9eae23d…66399` record above is as-of-date historical, divergence witnessed, not laundered)

## 2. wasm artifact re-pin — 332ea218 (re-executed at append time)

```
$ blake3sum priv/graphlaw.wasm; cat priv/graphlaw.wasm.blake3
332ea2188fafb465666e71407e8f940c5b60a15184e6ad6567bb4f723fe2c30c  priv/graphlaw.wasm
332ea2188fafb465666e71407e8f940c5b60a15184e6ad6567bb4f723fe2c30c  graphlaw.wasm
```
Both digests identical: artifact matches pin `332ea218…2c30c` (receipts/wasm-artifact-pin-2026-10-22.md — re-verified 2026-10-22: on-disk priv/graphlaw.wasm hashes to this pin). Negative control (tampered artifact → `graphlaw-run` refusal, exit 84) is documented at `src/bin/graphlaw_run.rs:47,56,177`; the runner binary was not rebuilt by this lane (cargo lock held by in-flight lib edits), so the exit-84 run itself is **as-of-date / not re-witnessed here**.

## 3. Moonshot layer (SQLRDF-16 corrected + 17 + 18)

- `docs/jira/v26.10.22/SQLRDF-16.md` — corrected 8-row table, skeptic-hardened falsifiers, corrected-per-skeptic banner at :26-29. `SQLRDF-17.md` — C2-hole ticket; Current-behavior pointers re-verified on disk 12:52 (`src/duckdb_op.rs:198-219`, `src/duckdb_eval.rs:183-188`/`:155`, `src/dialect.rs:740-758`). `SQLRDF-18.md` — on disk (verified present at append).
- `THESIS.md` / `PRD-P28-SIL.md` / `DISSERTATION.md` filed in the window (present in the window listing; verbatim prose per the wave dispatch). `designs/` — 10 files on disk (INDEX, RECEIPTS, SKEPTIC-thesis-review, P28-SIL-specs, TC-HOT-benchmarks, A1, C1, E1, E2, E3, R2).
- Laundering corpus **11/11** — witnessed in `designs/RECEIPTS.md` §2 ("cargo test --test sql_rdf_laundering_corpus / running 11 tests", 12:58) — as reported by that lane, embedded output there.
- Conformance rung **5/5** — **re-executed by this lane at append time**:
  ```
  $ cargo test --test sql_rdf_conformance
  test section4_shape_names_present_and_gates_refuse ... ok
  test result: ok. 5 passed; 0 failed; ... finished in 0.29s
  ```
  (receipts/sql-rdf-conformance-2026-10-22.md — re-verified 2026-10-22 at 7/0; 5/5 supersedes the 12:58 4/1 record. `designs/RECEIPTS.md` §4 had recorded 4/5 PARTIAL at 12:58 — the flip-fixture defect has since been repaired; 5/5 supersedes that entry.)

## 4. LICENSE-P28-SIL.md + MIT retention

`LICENSE-P28-SIL.md` on disk: "P28 Sovereign Invariant License" header (line 1), MIT retention at line 4 ("Code license: see [LICENSE](./LICENSE) (MIT)…"), full SIL grant/limitation text through line 136+.

## 5. Currency fixes: README / DIALECTS / refusals

`README.md`, `docs/DIALECTS.md`, `docs/refusals.md` all carry working-set modifications (pre-existing this session); pointers: `README.md:155` (offline verify, ADMITTED(0)), `docs/refusals.md` (REFUSAL-01..04 + `Σ_PQC` conformance note). No re-run — doc-currency, verified present.

## 6. Successor design + follow-up tickets (as-of 2026-10-22 14:1x PDT, verified on disk)

- **P29-RFL code-first refusals design landed** — `designs/code-first-refusals.md` (verified on disk at append): pure-data `RefusalCode` — `#[repr(C)] Copy` 64B, zero heap, no message field (offending-node `BLAKE3` `[u8;32]` replaces every String payload; <10ns construction by construction, TC-HOT-01 verifies); refusal code space is a **ggen-emitted code space** — discriminants ARE the register, REFUSAL-01..04 keep low u16 values, `docs/refusals.md` code tables REGENERATED from `refusal_codes.bin` (prose stays human-authored); **binary-first ledger** — 64B frames become ledger records verbatim, hand-rolled escaper/parser deleted, NDJSON demoted to cold export; **81-needle migration plan** — 81 string needles across 6 court files migrate to code asserts + one rendering court, N0→N6 order with the critical path N0→N1→N2→N3 (N2 depends on SQLRDF-17 canon-bytes replacement).
- **SQLRDF-19** (`docs/jira/v26.10.22/SQLRDF-19.md`) — CI feature-matrix gaps: 5 untested build cells (runner bin with zero coverage, ring-without-duckdb, ring+duckdb seam, no-default-features, pairwise off-default cells). **BLOCKED-BY: v26.10.22 integration commit** (same gate as SQLRDF-16/17/18). Current-behavior pointers verified on disk at filing (`ci.yml:29,33,37,82`, `release.yml:85`).
- **SQLRDF-20** (`docs/jira/v26.10.22/SQLRDF-20.md`) — BEAM hot-upgrade protocol: artifact swap under the supervisor with its own integration court over the real `graphlaw-run` bin + real artifact (`--expect-blake3` pin → admission-table version in the first post-upgrade ledger batch → BLAKE3 chain continuity across the swap; wrong digest must refuse loudly, broken chain must fail closed). **BLOCKED-BY: v26.10.22 integration commit + SQLRDF-19** (the court drives the runner surface, which has zero coverage until SQLRDF-19 cell 1 lands).

## 7. P28 spec layer + loud disclosure (compile freeze)

- `designs/P28-SIL-specs.md` (65 lines): §3.1 incremental receipt fold, §3.2 SPSC evidentiary ring (Phase 2), P28-SIL enforceability map, Phase sequencing (M0→M1→M2→M14); `tests/p28_hot_path_bench.rs` and `tests/p28_determinism_court.rs` on disk.
- **Compile freeze disclosure (other lane's in-flight edits, not this lane's)**: at append time `cargo check` fails on the shared lib (`E0271`/`E0369`/`E0063` — missing-field `receipt_state` initializer, `Option<blake3::Hasher>` `==`, `Fail: Default`), and `cargo test --test p28_determinism_court` is compile-aborted by the same edits (error set shifting between runs — tree actively edited). Disclosed per the compile-freeze SLA, not absorbed into any PASS claim; §1's trust_attest and §2's exit-84 re-witness are gated on the tree compiling.

## 8. Phase 2 — SPSC ring, ledger, join, cache, prepared-statement seam, digest pin (13:5x PDT)

As-of stamps per item; counts recalled from a dispatch were re-read from the tree at use time (one divergence found — digest-pin item).

- **SPSC evidentiary ring landed** — `src/evidence_ring.rs` (26 KB, mtime 13:44); `rtrb` optional behind the `evidentiary-ring` feature (`Cargo.toml:60` dep, `:87` feature, `:262-266` required-features gate on the court), default-off. Includes the no-blocking property test: 1M pushes against a stalled drainer, max push <10 µs, drops observed (loud).
- **Query ledger landed** — `src/query_ledger.rs` (35 KB, mtime 13:52): F1-F7 + format court. Re-executed: `cargo test --test query_ledger_court` → 9 passed; 0 failed (0.16s); lib-side format courts add 3/0 (11 lines-format total per the landing receipt). (receipts/query-ledger-court-2026-10-22.md — re-run 2026-10-22: 9 passed / 0 failed)
- **Ring→ledger join landed** — `joint_ring_to_query_ledger_court` witnessed in this lane's re-run (6/0 for the feature-gated ring court file, 5.14s): max push **292 ns** against the 10 µs fence, drop gaps persisted to the ledger exactly, verify passes; shutdown-always-drains fixed. Output tail:
  ```
  test no_blocking_property_1m_pushes_vs_stalled_drainer ... ok
  test joint_ring_to_query_ledger_court ... ok
  test result: ok. 6 passed; 0 failed; ... finished in 5.14s
  ```
  (Dispatch cited 5/0 for the ring court alone; the joint court raises the file to 6/0 as observed.)
- **C1 cache store landed** — `src/query_cache.rs` (epoch-bounded, F1-F5, receipt law untouched). Re-executed: `cargo test --test query_cache_court` → 7 passed; 0 failed (0.08s). (receipts/query-cache-court-2026-10-22.md — re-run 2026-10-22: 8 passed / 0 failed; court grew from the recorded 7/0)
- **Prepared-statement seam landed** — LRU 256; parameters are `$N`-bound so the same digest+params hits the cached handle.
- **Receipt-surface digest pin landed (then went stale — loud)** — `tests/registry_artifacts.rs` carried the pin at **20/0** at landing, but at 13:5x this lane's re-run is **16 passed / 4 failed** (3.65s): `receipt_surface_pin_matches_live_files_and_refuses_drift` and `artifacts_pin_file_carries_the_live_registry_and_surface_digests` fail with digest drift (`blake3:dcd46bf6…` pinned vs `blake3:049add3e…` live), plus the committed-vs-emission pair. Cause: surface edits (`src/query_ledger.rs` mtime 13:52) postdate the pin — stale-by-default per the count-recall law; needs a registry regeneration, not a hand patch. Filed as a follow-up, not absorbed into any PASS claim. (receipts/registry-courts-2026-10-22.md — re-run 2026-10-22: FAILED 17 passed / 5 failed, digest-drift root persists, needs registry regen)
- **Measure baseline** (dispatch-supplied, bench-class): fold path **184-216 ns** inside TC-HOT-01's 250 ns budget; full-rehash path **11.6x** over budget = the §3.1.1 fallback to close.
- Court files on disk verified at append: `tests/evidentiary_ring_court.rs` (13:45), `tests/p28_hot_path_bench.rs` (13:40), `tests/query_ledger_court.rs` (13:51), `tests/query_cache_court.rs` (13:46), `tests/registry_artifacts.rs` (13:39).

## 9. Never-block fence conversion + restart watermark (as-of 2026-10-22 14:0x PDT, appended 14:1x)

- **Never-block fence conversion landed** (as-of 14:0x): the single absolute "max push <10µs" dev-profile bound is now a two-leg fence per `designs/cold-path-ledger.md` § "Never-block fence conversion (landed)": **dev leg** = ratio fence (stalled max push ≤ 10× unstalled same-binary baseline, baseline floored at 100ns so a sub-instruction baseline cannot make the ratio vacuous — the PROPERTY, load-independent), `no_blocking_property_1m_pushes_vs_stalled_drainer` (`tests/evidentiary_ring_court.rs:158`); **release leg** = absolute stalled max <10µs, `no_blocking_release_absolute_bound` (`tests/evidentiary_ring_court.rs:302`), witnessed 1.584µs < 10µs (single-shot, darwin 25.2.0; embedded output in `designs/cold-path-ledger.md:91-95`).
- **Restart-watermark fix landed** (fuzz court case 4 — crash-restart had silently renumbered onto the persisted ledger head, laundering crash loss as a complete ledger): the drainer (`src/evidence_ring.rs:473-527`) extracts a restart watermark from the resumed NDJSON ledger's last line and opens via `QueryLedger::open_with_producer` (`src/query_ledger.rs:498`) with `restart_seq.max(producer_attempted)`; crash loss persists as gap markers with `complete: false`. **Deviation, receipt-cited**: the watermark is `restart_seq` (resumed NDJSON coverage), not the prescribed live `seq` counter — the live-counter form double-covered 911 buffered sequences (joint-court gap coverage exceeded `dropped_total` by exactly 911; witness `src/evidence_ring.rs:505-512`). Regression court `restart_mid_stream_persists_gap_not_silent_renumber` (`tests/evidentiary_ring_court.rs:329`), receipt embedded at `designs/cold-path-ledger.md:74-76` (7 passed / 0 failed, 5.74s). (receipts/evidentiary-ring-court-2026-10-22.md — re-run 2026-10-22: 10 passed / 0 failed with --features evidentiary-ring)

---

# Successor wave 2 — hardening + setup (appended 2026-10-22 ~17:0x PDT, lane L53)

Post-integration hardening wave over HEAD `615dbfa` (+ working set). Every
number below was re-read from the on-disk record at append time; the earlier
"SPC supplement" prose in Blockers (line 36) cited cpk −0.0018 / −0.0025,
n=1,000,000 — the receipts on disk now read cpk **−0.0007** /
**−0.0011**, samples **999999**, verdict FAIL both legs (superseding values).

## Green lanes (per-lane one-line receipts)

- **Stale-claims skeptic audit** — read-only over docs/registry; 9
  CONFIRMED-STALE + PLAUSIBLE-STALE findings, all with file:line + evidence
  commands; corrections applied same day (REFUSAL-04 trigger, `feature
  duckdb` sentences, 25 MiB ceiling, CLAUDE.md default-features comment);
  boundary gate `CARGO_TARGET_DIR=target-lane-skeptic cargo test -p
  graphlaw-wasm` → 9 passed / 0 failed, exit 0 (embedded at [receipts/graphlaw-wasm-2026-10-22.md — re-run 2026-10-22: 9 passed / 0 failed, exit 0]
  `designs/stale-claims-audit.md:108-109`); C1/C2 pin-corruption findings
  remediated (`docs/wasm-integration.md` pin section now names
  `332ea218…2c30c` / 6,793,617 B; corrected `REFUSAL-04` trigger at
  `docs/refusals.md:45`). Verdict: **LANDED (audit + partial remediation;
  registry re-pin remains open — C9)**.
- **Receipts-chain audit (L31)** — `b3sum` over all 6 `receipts/` files,
  exit 0; verdicts **5 WITNESSED / 1 NO-OUTPUT / 0 BROKEN**
  (`receipts/sa2a/factory-b-v26.9.29-executable-gates.md` is NO-OUTPUT:
  "authored, not executed"); chain **UNCHAINED** — zero parent/prev-hash
  fields in `receipts/`, the BLAKE3 chain exists only in
  `src/receipt_chain.rs:376-418` over in-memory frames; headline: the
  "BLAKE3-chained receipts/" claim is unsupported on disk. Verdict: **LANDED
  (audit-only; chain adoption is a coordinator decision)**.
- **ggen pack drift (L28)** — `ggen/sql-dialect` vs `src/sql_ast.rs`:
  **8 drift rows D1-D8 + 3 aligned D9-D11**; drift greps carry real exits
  (join/operator/SELECT-predicate greps exit 1 = term absent from pack;
  ConnectStatement/DisconnectStatement grep exit 0 ×2 = refusal arms at
  `src/sql_ast.rs:2160-2164`); D6 refusal is typed (REFUSAL-02), never
  silent. No edits made; gate surface unchanged. Verdict: **LANDED
  (report-only)**.
- **Shapes ↔ Rec §4 drift (L33)** — `ontologies/sql-ast.ttl` +
  `ast_governance.ttl` vs `docs/reference/sql-rdf-rec.md` §4: **21-row drift
  matrix, 7 ENFORCED-BOTH / 8 SHAPE-ONLY / 5 REC-ONLY**, plus 2 inter-shape
  findings (conjoined governance contradiction forcing alias forms;
  `<>` vs `!=` not-equals spelling); post-read boundary gate
  `cargo test --test sql_rdf_dialect` → 25 passed / 0 failed, exit 0. (receipts/sql-rdf-dialect-2026-10-22.md — re-run 2026-10-22: 33 passed / 0 failed)
  Verdict: **LANDED (report-only)**.
- **Doc-staleness gate (L35)** — `scripts/doc_staleness_gate.sh` landed
  (exit 0/1/2 = pass/findings/internal-error); verification run **50
  findings, EXIT=1 honest-FAIL**, anti-vacuity witnessed: injected stale
  file flips 50→52 findings, removal restores 50/EXIT=1; findings are the
  remediation backlog. Verdict: **LANDED (gate live, tree FAIL-by-design
  until backlog cleared)**.
- **Rules-pruner design (L34)** — design-only, no code: measured baseline
  **749 lines** of standing doctrine (~9k tokens/lane, ~20x a 20-lane wave),
  **14 laws surveyed, 4 MECHANISM (P1-P4), 8 PROSE-only** pruner candidates
  (P5, P8-P14); witnessed-firing event-log schema (NDJSON per scope)
  specified. Verdict: **LANDED (design)**.
- **TC-HOT-01 baseline (L20)** — `CARGO_TARGET_DIR=target-lane20 cargo bench
  --bench tc_hot_01`, **exit 1 honest-FAIL**: clone-path p99 **292ns** >
  250ns budget; refusal leg p50/p99 **250/416ns** > 100ns PRD; PASS legs:
  param-integration small **net 23.00ns** / large amortized **net −28.10ns**
  ≤ 40ns, timestamp source **0.7ns** < 10ns monotonic; variance self-check
  spread 0.00% < 5%; no timer-suspect numbers (5.8x baseline ratio
  consistent with mechanism). Verdict: **LANDED (honest-FAIL baseline; two
  budget closures filed)**.
- **SPC court receipts** — `scripts/spc_court.sh` wrote
  `receipts/spc-2026-10-22.json` + `-inject.json` (verdict FAIL both,
  embedded measured counts): base cpk **−0.0007**, mean **317.469ns**,
  p99 **292ns** vs USL 250ns; inject cpk **−0.0011**, mean **304.262ns**,
  p99 **333ns**; samples **999999** each; the gate can FAIL — the point.
  Verdict: **LANDED (honest-FAIL; TC-HOT-01 perf budget genuinely unmet)**.
- **TC-MEGA-FIBO-01 corpus** — generator `generators/gen_corpus.py`:
  smoke (`--legs 1000`) exit 0, full (`--legs 100000`) exit 0 (~1.3s wall);
  **100,000 legs / 3 jurisdictions / 5,000 LEIs / 12 holders**; dialect
  partition .ttl/.trig/.nq + hand-authored malformed anti-vacuity twin
  (`antivacuity-leg-41209-malformed.ttl`); **DETERMINISM-OK** (two full
  re-generations, `diff` of sha256 digest lists empty, digests recorded);
  rdflib parse checks exit 0 (full .nq = 400,000 quads; smoke .ttl 8,536
  triples; malformed twin 9 triples); full .ttl/.trig parse at scale
  NOT YET RUN (disclosed, not claimed). Verdict: **LANDED (corpus; full-scale
  admission run is the compiler lane's event)**.

## Blocked lanes (typed BLOCKED)

- **L3 — gws4_a / gws4_c IPC-injection courts** — **BLOCKED (wasm link)**:
  `tests/gws4_ipc_injection_court.rs:305`
  (`gws4_a_corpus_injected_as_linear_memory_buffer_loads_into_duckdb_eval`)
  and the gws4_c leg require
  the compiled wasm artifact (the harness builds `-p graphlaw-wasm --target
  wasm32-wasip1 --profile wasm` at `tests/gws4_ipc_injection_court.rs:237`
  and asserts on a real wasmi instance); the artifact link is unavailable in
  this wave's tree state. Falsifier: restore the artifact link and rerun
  `cargo test --test gws4_ipc_injection_court` — must go green or yield a
  code-level diagnosis.
- **L27 — mutation-court run-mode demo** — **BLOCKED**: no on-disk lane
  record found for this lane's demo run (searched `designs/`, `scripts/`,
  window docs); the full `bash scripts/mutation_court.sh` sweep is
  independently recorded as aborting pre-mutation (Blockers, 12:33) and the
  run-mode demo inherits that blocker. Loud skip, not a pass; falsifier:
  fix `tests/test_cold_chain_ocel.rs:61` and rerun the sweep.
- **L32 — TC_HOT_FULL run** — **BLOCKED (unexecuted full-corpus run)**:
  the 10M-iteration full run is gated at `benches/tc_hot_01.rs:154`
  (`TC_HOT_FULL=1`); only the SMOKE baseline (L20, above) executed. CTQ 4/5
  and the 25M-row TC_HOT_FULL event remain UNKNOWN
  (`designs/ctq-traceability.md:11,20`). Falsifier: `TC_HOT_FULL=1 cargo
  bench --bench tc_hot_01` must execute 10M iterations and gate, or yield a
  typed failure.

## INDEX status sync (wave-2 closures)

- **SPOF #3 cache poisoning closed on disk**: `src/query_cache.rs:53-63`
  now recovers the inner guard on a poisoned mutex
  (`PoisonError::into_inner`), replacing the process-wide `.expect` brick
  that SPOF scan item 3 registered; the "panic loses at most the
  in-flight map update" property is documented at `src/query_cache.rs:137-140`.
- **Silent-skip census item closed**: `tests/insa_law_order.rs:51-73` now
  carries the loud typed skip (env override `INSA_LAW_PACK_ONTOLOGY`,
  names the missing pack path, never a silent skip).
- **Stale-claims remediation landed**: `docs/wasm-integration.md` pin
  section rewritten to the live pin (`332ea218…2c30c`, 6,793,617 B);
  `docs/refusals.md:45` REFUSAL-04 trigger corrected; dialect-court counts
  in INDEX rows SQLRDF-03/09 synced 18/18 → 25/0 per the wave-final
  receipts (RECEIPTS.md "Standing per ticket" table, 13:0x).

---

# Successor wave 3 — DX + drift + pre-writes (appended 2026-10-22 ~17:4x PDT, lane L100)

Consolidation of the post-wave-2 skeptic/audit, DX, pre-write, and
cross-repo lanes. Every number below was re-read from its on-disk source
report at append time (mtime-stamped per file); counts recalled from the
dispatch were re-verified or marked not-found. No gate was re-executed by
this lane except where stated — each entry names its embedded receipt.

## Wave 3a — skeptic / audit lanes

- **L21 stale-claims audit** — `designs/stale-claims-audit.md`: 9
  CONFIRMED-STALE (C1-C9), 6 PLAUSIBLE-STALE (P1-P6), 5 UNVERIFIED, 4
  DISMISSED, 0 wrong-premise tests; boundary gate lane-isolated
  `cargo test -p graphlaw-wasm` → 9 passed / 0 failed, exit 0 [receipts/graphlaw-wasm-2026-10-22.md — re-run 2026-10-22: 9 passed / 0 failed, exit 0]
  (embedded at `stale-claims-audit.md:107-113` command table). C1/C2 pin
  remediation landed; **C9 (registry pin drift) closed by L73 below**.
  Verdict: **LANDED (audit + partial remediation; C9 → L73)**.
- **L68 split verification (skeptic) + L76 repair** —
  `designs/split-prewrite-verification.md`: 186 range-containment checks,
  174 byte-exact pass, 12 diagnosed; verdicts L43/L45/L46/L67 VERIFIED,
  L66 DISCREPANCY (4 rustfmt reformat sites vs the "verbatim" claim,
  semantics intact), L44 DISCREPANCY (RefusalCode/RefusalContext/AstError
  never extracted + false "HEAD 615dbfa" base pin). **L76 repair landed**
  (post-17:20): `src/sql_split/SPLIT-NOTES.md` "Base pin (corrected
  2026-10-22, lane L76)" states the working-tree base and the working-tree
  lane edits included; `src/sql_split/ast.rs` now carries RefusalCode
  (7 hits) and `refusals.rs` (21 hits) — the missing extraction is in.
  Mutation control on the court witnessed non-vacuous (struct rename flips
  containment to False). Verdict: **LANDED (verify + repair; L66 reformat
  finding open for the switchover lane)**.
- **L69 pending-marker census** — `designs/pending-marker-census.md`:
  sweep `grep -rn 'PENDING\|TBD\|…'` exit 0, 126 hit lines; classification
  **7 RESOLVED-SINCE / 10 LIVE-BLOCKED / 8 HONEST-STATE families (~70) /
  0 ABANDONED**; dominant blocker = the v26.10.22 integration commit
  (unblocks SQLRDF-16..20 chains); mutation-court abort-class blocker's
  subject landed (`fn object_count` at `src/sql_ast.rs:1150`) but the
  sweep re-run receipt is pending (item 8b). Verdict: **LANDED (census)**.
- **L93 scripts census** — `designs/scripts-census.md`: 5 scripts —
  wasm_audit.sh CURRENT (typed exit 3 MISSING-SUBJECT; fat-fixture
  non-vacuity probe → exit 1 BREACH), wasm_unknown_unknown_smoke.sh
  LINK-BLOCKED (typed artifact-missing exit 1),
  gen_wasm4pm_graphlaw_vectors.py CURRENT exit 0, gen_corpus.py CURRENT
  exit 0, gen_shapes.py CURRENT exit 0 (determinism: identical aggregate
  sha256 on rerun); **0 ORPHAN**. Verdict: **LANDED (census)**.
- **L31 receipts-chain audit** — `designs/receipts-chain-audit.md`:
  `b3sum` over all 6 `receipts/` files, exit 0; **5 WITNESSED / 1
  NO-OUTPUT / 0 BROKEN** (`receipts/sa2a/factory-b-…-executable-gates.md`
  is "authored, not executed", :9); chain **UNCHAINED** — zero
  parent/prev-hash fields on disk; the BLAKE3 chain exists only in
  `src/receipt_chain.rs:376-418` over in-memory frames. Verdict:
  **LANDED (audit-only; chain adoption is a coordinator decision)**.

## Wave 3b — DX lanes (landing receipts on disk)

- **L50 CLAUDE.md ladder receipt** — `designs/claude-md-ladder-receipt.md`:
  per-rung verdicts — cargo check exit 0 (52.32s); acceptance targets
  TARGET-PROVEN via `--list` (chicago 9, dialect 25, trust 10, kinds 6 fns);
  mutation_court **TYPED-REFUSAL exit 6** (MUTATION-COURT-DIRTY-TREE — gate
  healthy, loud skip); spc_court FAIL (gate healthy); wasm rung KNOWN-RED
  NOT RUN. Verdict: **LANDED (ladder receipt)**.
- **L57 SPC mktemp robustness fix** — `designs/spc-mktemp-fix.md`:
  `scripts/spc_court.sh:37` mktemp died pre-gate under a polluted TMPDIR
  (6178 stale entries, mkstemp failed); temp-file block fixed,
  measurement untouched; L50 reproducer stands. Verdict: **LANDED (fix)**.
- **L59 eyeron pin reconciliation** — `designs/eyeron-pin-reconciliation.md`:
  the 0.7.8 (vendored crate) vs 0.7.7 @ d6568f65 (BACKEND_AUTHORITIES)
  split is a court-enforced invariant, not drift — DOC-ONLY, no change;
  `git diff fe87783 HEAD -- crates/graphlaw-eyeron/src` empty (exit 0).
  Verdict: **LANDED (doc-only reconciliation)**.
- **L60 PEG-kinds claim receipt** — `designs/peg-kinds-claim-receipt.md`:
  the "35 PEG statement kinds" CLAUDE.md claim is witnessed inside 6 test
  fns — 35 `sql:StatementKind` in `statements.ttl`, 35 rows in
  `sql_statement_kinds.rs`, loop over `SQL_STATEMENTS` at
  `tests/duckdb_statement_kinds.rs:358`. Verdict: **LANDED (claim verified)**.
- **L61 ggen-pack reconciliation plan** —
  `designs/ggen-pack-reconciliation.md`: adjudicates L28's D1-D8 drift into
  per-row verdicts; generator-of-record finding — `statement_classes.ttl`
  is generated from the frozen DuckDB PEG grammar via
  `extract_statements.py` (drift-detect exit 1), so any "add the term"
  verdict executes via generator + regen, never hand-edit. Verdict:
  **LANDED (plan; read-only)**.
- **L63 cargo-deny receipt** — `designs/cargo-deny-receipt.md`: cargo-deny
  0.20.2; before: exit 5 config deserialize error (deny.toml:31); after:
  full `cargo deny check` exit 5 with 11 errors (advisories 9 RUSTSEC,
  licenses 2 LicenseRef) — **bans/sources ok**; honest-FAIL, backlog is
  the receipt. Verdict: **LANDED (honest-FAIL baseline)**.
- **L70 examples census** — `designs/examples-census.md`:
  `cargo build --examples` EXIT=0 (15/15) in 1m49s; **15 COMPILED+RAN /
  0 COMPILE-FAIL / 0 SKIP-TIMEOUT**; cross-repo side effect disclosed
  (vector_gen examples write into the wasm4pm fixtures dir). Verdict:
  **LANDED (census)**.
- **L72 refusals-register reconciliation** —
  `designs/refusals-register-reconciliation.md`: 12 surfaces enumerated,
  **5 register additions** (kernel `NO-CANONICAL-GRAPH`, `ATTEST-REFUSED`;
  harness section: `WASI_PATCH_BUILD_REFUSED`, `MUTATION-COURT-DIRTY-TREE`
  exit 6, `GRAPHLAW-WASM-BUILD-FAILED`/`BLOCKED-LINK`, design-stage
  MANIFEST/DISK-REFUSED); 0 renumberings, 0 discriminant-number defects
  (LITERAL_NOT_INT=36 / LITERAL_NOT_FLOAT=37 / JOIN_TYPE_IRI=37 all MATCH);
  appended-row grep 7 rows exit 0. Verdict: **LANDED (register
  reconciliation)**.
- **L85 refusal playbook** — `docs/refusal-playbook.md`: user-facing
  troubleshooting table of 13 refusal messages captured by triggering each
  refusal FOR REAL via `examples/l85_refusal_probe.rs` (exit 0, 13/13
  MATCH); Display-string improvement findings filed for the src lane;
  first lane-target build failed on a dependency compile (disclosed, :110).
  Verdict: **LANDED (playbook + probe)**.

## Wave 3c — pre-writes (explicit UNWIRED status)

All seven split trees remain **UNWIRED**: `grep -rn "_split"` and
`grep -rn "mod .*split"` over `src/` + `crates/graphlaw-eyeron/src`
outside the split dirs return no matches (exit 1) — no mod declaration
references the splits; switchover is a later phase (L68, bottom line).

- **L43 reasoner_split** — VERIFIED (131/131 provenance ranges byte-exact;
  sha256 matches SPLIT-NOTES pin; rustfmt @2021 exit 0 ×7). UNWIRED.
- **L44 sql_split** — DISCREPANCY → **repaired by L76** (missing
  RefusalCode/RefusalContext/AstError now extracted; base pin corrected to
  the working tree in `src/sql_split/SPLIT-NOTES.md`). UNWIRED.
- **L45 parser_split** — VERIFIED (8/8 ranges byte-exact; gates exit 0).
  UNWIRED.
- **L46 receipt_split + registry_split** — VERIFIED (frame/record/store +
  emit ranges byte-exact; repr/field-order invariances held by
  byte-exactness; edition-2024 gates exit 0). UNWIRED.
- **L65 query_ledger_split** — present (`src/query_ledger_split/`, batch.rs
  442 lines, storage.rs 534 lines, no SPLIT-NOTES); **not verified by L68
  (skip per dispatch) — verification status UNKNOWN**, UNWIRED.
- **L66 dialect_split** — DISCREPANCY (4 rustfmt reformat sites vs the
  "verbatim" claim; refusal codes/variants/discriminants untouched;
  gates exit 0). UNWIRED.
- **L67 abi_split** — VERIFIED (disclosed arm-glue only; union-diff zero
  body edits; edition-2024 gates exit 0 ×6). UNWIRED.

## Wave 3d — cross-repo + registry/shapes

- **L73 registry regen (closes L21 C9)** —
  `designs/registry-regen-receipt.md`: real flow
  `cargo run --features abi --bin graphlaw-registry -- --write`, exit 0;
  registry pin `7db561aa…` → `4cd56fd5…` (L21's cited `5d31d472…` had
  already moved again — pinned what the tree hashes at run time);
  receipt-surface `78ebb149…` → `0cc02ca4…`; wasm row byte-identical
  (never re-pinned by the bin). Gates re-run: `registry_artifacts`
  **22 passed / 0 failed**, `registry_dispatch` **14 passed / 0 failed**, [receipts/registry-courts-2026-10-22.md — re-run 2026-10-22: registry_artifacts FAILED 17/5, registry_dispatch 14/0]
  both exit 0. Verdict: **LANDED (regen; C9 closed)**.
- **L71 shapes ↔ Rec §4 reconciliation applied** —
  `designs/shapes-reconciliation-applied.md`: rows 16/19/12/§2.B applied to
  `ontologies/sql-ast.ttl` + `ontologies/shapes/ast_governance.ttl`
  (hand-authored canonical graphs; nothing generated exists to regen);
  SHACL-legality deviation from L55's literal text disclosed (pyshacl
  ShapeLoadError as receipt); Chicago gate: real pyshacl validator —
  normative-only doc False→True, WHERE-less refused in BOTH spellings
  (anti-vacuity confirmed), gate exit 0. Courts:
  `sql_rdf_dialect` **25/0**, `w3c_conformance` **8/0**, both exit 0
  (embedded at the top of the file). Verdict: **LANDED**.
- **L90 wasm4pm fixtures drift court** — `designs/wasm4pm-fixtures-drift.md`
  (repo `/Users/sac/wasm4pm`, canonical checkout; graphlaw @ 615dbfa):
  44/46 generator-emittable fixtures **ALIVE** (regeneration at 615dbfa
  reproduces committed bytes or the refreshed bytes are the generator's
  exact output); 4 description_logic fixtures STALE-FIXTURE (updated in
  place by the real generator, provenance rev 82c67c2b→615dbfa); 6
  situation_calculus/strips fixtures ALIVE with registered
  provenance-constant drift (`gen_wasm4pm_graphlaw_vectors.py:24` pins
  GRAPHLAW_REV as a stale constant — graphlaw-side fix suggested); 2
  hand-derived selftest twins excluded by design (typed HAND-DERIVED).
  `cargo test -p wasm4pm-cognition --test graphlaw_vector_oracle` —
  **DISK-BLOCKED** (11.3 GiB free < 20 GiB hook refusal), not a pass.
  Verdict: **LANDED (fixtures refreshed; oracle gate BLOCKED-disk)**.
- **L89 marketplace qualification — NOT FOUND on disk** (loud, not a
  silent skip): no qualification report for a marketplace pack exists in
  `docs/jira/v26.10.22/` (grep for marketplace/qualification across the
  window returned no lane report). The ggen pack surface on disk is
  `ggen/sql-dialect/` with the L61 reconciliation plan (above) as its only
  qualification-adjacent artifact. Falsifier: the L89 lane report file —
  if it exists outside the window, INDEX/RECEIPTS must cite it; until
  filed, this item stands **UNFILED**.
- **L98 wasm4pm fixtures (second leg) — NOT FOUND on disk** (loud): no
  L98-labeled report exists in the window; the only wasm4pm fixtures
  record is L90 above. Same falsifier as L89.

## Typed BLOCKED entries

- **L1 wasm link — BLOCKED (environmental, tree-wide)**: the
  frozen-duckdb-sys wasm32 cache holds a host archive
  (`~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1/libduckdb.a is not a
  wasm32 archive … rebuild with WASI_SDK_PATH set`), witnessed at
  `frozen-duckdb-sys/build.rs:97` — sweep census
  (`designs/sweep-pre-integration-2.md`): **95 test failures across 12
  targets all trace to this one root** (typed
  GRAPHLAW-WASM-BUILD-FAILED / BLOCKED-LINK at `tests/common/mod.rs:65`);
  gws4 IPC-injection court 1 passed / 2 failed on the same root [receipts/gws4-ipc-injection-court-2026-10-22.md — re-run 2026-10-22: FAILED 1 passed / 2 failed, same WASI-SDK root]
  (`refusals-register-reconciliation.md` embedded run). Falsifier: rebuild
  the wasm archive with WASI_SDK_PATH and rerun
  `cargo test --test gws4_ipc_injection_court` — must go green or yield a
  code-level diagnosis.
- **L92 — TC_HOT_FULL receipt pending**: the full-corpus adjudication
  records conflict on disk and both carry the as-of law. ctq-traceability
  (`designs/ctq-traceability.md:11,20`): 25M-row TC_HOT_FULL **UNCOURTED
  at scale — unrun**; yet `designs/pce-baseline.md` (earlier mtime)
  witnesses a TC_HOT_FULL=1 min-of-5 25M run (PCE **70.36%**, substrate
  overhead **42.12%** — charter "<2.8%" REFUTED at that operating point).
  No durable FULL receipt file for tc_hot_01's 10M-iteration latency court
  exists either. Status: **FULL receipt pending** until a durable,
  embedded-output receipt lands and ctq-traceability is synced. Falsifier:
  `TC_HOT_FULL=1 cargo bench --bench tc_hot_01` (and tc_hot_02 at 25M)
  with the output written to `receipts/`.
- **L99 — mutation-court FULL sweep receipt pending**: the abort-class
  blocker's subject landed (`fn object_count`, `src/sql_ast.rs:1150`;
  L69 RESOLVED-SINCE #7) and L50 witnessed the gate's healthy typed
  refusal (exit 6 MUTATION-COURT-DIRTY-TREE) — but no on-disk receipt of a
  full `bash scripts/mutation_court.sh` run reaching mutant testing exists.
  Status: **FULL receipt pending** (L69 item 8b). Falsifier: the sweep
  re-run must reach mutant testing (exit not-4, not-6) or yield a typed
  failure.

## Wave-3 verification (this lane)

Spot-greps at append time (all sources re-read, not recalled): L73 exits
22/0 + 14/0 vs `registry-regen-receipt.md` "Court exits" — MATCH; L71
courts 25/0 + 8/0 vs `shapes-reconciliation-applied.md` court block —
MATCH; L68 "174 pass / 12 diagnosed" vs
`split-prewrite-verification.md` "Court commands + exits" — MATCH; L76
repair confirmed on disk (`src/sql_split/SPLIT-NOTES.md` base-pin section;
RefusalCode in ast.rs). Gate commands embedded in the source reports are
reproduced verbatim above; this lane executed no gates (docs-only lane,
coordinator owns git).

---

# Successor wave 4 — link-green + truth pass (appended 2026-10-22 ~17:5x PDT, lane L117)

Consolidation of the post-wave-3 landings. Every number below was re-read
from its on-disk source at append time; items with no on-disk receipt are
marked **UNFILED** loudly, not absorbed into any PASS claim. This lane's own
spot-verification commands + exits are embedded per entry.

## L1 — wasm link GREEN (on-disk witness: `docs/WASM.md` "Pin currency")

The wasm32-wasip1 link now **exits 0 with full DuckDB linked when
`WASI_SDK_PATH` is a wasi-sdk root** (`bin/clang`,
`share/cmake/wasi-sdk-p1.cmake`; CI installs wasi-sdk 34 at `/opt/wasi-sdk`).
`~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1` is the engine archive, not
that SDK. **[CORRECTION 2026-10-22, lane L134: this paragraph's original
drafting conflated `WASI_SDK_PATH` with the frozen-duckdb engine cache
directory. They are different things — `WASI_SDK_PATH` is a wasi-sdk root
(`bin/clang`, `share/cmake/wasi-sdk-p1.cmake`, e.g. `/opt/wasi-sdk`), while
`~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1/` is the engine's archive
cache. The recorded build/test commands ran against the engine cache
directory; that history stands verbatim — only the identification was
wrong and is corrected here.]** Typed failure at frozen-duckdb-sys
`build.rs` without the root —
`docs/WASM.md:37-45,89-95` (re-read at append). Consequence: the sweep
census's 95 WASM-LINK-BLOCKED failures
(`designs/sweep-pre-integration-2.md:15-21`) trace to an environmental
root that is now cured for any host with a wasi-sdk root on `WASI_SDK_PATH`; the lanes
they blocked (L3 gws4 IPC courts, TC-MEGA WO-10) are unblocked on this
host class. **Re-pin is explicitly PENDING**: the on-disk pins
(`332ea218…`, 6,793,617 B) still describe the 26.10.21 artifact; the
coordinator MUST NOT re-pin until the reproducible-SDK rebuild lands
(`docs/WASM.md:93-98`). **UNFILED (loud)**: no lane report exists on disk
for the four-hop root-cause chain or the file(1) caveat correction
(searched `designs/`, `receipts/`, `docs/`; the only on-disk record is
WASM.md + the sweep census's `build.rs:97/100` panic text). The file(1)
caveat (the cached archive WAS wasm32; `file(1)` cannot distinguish
wasm32 from host archives; the real blocker was the validator clamp) is
dispatch-supplied, marked as-of-dispatch, not witnessed on disk.

## L105 — gws4 court repointed to the single shared build path (fix on disk, receipt UNFILED)

`tests/gws4_ipc_injection_court.rs:229-231` now reads the artifact via
`common::wasm_path()` ("Single shared build path (tests/common/mod.rs):
honors CARGO_TARGET_DIR, …") instead of the court-local duplicate build —
verified on disk at append (git working set shows the file rewritten,
161 lines removed / 64 kept). No lane receipt file exists in `designs/`
— **UNFILED (loud)**. Falsifier: the L105 lane report naming the repoint
must land, or the on-disk comment stands as the sole witness.

## L106 — cross-repo wasm4pm oracle 4/0 (WITNESSED by this lane)

```
$ cd /Users/sac/wasm4pm && cargo test -p wasm4pm-cognition --test graphlaw_vector_oracle
running 4 tests
test exclude_list_refuses_named_breeds ... ok
test graphlaw_rev_court_sabotage_twin ... ok
test graphlaw_rev_ancestry_court ... ok
test graphlaw_vector_oracle_roundtrip ... ok
test result: ok. 4 passed; 0 failed; ... finished in 0.04s
EXIT=0
```
This closes the disk-BLOCKED leg recorded at wave 3 (receipts/wasm4pm-oracle-2026-10-22.md — re-executed 2026-10-22 by lane V-L17, 4/0 confirmed) (L90's oracle gate was
refused at 11.3 GiB free; this lane ran with 66 GiB free) and the L98
routing (the schema-v1 family leg). 4 `#[test]` fns confirmed by grep
against `crates/wasm4pm-cognition/tests/graphlaw_vector_oracle.rs`.

## L111 — spc_court.sh append-only receipts (LANDED; two-run witness)

`designs/spc-append-only-fix.md`: the receipt writer now stamps per-run
timestamps (`receipts/spc-<date>T<HHMMSS>.json`), never overwrites;
two-run witness with distinct files `spc-2026-10-22T173758.json` /
`T173809.json` — **both files on disk** (this lane's `ls receipts/`).
Measurement logic untouched. Spot-grep: exit 0 (files present).

## L108 — chain finding + coordinator re-baseline (LANDED, verified green)

`designs/chain-resync-receipt.md`: root cause = `spc_court.sh` overwrote
the same-day receipt in place while `CHAIN.blake3` treats receipts/ as
immutable — an in-place-mutating writer feeding an immutability chain can
never re-sync (the gate worked as designed; the writer was the defect,
fixed by L111). Coordinator decision (documented in the same file,
"Coordinator re-baseline decision"): the overwrite was a harness defect,
not tampering; re-baseline executed — 8 receipts chained, head
`2c316b0261c992cfb0463d135e4af4f6975daf9318368bfaf531098b57749d05`. (receipts/CHAIN.blake3 — chain register; this head line is as-of-date historical — the chain has since advanced, current head via `tail receipts/CHAIN.blake3`)
This lane re-verified: `tail receipts/CHAIN.blake3` →
`chain_head 2c316b02…d05` (9 lines); `bash scripts/receipt_chain_sync.sh
--check` → **exit 0** (VERIFIED as-of 2026-10-22 ~17:5x PDT).
**[SUPERSEDED 2026-10-22, lane D2 re-audit: on the current tree the same
command exits 1 — `TAMPER-DETECTED:
receipts/sa2a/factory-b-v26.9.29-executable-gates.md` — the file already
audited NO-OUTPUT / "authored, not executed" by L31. Embedded output in the
lane-D2 audit section below. The exit-0 claim above is as-of-date
historical.]** Pre-overwrite content preserved in git
history (cpk −0.0016 numbers).

## L107 — js receipt (LANDED; browser gap noted)

`designs/js-test-receipt.md`: build exit 0 (wasm pin `332ea218…` matches
`priv/graphlaw.wasm` byte check); unit vitest **197/198** — sole failure
`max_outstanding_alloc_bytes` expected 16777216 vs limits.ts 268435456
(stale js mirror vs registry); browser court typed-FAIL (Playwright
Chromium binary missing, `test/browser.test.ts:108` hardcodes
`chromium.launch`, no `channel` escape hatch) with the graceful-skip form
recorded verbatim — **not a pass**; drift gates 3× exit 0.

## L112 — js limits fix (fix verified on disk, receipt UNFILED)

`js/npm/src/limits.ts:43` now reads
`max_outstanding_alloc_bytes: 16777216` (this lane's grep, exit 0) —
the L107 finding is remediated; the doc-table row at `:23` agrees. No
lane receipt file in `designs/` — **UNFILED (loud)**; falsifier: the
L112 lane report, or a re-run of `npm run test` going 198/198.

## L102 — ASSETS regen (receipt UNFILED)

`ASSETS.blake3` carries working-set modifications (git status), but no
lane receipt for the claimed 12/0 + 8/0 courts exists in `designs/` —
**UNFILED (loud)**; the counts are dispatch-supplied, not re-witnessed.

## L103 — advisories 8 → 3 (LANDED)

`designs/advisory-update-receipt.md`: minimal patch-level
`cargo update` (rustls 0.23.45→0.23.46, rustls-webpki →0.103.16) closed 5
RUSTSEC; `cargo check` exit 0; 5-target court run exit 0 (66/0); advisories
re-run exit 1 with **3 errors remaining**, all REQUIREMENTS-CHANGE (h2
0.3→0.4 semver-incompatible; rustls-pemfile + unsafe-libyaml unmaintained
upstream). Spot-grep: `Cargo.lock` rustls **0.23.46** (exit 0, MATCH).

## L104 — ARCHITECTURE.md (LANDED)

`docs/ARCHITECTURE.md` on disk (this lane's `ls`, exit 0); INDEX carries
the closure ("Status: CURRENT, landed 2026-10-22, lane L104").

## L89 / L98 — prior UNFILED markers flipped to FILED

Wave 3's loud not-found entries are superseded: lane reports landed at
17:43 as `designs/l89-marketplace-qualification.md` (marketplace validate
exit 0, 339 packs / 1927 templates; lifecycle court 3/0; wasmex court 1
SKIPPED loud + Jinja2 render byte-identical to `wasm/src/ffi.rs`, b3sum
`9dcab4e4…`; from_raw_parts sweep lawful) and
`designs/l98-wasm4pm-vecgen-provenance.md` (runtime provenance stamping in
`gen_wasm4pm_graphlaw_vectors.py`; 6 fixtures regenerated
SEMANTIC_IDENTICAL, provenance-only diff 82c67c2→615dbfa; gate 3/4 with
the failing leg routed to L106 — since closed by the 4/0 above).

## L113 / L114 / L115 / L116 — UNFILED (loud)

No receipts, reports, or file:line traces for lanes L113-L116 exist
anywhere in `docs/jira/v26.10.22/` or `designs/` (grep at append time,
no matches). Status: **UNFILED — not landed-as-witnessed**; falsifier:
their lane report files landing in `designs/`.

## Wave-4 spot-verification (this lane)

Re-read at use time, not recalled: chain head `2c316b02…` + sync `--check`
exit 0 (commands + output above); rustls 0.23.46 in `Cargo.lock` (L103
MATCH); `limits.ts:43` = 16777216 (L112 fix MATCH); append-only SPC files
present (L111 MATCH); wasm4pm oracle 4/0 exit 0 re-executed by this lane
(L106 WITNESSED). This lane edited docs only; coordinator owns git.

## Wave-M M-lane sync (appended 2026-10-22 ~18:3x PDT, lane M8)

Dispatched M-lanes: statement courts (M2), shapes (M3), wasm boundary (M4),
refusals (M6), authority (M9), ledger/cache (M10), docs-teach (M11),
staleness (M12). Re-read from disk at use time (`ls -lt designs/` +
keyword grep, polls ~90s apart): early polls (18:2x–18:3x) found no
M-lane receipts on disk; a final poll
at close flipped two lanes to FILED. Per dispatch, nothing fabricated.
Status per lane:

- M2 statement courts — OUT; falsifier: its receipt landing in `designs/`.
- M3 shapes — **FILED**: `designs/m3-shapes-receipt.md` (18:29) —
  `sql_rdf_dialect` 25/0 exit 0 against `ontologies/sql-ast.ttl` +
  `ggen/sql-dialect/`; shape-coupled refusal courts named; no shape or
  statement-kind drift.
- M4 wasm boundary — ~~OUT~~ **FILED** (flipped 2026-10-22 ~21:1x PDT, lane L149): `designs/m4-wasm-boundary-receipt.md` — size ceiling PASS (45,162,113 B on-disk re-measure ≤ 64 MiB; dispatch's 45,179,203 B recorded as reported), fat-fixture non-vacuity witnessed, missing-artifact typed panic witnessed with priv/ before==after shas and `git status --porcelain priv/` clean, wasi_patch_guard 3/3 + 2/2 re-executed by L149 exit 0, fuzz + memory-wall typed BLOCKED on the EH wall (third independent corroboration), toolchain: wasi-sdk-34 personality mismatch (`__gxx_personality_wasm0` vs legacy `__gxx_wasm_personality_v0`) — staged emsdk is the only green path. Falsifier satisfied: the receipt landed in `designs/`.
- M6 refusals — OUT; same falsifier.
- M9 authority — OUT; same falsifier.
- M10 ledger/cache — OUT; same falsifier.
- M11 docs-teach — OUT; same falsifier.
- M12 staleness — **FILED**: `designs/m12-staleness-receipt.md` (18:30) —
  `doc_staleness_gate.sh` 2 gate bugs fixed (R2 wrapped-historical FP,
  R3 self-fire on its own spec doc), 92→83 findings, anti-vacuity
  inject/removal re-witness (85→83), exit 1 throughout, 83 findings
  reported not edited.

Spot-verification by this lane (re-executed at use time, not recalled):
- M3 MATCH: `CARGO_TARGET_DIR=target-lane-m8 cargo test --test
  sql_rdf_dialect` → 25 passed, 0 failed, exit 0 (this lane's own run). (receipts/sql-rdf-dialect-2026-10-22.md — re-run 2026-10-22: 33 passed, 0 failed, exit 0)
- M12 MATCH: `shasum -a 256 scripts/doc_staleness_gate.sh` =
  `296812dcc06582d3…a94d0` (matches the receipt's historical sha);
  `bash scripts/doc_staleness_gate.sh` → `83 finding(s)`, exit 1;
  `docs/scratch-m12-inject.md` absent (cleanup confirmed, exit from
  `ls` = ENOENT as expected).
- OUT census MATCH: early `grep -rln 'lane M' designs/*.md` → no matches
  (exit 1) at 18:2x–18:3x, superseded by the 18:29/18:30 landings above.

This lane edited `RECEIPTS.md` only (below the wave-4 heading);
coordinator owns git.

## Wave-6 switchover sync (appended 2026-10-22 ~21:4x PDT, lane T5)

Sources verified on disk at use time (`test -f` exit 0 for each cited file).
One line per item; receipt pointer + key count; no FAIL marked closed.

- **Switchover execution + N25 fix-forward + N26 census + S2 verification +
  S8/S12 canary** — `designs/commit-receipt-provenance-gap.md` (lane L176,
  STAGED section of record): all 8 split trees WIRED with zero logged
  Write/Edit records in the wiring window; staging dirs + the only
  provenance.json DELETED via the same unlogged channel (§2); executor
  **UNDETERMINED** (§1, S2 confirms "UNDETERMINED, with escalation");
  content accepted on merits — N25 `designs/eyeron-switchover-fixforward.md`
  eyeron 21/0, sql_rdf_dialect 33/0, chicago_validation 9/0, check exit 0;
  N23/L147b `designs/precommit-acceptance-receipt.md` 124 passed / 4 failed
  (all 4 expected-known registry-drift rows, deterministic); wasm L1 9/0.
  Canary: S8 `designs/s8-unlogged-writer-watch.md` QUIESCED 20:14–20:20
  (zero changes over 31 sha-compared files); **S12 canary NOT landed** at
  provenance-gap write time — doc records it as appending when it lands;
  S8 is the witness of record. Attribution investigation OPEN. Nothing here
  is marked closed.
- **P7 six-entry receipt-surface patch applied + regen** —
  `designs/surface-six-entry-patch.md` §6 (lane P7, executed): applied per
  §1-§2 ordered patch→check→court→--write→--check→court; receipt_surface
  pin `blake3:e0215f61…` (2-entry) → `blake3:4f28d0f1…` (6-entry: 3 src
  files + sql_ast/ast.rs + refusals.rs + serializer.rs split files +
  ontologies/sql-ast.ttl); pre-write court 18 passed / 4 failed (exactly
  the committed-pin courts, expected); `graphlaw-registry --write` exit 0;
  `--check` exit 0; **post-write court 22 passed / 0 failed** [as-of-date:
  lane D2 re-audit re-ran `cargo test --test registry_artifacts` on the
  later tree → **FAILED 17 passed / 5 failed** — pin drift recurred after
  subsequent surface edits; embedded output in the lane-D2 audit section
  below;
  **[NOT-REPRODUCED 2026-10-22, lane AA11]** lane Z3 re-ran on the current
  tree: `registry_dispatch` **14 passed / 0 failed**,
  `authority_manifest` **6 passed / 0 failed**, `registry_artifacts`
  **22 passed / 0 failed** (digest-drift gates included — the D2 red did
  not reproduce). Hypothesis: D2 ran against a pre-re-pin tree, or a
  concurrent lane mutation (`scripts/mutation_court.sh`-class in-place
  edit) was in flight during D2's run. The D2 FAILED 17/5 counts above
  stand as a historical observation of that earlier state, not of the
  current tree.];
  wasm row preserved byte-identical (`332ea218…c30c`, 6793617 B);
  deviations from spec: none. This regen is the fix-forward for the
  witnessed registry_artifacts FAILED 17/5 digest-drift
  (`receipts/registry-courts-2026-10-22.md`, L167, pre-regen subject).
- **V-L18 independent regen + 22/0 — NOT FOUND on disk.** Repo-wide grep
  for `V-L18` over docs/ + receipts/ → 0 hits (exit 1); no dedicated
  receipt file exists; the only 22/0 registry_artifacts counts on disk are
  P7's §6 above and the earlier L73 regen
  (`designs/registry-regen-receipt.md:41`, 17:11, pre-switchover subject).
  Recorded as UNFILED, not claimed.
- **V-L11 refusal re-audit** — `receipts/refusal-register-audit-2026-10-22.md`
  (21:31): carriers verified against code (LawError, RefusalKind::DuckDb,
  AstError variants 16-21, `src/sql_ast.rs:495` regex); 5 dated corrections
  applied to `docs/refusals.md` + `docs/refusal-playbook.md` (anchors
  moved to `src/sql_ast/refusals.rs` post-switchover, LITERAL_NOT_INT
  message shape, AstError pure-data `{code, ctx}`, REFUSAL-04 literal
  call site); probe re-run 13/13 MATCH exit 0; **no refusal status
  flipped, register content untouched**.
- **M-wave verdicts** — `designs/m-wave-verdicts.md` (coordinator-filed
  from lane reports): M2 sql_rdf_dialect 25/0, duckdb_statement_kinds 6/0,
  sql_rdf_conformance 7/0; M6 trust_attest 10/0 + 2 register corrections;
  M9 authority_manifest 6/0 PASS no-change; M10 ledger 9/0, cache 8/0,
  evidentiary_ring featured 10/0 (bare = typed loud SKIP, expected);
  M11 positive no-change (readme_first_ten_minutes 3/0,
  docs_release_claims 2/0). M4 pending at that file's time — superseded
  by `designs/m4-wasm-boundary-receipt.md` (FILED, see Wave-M section
  above). No FAIL marked closed; M-wave rows move from UNFILED to FILED
  against `designs/m-wave-verdicts.md`.

This lane edited `docs/jira/v26.10.22/RECEIPTS.md` only (this section);
coordinator owns git.

---

# Lane D2 — receipt audit: embedded-output verification (appended 2026-10-22 late PDT)

Task: verify every receipt entry in this file embeds real command output;
re-witness the load-bearing gates on the current tree (HEAD `615dbfa`,
41 GiB free) and mark stale claims as-of-date. Nothing fabricated: every
block below is this lane's own executed command and verbatim output,
2026-10-22, cwd /Users/sac/graphlaw.

```
$ git rev-parse --short HEAD
615dbfa
$ df -h / | tail -1
/dev/disk3s1s1   926Gi    11Gi    41Gi    22%    453k  426M    0%   /
$ cargo test --test sql_rdf_dialect
test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s
$ cargo test --test chicago_validation
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.68s
$ cargo test --test trust_attest
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
$ cargo test --test sql_rdf_conformance
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
$ cargo test --test duckdb_statement_kinds
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.66s
$ cargo test --test query_ledger_court
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
$ cargo test --test query_cache_court
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
$ cargo test --features evidentiary-ring --test evidentiary_ring_court
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 13.75s
$ cargo test --test registry_artifacts
test artifacts_write_repin_matches_live_digests ... FAILED
test artifacts_check_flags_stale_owned_row ... FAILED
test b_emitter_check_flag_accepts_committed_files ... FAILED
test artifacts_pin_file_carries_the_live_registry_and_surface_digests ... FAILED
test result: FAILED. 17 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 47.11s
$ cargo test --test gws4_ipc_injection_court
test gws4_a_corpus_injected_as_linear_memory_buffer_loads_into_duckdb_eval ... FAILED
test gws4_c_fail_closed_when_corpus_exceeds_bounds ... FAILED
test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 33.44s
$ b3sum priv/graphlaw.wasm
332ea2188fafb465666e71407e8f940c5b60a15184e6ad6567bb4f723fe2c30c  priv/graphlaw.wasm
$ tail -1 receipts/CHAIN.blake3 && bash scripts/receipt_chain_sync.sh --check
chain_head e4a7597bcb28269c693db90a3efc971c5a928fa16dfff34f5a7af067a2d96783
TAMPER-DETECTED: receipts/sa2a/factory-b-v26.9.29-executable-gates.md (chain=d464dad4e0e8e1979ca8cf8b4426584f6186bfc98476da22f4d4af301a5d4aa6 actual=096cfeabead6814f24b8309e27aafaa34120291789f639356069f0009b2e4dee)
exit=1
$ ls receipts/spc-2026-10-22T173758.json receipts/spc-2026-10-22T173809.json
receipts/spc-2026-10-22T173758.json
receipts/spc-2026-10-22T173809.json
```

## Verdicts

- **GREEN (re-witnessed, counts match or supersede prior claims)**:
  sql_rdf_dialect **33/0** (matches the inline re-run notes; supersedes all
  25/0 rows), chicago_validation **9/0**, trust_attest **10/0**,
  sql_rdf_conformance **7/0** (matches the inline supersession note),
  duckdb_statement_kinds **6/0** (the `.gram` environmental exception has
  CLEARED — no failure on this host now), query_ledger_court **9/0**,
  query_cache_court **8/0** (matches the inline re-run note), ring court
  **10/0** featured (matches the inline re-run note), wasm artifact pin
  **332ea218…2c30c byte-exact match**, SPC append-only receipts present.
- **RED on the current tree (pin-drift class, needs registry regen, not a
  hand patch)**: `registry_artifacts` **17/5** — the P7 "22/0 post-write"
  claim is stale; drift recurred because surface files moved after the
  regen. Same root as the earlier L167 17/5 record.
- **RED on the current tree (same two legs as recorded)**: `gws4_ipc_
  injection_court` **1/2** — `gws4_a` buffer-load and `gws4_c` fail-closed
  legs still failing; matches the receipts/gws4 re-run note; the L1/L3
  falsifier (rebuild wasm archive with WASI_SDK_PATH) remains unmet on this
  host.
- **CHAIN divergence (new since wave 4)**: `receipt_chain_sync.sh --check`
  exits **1** with TAMPER-DETECTED on
  `receipts/sa2a/factory-b-v26.9.29-executable-gates.md` — the same file
  L31/L31-wave3 already adjudicated NO-OUTPUT ("authored, not executed").
  Chain head has advanced `2c316b02…` → `e4a7597b…` (36 lines, was 9).
  Not adjudicated here: the file is either post-baseline-edited or was
  chained in an earlier state — coordinator decision (re-baseline vs
  revert), not this lane's.
- **Audit of entries lacking embedded output (as-of law, not re-run here)**:
  the early table (rows SQLRDF-01..12, 11:38/11:40 stamps) is already
  marked as-reported; the final-table "13:0x" rows and wave-2/3/4 lane
  entries cite on-disk receipts or carry inline supersession notes — their
  counts without embedded output are treated as **as-of-date historical**,
  now anchored by this section's re-witnessed runs of the same gates.
  Not re-witnessed by this lane (no run executed): L107 js vitest 197/198,
  L103 cargo-deny/advisories, L90/L98 wasm4pm fixture legs, TC-HOT/TC-MEGA
  benches, mutation_court full sweep, spc_court re-measure. These stand
  exactly as written in their sections, per the receipt rule (embedded
  output in the cited source file = receipt; prose-only = as-of-date).
