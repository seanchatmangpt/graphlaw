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
| SQLRDF-10 | `cargo test --test trust_attest` | DONE | 11:38 | 10 passed / 0 failed |
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
| SQLRDF-01 | `cargo test --test sql_rdf_dialect` | DONE — re-executed by this lane: 25 passed / 0 failed, exit 0, 13:0x |
| SQLRDF-02 | shape-probe + hostile-court rungs (`sql_rdf_dialect` + `sql_rdf_laundering_corpus`) | DONE — dialect 25/0 (13:0x, this lane); hostile corpus 11/11 as-reported by the corpus lane (`designs/RECEIPTS.md` §2, 12:58 PDT, embedded output there) |
| SQLRDF-03 | `cargo test --test sql_rdf_dialect` | DONE — 25/0, exit 0, 13:0x (this lane; HAVING court + schema-qualified-from courts green in the run) |
| SQLRDF-04 | `cargo test --test chicago_validation` | DONE — re-executed: 9 passed / 0 failed, exit 0, 13:0x (§7.1 MERGE + BLAKE3(Canon(G_AST)\|\|P_params) receipt courts green) |
| SQLRDF-05 | `cargo test --test duckdb_statement_kinds` | DONE with standing environmental exception — re-executed: 5 passed / 1 failed, exit 1, 13:0x; sole failure is the `.gram` pin-source absence (see Known-environmental exceptions) |
| SQLRDF-06 | `docs/refusals.md` register row + `cargo test --test sql_rdf_dialect` | DONE — REFUSAL-02 row confirmed in `docs/refusals.md` (grep 13:0x, `UnimplementedStatementRefusal`); refusal court green inside dialect 25/0 |
| SQLRDF-07 | `cargo test --test chicago_validation` | DONE — re-executed: 9/0, exit 0, 13:0x (evasion set + join/refusal-03 courts green in the run) |
| SQLRDF-08 | wasm boundary build + `cargo test -p graphlaw-wasm` | DONE — host contract re-executed: 7/0, exit 0, 13:0x; boundary build re-executed green lane-isolated (`CARGO_TARGET_DIR=target-lane-docs`, `wasm` profile 4m25s — shared `target/wasm` unusable, see environment notes); artifact re-pinned, digest verified (Successor §2) |
| SQLRDF-09 | `sql_rdf_dialect` + `chicago_validation` | DONE — re-executed: 25/0 + 9/0, 13:0x; `H_query = BLAKE3(Canon(G_AST) \|\| P_params)` via `purrdf::try_canonicalize`, incl. the 5-variant cosmetic-invariance set (receipt identical across comment/whitespace/alias variants) |
| SQLRDF-10 | `cargo test --test trust_attest` | DONE — re-executed: 10 passed / 0 failed, exit 0, 0.06s, 13:0x |
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

`Cargo.lock:1182-1186` — `name = "frozen-duckdb"`, `version = "2.0.0"`, crates.io registry source, `checksum = "f9eae23d5949bbab32c26cfebc1784239c52b1bb0c95d33fed63f1b04db66399"`. The path/checkout pin is gone. `Cargo.toml:35` — `frozen-duckdb = { version = "2.0", optional = true }`. Authority gate (trust_attest, previously 10/0 at 12:24): claim is **as-of-date** — a re-run at append time was compile-blocked by another lane's in-flight `src/` edits (see §6), not re-witnessed by this lane.

## 2. wasm artifact re-pin — 332ea218 (re-executed at append time)

```
$ blake3sum priv/graphlaw.wasm; cat priv/graphlaw.wasm.blake3
332ea2188fafb465666e71407e8f940c5b60a15184e6ad6567bb4f723fe2c30c  priv/graphlaw.wasm
332ea2188fafb465666e71407e8f940c5b60a15184e6ad6567bb4f723fe2c30c  graphlaw.wasm
```

Both digests identical: artifact matches pin `332ea218…2c30c`. Negative control (tampered artifact → `graphlaw-run` refusal, exit 84) is documented at `src/bin/graphlaw_run.rs:47,56,177`; the runner binary was not rebuilt by this lane (cargo lock held by in-flight lib edits), so the exit-84 run itself is **as-of-date / not re-witnessed here**.

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
  (`designs/RECEIPTS.md` §4 had recorded 4/5 PARTIAL at 12:58 — the flip-fixture defect has since been repaired; 5/5 supersedes that entry.)

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
- **Query ledger landed** — `src/query_ledger.rs` (35 KB, mtime 13:52): F1-F7 + format court. Re-executed: `cargo test --test query_ledger_court` → 9 passed; 0 failed (0.16s); lib-side format courts add 3/0 (11 lines-format total per the landing receipt).
- **Ring→ledger join landed** — `joint_ring_to_query_ledger_court` witnessed in this lane's re-run (6/0 for the feature-gated ring court file, 5.14s): max push **292 ns** against the 10 µs fence, drop gaps persisted to the ledger exactly, verify passes; shutdown-always-drains fixed. Output tail:
  ```
  test no_blocking_property_1m_pushes_vs_stalled_drainer ... ok
  test joint_ring_to_query_ledger_court ... ok
  test result: ok. 6 passed; 0 failed; ... finished in 5.14s
  ```
  (Dispatch cited 5/0 for the ring court alone; the joint court raises the file to 6/0 as observed.)
- **C1 cache store landed** — `src/query_cache.rs` (epoch-bounded, F1-F5, receipt law untouched). Re-executed: `cargo test --test query_cache_court` → 7 passed; 0 failed (0.08s).
- **Prepared-statement seam landed** — LRU 256; parameters are `$N`-bound so the same digest+params hits the cached handle.
- **Receipt-surface digest pin landed (then went stale — loud)** — `tests/registry_artifacts.rs` carried the pin at **20/0** at landing, but at 13:5x this lane's re-run is **16 passed / 4 failed** (3.65s): `receipt_surface_pin_matches_live_files_and_refuses_drift` and `artifacts_pin_file_carries_the_live_registry_and_surface_digests` fail with digest drift (`blake3:dcd46bf6…` pinned vs `blake3:049add3e…` live), plus the committed-vs-emission pair. Cause: surface edits (`src/query_ledger.rs` mtime 13:52) postdate the pin — stale-by-default per the count-recall law; needs a registry regeneration, not a hand patch. Filed as a follow-up, not absorbed into any PASS claim.
- **Measure baseline** (dispatch-supplied, bench-class): fold path **184-216 ns** inside TC-HOT-01's 250 ns budget; full-rehash path **11.6x** over budget = the §3.1.1 fallback to close.
- Court files on disk verified at append: `tests/evidentiary_ring_court.rs` (13:45), `tests/p28_hot_path_bench.rs` (13:40), `tests/query_ledger_court.rs` (13:51), `tests/query_cache_court.rs` (13:46), `tests/registry_artifacts.rs` (13:39).

## 9. Never-block fence conversion + restart watermark (as-of 2026-10-22 14:0x PDT, appended 14:1x)

- **Never-block fence conversion landed** (as-of 14:0x): the single absolute "max push <10µs" dev-profile bound is now a two-leg fence per `designs/cold-path-ledger.md` § "Never-block fence conversion (landed)": **dev leg** = ratio fence (stalled max push ≤ 10× unstalled same-binary baseline, baseline floored at 100ns so a sub-instruction baseline cannot make the ratio vacuous — the PROPERTY, load-independent), `no_blocking_property_1m_pushes_vs_stalled_drainer` (`tests/evidentiary_ring_court.rs:158`); **release leg** = absolute stalled max <10µs, `no_blocking_release_absolute_bound` (`tests/evidentiary_ring_court.rs:302`), witnessed 1.584µs < 10µs (single-shot, darwin 25.2.0; embedded output in `designs/cold-path-ledger.md:91-95`).
- **Restart-watermark fix landed** (fuzz court case 4 — crash-restart had silently renumbered onto the persisted ledger head, laundering crash loss as a complete ledger): the drainer (`src/evidence_ring.rs:473-527`) extracts a restart watermark from the resumed NDJSON ledger's last line and opens via `QueryLedger::open_with_producer` (`src/query_ledger.rs:498`) with `restart_seq.max(producer_attempted)`; crash loss persists as gap markers with `complete: false`. **Deviation, receipt-cited**: the watermark is `restart_seq` (resumed NDJSON coverage), not the prescribed live `seq` counter — the live-counter form double-covered 911 buffered sequences (joint-court gap coverage exceeded `dropped_total` by exactly 911; witness `src/evidence_ring.rs:505-512`). Regression court `restart_mid_stream_persists_gap_not_silent_renumber` (`tests/evidentiary_ring_court.rs:329`), receipt embedded at `designs/cold-path-ledger.md:74-76` (7 passed / 0 failed, 5.74s).
