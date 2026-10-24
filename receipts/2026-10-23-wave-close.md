# Wave-close receipt — 0d11bc3 integration commit (2026-10-23)

- Subject: `/Users/sac/graphlaw` @ main, HEAD `0d11bc3`
  (`feat(v26.10.22): sovereign in-WASM DuckDB — EH closure, sequences, SQLRDF-21..25`).
- Companion receipts: `receipts/2026-10-23-eh-closure-wave.md` (wave detail),
  `receipts/2026-10-23-chain-rebaseline-note.md`,
  `receipts/2026-10-23-staging-wipe-incident.md`.
- Ledger: `docs/jira/v26.10.22/RECEIPTS.md` (W1–W58 rows);
  lane leases: `docs/jira/v26.10.22/_LANES.md` (EH-01..EH-32 table).

## 1. Embedded real output (commands run by this lane at receipt time)

`git log --oneline -3`:

```
0d11bc3 feat(v26.10.22): sovereign in-WASM DuckDB — EH closure, sequences, SQLRDF-21..25
dab6a73 chore(snapshot): fleet wave safety snapshot — 2026-10-22 annealing waves W/Z/AA/AB/K
615dbfa chore(tooling): dialect pack, P28-SIL courts, evidentiary ring, ledger
```

`b3sum receipts/2026-10-23-eh-closure-wave.md`:

```
933a6e46c070f98d663c45101437560d43283bf021cd0396c4593199b8d0e504  /Users/sac/graphlaw/receipts/2026-10-23-eh-closure-wave.md
```

(witnessed file length: 164 lines, `wc -l` at receipt time).

## 2. Landed surfaces in 0d11bc3

Per the commit subject and the witnessed ledger rows cited in
`receipts/2026-10-23-eh-closure-wave.md` (§2, §4, §5, §8):

- **EH closure**: opcode EH gate try_table=99 / legacy=0 on the rebuilt
  frozen-duckdb archive (`-fwasm-exceptions -mllvm -wasm-use-legacy-eh=0`,
  pin `b6013f7aa2`); wasmtime 48 hosting wall closed (EH-OK probe exit 0);
  SQLRDF-24 dtor trap root-caused and fixed (wasi-libc `atexit.c.obj` swap +
  reactor-contract completion in `wasm/src/lib.rs`); canonical re-pin LANDED:
  `priv/graphlaw.wasm` blake3
  `fa1558306690493249b176ec312a4f01e97798937e05cd49f2946d61a7e6d107`,
  57,781,384 B, Tag(13)×1, registry `--check` exit 0.
- **SQLRDF-21..25**: document-sequence wire surface verified end to end
  (duckdb_op 16/0, duckdb_eval 18/0, sql_ast 32/0; session ABI 9/9);
  REFUSAL-05 register row + pins 36/0; G5 arms
  MIXED_DOCUMENT_ROOTS / INDEX_SEQUENCE_DISORDER / SEQUENCE_EMPTY flipped
  PINNED_GAPS→REGISTERED_ARMS at `docs/refusals.md:96-110` (8/0);
  SQLRDF-25 deferred-gap ledger landed
  (`docs/jira/v26.10.22/SQLRDF-25-remaining-gaps.md` — 5 open rows, status
  Open).
- **Banking courts**: MERGE mutating-classification court 12/0 with
  anti-vacuity flip; G6 ATTACH allowlist 13/0; wired-op court landed
  (413-line rewrite); G1 DECIMAL 3 courts + G4 multi-column 6 courts folded
  into sql_ast 32/0.
- **Benches**: tc_hot_02 bench restore via the coordinator manifest batch
  (`benches/sql_rdf_admission.rs`, `benches/sql_rdf_egress.rs`,
  `benches/sql_rdf_engine.rs` in the committed diff).
- **Docs**: how-to run-the-wasm-module, DIALECTS session +
  document-sequence sections, SQLRDF-21/22/23/24 progress, refusals rows
  96-110 + 119-121 + 124, CHANGELOG/MIGRATION, WASM boundary docs
  (`docs/WASM.md`, `docs/how-to/`, `docs/wasm-*.md` in the committed diff).
- **Receipt chain**: re-baselined + VERIFIED (773 receipts, head `d4447281…`;
  second re-baseline 840 receipts, head `c2d79ff2…` at W57).

## 3. Still-open rows (owners per `_LANES.md`; counts as witnessed)

| Row | Count | Owner lane | Evidence locus |
|---|---|---|---|
| tc08 COPY egress EINVAL | failing leg in `tests/wasm_fibo_fixtures_eh.rs` (DIAG-NOTE(tc08) bisect toggle live at line 108) | EH-19 (FIBO fixtures) | `tests/wasm_fibo_fixtures_eh.rs:108`, `:515` |
| `graphlaw-run-eh` stdin hang — smalt real-runner court | 3/7 (gates 1/1b/1c/3 fail) | EH-08 (wasmtime host bin / smalt pairing) | `docs/jira/v26.10.22/designs/eh-runner-decision-memo.md:18` (W7 row) |
| FIBO fixtures court | 3/4 | EH-19 (FIBO fixtures) | `receipts/2026-10-23-eh-closure-wave.md` §8 (final run vs final pin in flight at receipt time) |
| gws4 IPC-injection court | 1/2 (`gws4_a` buffer-load + `gws4_c` fail-closed legs failing) | EH-11 (gws4 court); predecessor L3 | `docs/jira/v26.10.22/RECEIPTS.md:497`, `:881-883`; typed BLOCKED on WASI_SDK_PATH archive rebuild (`_LANES.md` "In flight") |

No receipt, no flip: these rows close only against a witnessed re-run on a
descendant head. The FIBO final run vs final pin and the pm4pytest wasm-leg
final rerun were in flight at the companion receipt's time and remain open
here.

## 4. Standing

Landed surfaces: CONFORMANT-ON-OBSERVED-OUTPUT on the witnessed court rows
cited in §2. Open rows in §3: OPEN with named owners; nothing here rewrites
a status without a receipt. The coordinator runs `receipt_chain_sync` and
owns the git transition; this receipt is evidence, not authority.
