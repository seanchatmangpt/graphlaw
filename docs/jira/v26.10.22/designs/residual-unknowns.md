# Residual unknowns — honest list (Lane T3, DOC) — 2026-10-22

Scope: what is still UNKNOWN / BLOCKED / UNVERIFIED on this tree after ~5,000
lane-receipts of work today. Lease: this file only. Each row: claim, why open,
what closes it. Nothing here is a pass; an absence of red is not a green.

## 1. EH wall — duckdb op host-only; in-wasm SQL execution awaits emsdk rebuild
- **STATUS FLIP 2026-10-23**: RESOLVED 2026-10-23: EH wall closed by uniform
  new-EH rebuild + wasmtime 48 host; residual: duckdb-open execution trap under
  diagnosis (see `eh-runner-decision-memo.md` DECISION block).
- **Claim**: the DuckDB-linked `graphlaw-wasm` engine artifact (mixed legacy+new
  EH) is hostable by NO staged runtime (wasmi 2.0.0 no exceptions knob; node 26
  rejects; wasmtime legacy_exceptions fails). In-wasm SQL execution does not run.
- **Why open**: OD-1 is recorded with recommendation "C now + A1' engine
  backend" (`designs/OPERATOR-DECISIONS.md:244`) but the decision line is a
  fill-in; the lane that wrote it decided nothing. A1' is probe-witnessed only
  (`designs/a1prime-trytable-recipe.md:9-29`), not a full duckdb rebuild; R2b
  (runtime .a members may still carry legacy opcodes) and R2c (rustc `-Z
  llvm-args` on stable) are UNVERIFIABLE-LOCALLY (`a1prime-trytable-recipe.md:120-127`).
- **Closes it**: operator fills the OD-1 decision line; full emsdk rebuild of
  frozen-duckdb per `dual-artifact-policy-design.md` §4 step 1; a witnessed
  real module load + one real eval frame on node or modern wasmtime, 3x.

## 2. Mutation court full sweep — attempt ran, incomplete (DISK-BLOCKED)
- **Claim**: a full-sweep attempt ran (T1,
  `designs/t1-mutation-full-receipt.md`): unmutated baseline ok (72s build +
  34s test), 88 mutants inventoried (not the 319 cited earlier), first verdict
  **MISSED** at `src/abi.rs:152:5` (replacing `limit_response -> Vec<u8>`
  body with `vec![]` survives the suite). The sweep did NOT complete —
  DISK-BLOCKED (DISK-REFUSED at 17.3 → 4.8 GiB free while sibling lanes
  built). One executed mutant is a genuine red, not a completed score.
- **Why open**: no completed sweep verdict exists on this tree; the earlier
  dirty-tree refusal (`designs/mutation-court-full-receipt.md`) was replaced
  by this incomplete run, which is still not a result. Runtime estimate
  ~3-4.5 h for 88 mutants.
- **Closes it**: a full `scripts/mutation_court.sh` sweep to completion with
  kill counts and runtime; the `src/abi.rs:152:5` MISSED either killed by a
  test or accepted with operator sign-off.

## 3. cargo-mutants sweep incomplete (disk-blocked, one MISSED verdict)
- **Claim**: no `cargo-mutants` sweep has produced a COMPLETED verdict on this
  tree. The T1 attempt (`t1-mutation-full-receipt.md`) executed 1 of 88
  mutants before DISK-BLOCKED; that one verdict is MISSED
  (`src/abi.rs:152:5`).
- **Why open**: the sweep is a real channel now (baseline ran, mutants
  execute), but it has never finished — disk pressure under concurrent lanes
  killed it. An incomplete sweep supports no "mutation score" claim.
- **Closes it**: full sweep to completion on a window with ≥20 GiB free and
  enough runtime (~3-4.5 h); receipt with kill counts.

## 4. TC-HOT bench gate FAIL under load variance — CLOSED as measured, OPEN as performance question
- **Claim**: CLOSED (post-edit): the 10M-iteration full bench was re-run after
  today's edits with `TC_HOT_FULL=1` and a fresh SPC receipt landed —
  `designs/tc-hot-01-errc-receipt.md` + `receipts/spc-2026-10-22T191548.json`.
  The numbers are FRESH honest-FAIL, not stale: clone-path p99 209 ns meets
  the 250 ns bound and the refusal invariant passes, but the combined gate
  exits 1 because batch-p50 variance is 26.91% against a 5% budget
  (3x33/3M split: [125, 166, 166] ns; the 3x333k control split passes at
  0.60%), and the SPC smoke exits 1 (Cpk 0.001 vs 1.33; mean 185.686 ns,
  p99 inside USL; Shewhart rule 1 flagged 27 samples).
- **Why open**: as a performance question — the miss is real load variance on
  this shared tree, not measurement staleness; whether hot-path latency holds
  5% variance on a quiet machine is untested. TC-HOT-02 has no gate (the old
  bench was removed; `target-lane92/tc-hot-25m` absent).
- **Closes it**: quiet-machine re-run with variance under gate (or a variance
  gate that is load-robust); SPC refresh from that run; decide the TC-HOT-02
  gate question explicitly.

## 5. js publish flow untested end-to-end
- **Claim**: `js/npm` publish (sync-version → build → pack → publish) has
  never been executed end-to-end in this cycle.
- **Why open**: receipts cover generated-artifact sync and tests
  (`designs/js-gen-receipt.md`, `js-test-receipt.md`); no receipt shows
  `npm pack` contents inspection or a real publish (or dry-run publish)
  against a registry.
- **Closes it**: `npm pack` + inspect tarball against a manifest; then
  `npm publish --dry-run` against the real package.json; receipt both steps.

## 6. wasm4pm oracle legs (fuzz / memory-wall) BLOCKED on EH
- **Claim**: cross-repo oracle gates cannot run.
- **Why open**: two independent blockers. (a) EH wall (row 1) — oracle legs
  that need the engine artifact hosted have no host. (b) FILE_MISSING —
  `/Users/sac/wasm4pm/tests/graphlaw_vector_oracle.rs` does not exist on disk;
  L106 gate recorded BLOCKED(FILE_MISSING)
  (`designs/gws4-postlink-receipt.md` cross-repo section;
  `l98-wasm4pm-vecgen-provenance.md`).
- **Closes it**: land naive-physics fixtures in wasm4pm, then A1' host
  witness; fuzz and memory-wall legs then run for real.

## 7. gws4 court — stale contract CLOSED (S1 landed); in-wasm SQL parse sub-claim remains
- **Stale-contract half CLOSED**: the court was written against a contract the
  tree no longer has (it expected `duckdb` dispatch in-wasm). S1 rewrote it to
  the real boundary contract: the duckdb op is HOST-ONLY (`src/abi.rs`
  `#[cfg(not(target_arch = "wasm32"))]` on the `"duckdb"` dispatch arm), so
  in-wasm `{"op":"duckdb"}` fail-closes with a typed `Unsupported` /
  `unknown op \`duckdb\`` refusal — asserted directly
  (`tests/gws4_ipc_injection_court.rs`, gws4_a leg (a')). The corpus→
  duckdb_eval injection leg now runs on the HOST dispatch path
  (`graphlaw::abi::call_json`, op-15 exec with a `.csv` view mount, cursor +
  receipt readback). Court is **3/0 green**:
  `test result: ok. 3 passed; 0 failed ... finished in 53.96s`
  (`cargo test --test gws4_ipc_injection_court`). Receipt = the diff to
  `tests/gws4_ipc_injection_court.rs` (+64/−18) plus that run output.
- **Sub-claim still OPEN (300-quad in-wasm parse/eval)**: what S1 witnessed in
  the in-wasm leg is that the corpus bytes cross the *compiled* module's
  linear-memory boundary and the module answers TYPED (refusal, not crash) —
  i.e. module load, ABI injection, and fail-closed dispatch are witnessed.
  It does NOT witness SQL admission or duckdb evaluation *inside* wasm: the
  duckdb op is refused before any parse. That execution capability is exactly
  row 1 (EH wall) and stays open until the emsdk rebuild + host witness.
- **Why open (remainder)**: same EH wall (row 1); this row now tracks only
  the in-wasm-execution sub-claim, not the court's own health.
- **Closes it**: A1' rebuild + host witness (row 1); then an in-wasm leg that
  asserts a real eval frame (rows back through the boundary), not a refusal.

## 8. Additional open rows verified this session
- **N2-N6, N8 receipts UNFILED (loud)**: no lane report files in `designs/`;
  falsifier is their landing. Closes: receipt files land or lanes are typed
  dead.
- **smalt pairing 3/7**: four red gates typed as unhostable-pair refusals
  (`designs/smalt-pairing-receipt.md`); stays 3/7 until A1' lands
  (`dual-artifact-policy-design.md:142`). Closes: A1' + pairing re-run to 7/7.
- **Gate backlogs are live, not green**: `doc_staleness_gate.sh` exit 1
  (anchor R5 backlog: 1173 rows post-mega-corpus; L168's rule changes cut
  findings 100 → 76 via Cargo.lock/CHAIN/R3-allowlist/R4-block-pointer admits
  — `designs/gate-rule-changes-receipt.md` — with flip witnesses both
  directions), `witness_gate.sh` exit 1 (190 findings), `count_gate.sh` exit 1
  (106 stale rows) — recorded as remediation queues in RECEIPTS.md tail; the
  backlogs themselves are unresolved. Closes: remediate rows; gates must then
  still refuse correctly on injected defects.
- **wasm-engine registry row PIN-ONLY**: "no EH host witnessed; see A1'"
  (`dual-artifact-policy-design.md:46`). A1's mechanism premise is now
  FALSIFIED, not merely unwitnessed: L122
  (`designs/wasm-repin-receipt.md`) clean SDK-34 rebuilds of duckdb + full
  link still carry exactly one wasm Tag (id 13) section — wasmi 2.0.0 rejects
  both artifacts (`exceptions proposal not enabled`, 0/2 on
  `wasm_native_differential`) — so "rebuild without tags" is not reachable by
  the pinned toolchain. EH wall OPEN with corrected mechanism: closure
  requires a tags-capable host (wasmtime EH / node / wasmi knob), not an
  emsdk tag-free rebuild. Closes: host witness on any current artifact →
  re-pin via the wasm rebuild flow only.
- **M17 bins gate UNFILED**: witnessed by a receipt-section run only, no
  dedicated receipt file (`RECEIPTS.md` Wave M). Closes: dedicated receipt or
  explicit waiver.

## Non-rows (explicitly closed today, listed to prevent re-opening)
- Cross-authority court 71/0 re-witnessed against real frozen-duckdb
  (RECEIPTS.md tail, count-pinned).
- M16 kill-discipline doctrine landed; M18 fleet-cleanup landed (68.69 GiB
  reclaimable witnessed).

Standing: CONFORMANT-AS-A-DISCLOSURE — this file enumerates unknowns; it
proves nothing beyond that the unknowns were stated. Falsifier: any row above
closes silently (no receipt, no re-run) and stays listed.
