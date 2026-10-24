# Wasm Gap Register

Consolidated gap register for milestone v26.10.23, derived from
`docs/jira/v26.10.23/ROADMAP.md` and the approved plan
`~/.claude/plans/here-is-the-voice-golden-bengio.md`. Status is as of
register authoring (pre-Wave A); flips are receipt-gated — a row flips
only on a landed receipt.

| ID | Description | Evidence locus | Wave | Status | Closing witness |
|---|---|---|---|---|---|
| G1 | receipt_v2 has no emission site | attest.rs:704 | D | OPEN | Twin A; v2 e2e court |
| G2 | CanonArrow DCE'd in-wasm | egress/arrow.rs | D | OPEN | Twin B; pinned-view; FIBO |
| G3 | conftest hardcode; no env honor | conftest.py:35 | B | OPEN | pytest, env set AND unset |
| G4 | read_frame unbounded alloc | run_eh.rs:336-350 | C | OPEN | bound court: exit 92 |
| G5 | WALL vs MAX_REQUEST drift | abi.rs:48; abi_meta:29 | C | OPEN | limits table; assert |
| G6 | benches unreceipted | no receipts/ files | E | PENDING | bench vs REQ budgets |
| G7 | abi builders uncovered | src/abi.rs | B | OPEN | builders court; mut rerun |
| G8 | sniff literal/IRI gaps | sniff.rs:252-279 | C | OPEN | twins; corpus_conformance |
| D1 | WASI inventory absent | SECURITY.md | A | OPEN | count receipt; gate |
| D2 | 256 MiB doc errors | WASM.md:206 | A + C | OPEN | delta; pointer rows |
| D3 | GWS stale; INDEX orphans | integration.md:49 | A | OPEN | appends; GWS-2/3 flip |
| D4 | witnesses not durable | terminal receipt 2 | A | OPEN | rerun on 472d5557 |
| BB1 | blackbox pins wasmi | tests_py blackbox | F | OPEN | host flip; tree -i |
| UU1 | unknown-unknown smoke | audit, deferred | E | PENDING | battery re-run |
| W08 | differential no receipt | common/mod.rs:20 | C | OPEN | differential receipt |
| SEC1 | ATTACH allowlist bypass: whitespace-only leading-token classifier skipped the allowlist on comment-camouflaged ATTACH (`/* */`, `--`, `/*!*/`) | duckdb_op.rs `leading_sql_token` (:235) + `enforce_attach_policy_with` (:437 as of read); seam courts `src/duckdb_op.rs::tests`; wire court `tests/attach_bypass_court.rs` | SEC | OPEN | green-court receipt for the hardened classifier (W15 `2026-10-23-eh-closure-wave.md:59` predates the hardening) |
| INC1 | tree-revert incident: verified security fixes destroyed in-tree (~00:48-00:56 Oct 24), re-landed as `679299d` + `907e663` | GO-NO-GO-v26.10.22.md §1.1-1.2 (v3, incident note) | SEC | OPEN | re-land + green courts + incident note (re-land and note witnessed; green courts receipt pending) |

Row count: 17 (G1-G8, D1-D4, BB1, UU1, W08, SEC1, INC1).

Behavioral-audit deferred witness classes: G1 = emission-site court (Twin
A), G2 = columnar-bytes twins (Twin B + pinned-view), G3 = env-matrix
pytest (set/unset), BB1 = host-flip green + absence-from-graph witness,
UU1 = full-battery re-run, W08 = run+receipt against the pinned artifact.

## Flip log

### 2026-10-24 — security-wave landing observed; flips held on the receipt gate (register lane, write lease = this file only)

Gate: the security-wave fixes landed and are committed — `679299d`
(`leading_sql_token` scanner, 00:56:43) and `907e663` (EH-host frame-bound
mirror + ATTACH unit courts, 01:09:56). This register lane ran seven bounded
poll windows (00:59-01:33 PDT) watching the landing markers
(`leading_sql_token`, `RunError::FrameOversize`) and receipts. **No durable
green-court receipt for the security-wave courts had landed by authoring**
(GO-NO-GO v3 read-close 01:15 independently records "no landed green
receipt"; the courts were witnessed mid-run at 01:21 via process table), so
**zero status flips this session** — the register's gate is "a row flips
only on a landed receipt". The flips fire on the receipt; each names its
exact pending gate:

- **G4 (read_frame unbounded alloc) — flip held, stays OPEN.** The fix is
  verified on disk by this lane: both hosts bound the untrusted 4-byte
  prefix BEFORE any body allocation, cap single-sourced from
  `graphlaw::abi::MAX_REQUEST_BYTES` (no local duplicate), coherence asserts
  vs the 64 MiB wall, exit-92 rows in both `code()` tables, len+cap
  diagnostics, mirrored `read_frame` bodies. Loci (as of read):
  `src/bin/graphlaw_run.rs:105,123,152,176,484-491`;
  `src/bin/graphlaw_run_eh.rs:40,70,97,123-125,407-420`. Design:
  `docs/jira/v26.10.23/design-read-frame-bound.md` (SPEC now implemented).
  Pending gate: a receipt embedding `tests/wasm_frame_bound_court.rs` output
  (at-cap admitted end to end / cap+1 → exit 92 / 0xFFFFFFFF → exit 92 with
  length+cap+no-reply, zero-len → exit 0, both hosts). The court file is
  landed; its green receipt is not.
- **SEC1 — new row, OPEN.** Scanner landed: `leading_sql_token`
  (`src/duckdb_op.rs:235`, comment-camouflage-immune: whitespace, `--`
  line comments, nesting-aware `/* */`, quote-aware stop), consumed by
  `enforce_attach_policy_with` (:437); seam courts :1336+
  (`attach_behind_block_comment_hits_policy` etc.); wire court
  `tests/attach_bypass_court.rs` (typed DuckDb refusal envelope + liveness
  corpus + document-member gate). Pending gate: a receipt embedding the
  seam + wire court output on the HARDENED classifier — W15
  (`receipts/2026-10-23-eh-closure-wave.md:59`, 13/0) predates the
  hardening edit (GO-NO-GO v2 §2), so it cannot witness this closure.
- **INC1 — new row, OPEN.** Fixes destroyed post-verification: the
  v2-era working-tree hardening was reverted out of the tree ~00:48-00:56
  (GO-NO-GO v3 §1.1 "present→absent→present"); the mc4 mutation-court
  exit-137 leak put the json_depth shim into `b4283a3` (recovered by
  `16678de`). Re-land: `679299d` + `907e663` — witnessed by this lane's
  poll (wasmi bound 00:54:50, scanner 00:57:23, EH plumbing 01:00:47, EH
  body check by 01:10, seam courts + frame court file 01:02-01:18) and
  GO-NO-GO §1.1 live grep counts. Incident note: GO-NO-GO v3 §1.1-1.2.
  Third closure condition (green courts receipt) pending — same gate as
  G4/SEC1.

Falsifiers for this flip-log entry: (a) a durable receipt embedding the
frame-bound court output lands → G4 flips OPEN → CLOSED in a new append;
(b) a durable receipt embedding the attach seam+wire court output lands →
SEC1 flips; (c) both plus the incident-note closure triad → INC1 flips.
No existing row was rewritten; row count 15 → 17 reflects appends only.

Addendum 01:55 PDT — red witness on the re-landed tree. The coordinator's
`cargo test --test graphlaw_run_eh_court -- --test-threads=1` (log
`/tmp/coord_t3.log`, finished 01:52) returned **FAILED, 6 passed / 1
failed**: `pending_artifact_pin_capabilities_frame_exit97_uninitialized_element`
expected exit 97, observed 98 — the C0c fuel metering (landed `907e663`)
exhausted its 10,000,000 budget on the pinned-artifact capabilities leg
("guest exhausted fuel budget 10000000"). The re-land is therefore not
green as of this addendum; the G4/SEC1/INC1 gates (green-court receipts)
remain unmet on current evidence, and this conflict (fuel budget vs the
pinned-artifact leg) must be resolved before any flip.

### 2026-10-24 03:20 PDT — green-court receipts landed; G4/SEC1/INC1 flip OPEN → CLOSED (register lane, write lease = this file only)

Gate: all courts below ran on the settled tree at HEAD
`4df19b5b1d2480d50851e7c0e0bf855802a2c46a` (working tree clean at every
cited locus — tested tree == committed tree). Exit codes captured via zsh
`${pipestatus[1]}`, never a pipe tail.

- **G4 → CLOSED.** Receipt:
  `GRAPHLAW_WASM=priv/graphlaw.wasm GRAPHLAW_RUN_EH=target-coord/debug/graphlaw-run-eh CARGO_TARGET_DIR=target-coord cargo test --test wasm_frame_bound_court`
  → `running 8 tests` / `test result: ok. 8 passed; 0 failed; 0 ignored;
  0 measured; 0 filtered out; finished in 0.36s`, exit 0. Discharges the
  pending gate named in the 01:33 entry: at-cap frame admitted and answered
  end to end (both hosts), cap+1 → exit 92, 0xFFFFFFFF → exit 92 before
  body, zero-len → exit 0 — `wasmi_at_cap_frame_admitted_and_answered`,
  `eh_at_cap_frame_admitted_and_answered`, `wasmi_over_cap_by_one_exit92`,
  `eh_over_cap_by_one_exit92`, `wasmi_oversize_0xffffffff_exit92_before_body`,
  `eh_oversize_0xffffffff_exit92_before_body`, `wasmi_zero_len_frame_clean_exit0`,
  `eh_zero_len_frame_clean_exit0`.
- **SEC1 → CLOSED.** Receipt:
  `CARGO_TARGET_DIR=target-coord cargo test --test attach_bypass_court`
  → `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured;
  0 filtered out; finished in 0.40s`, exit 0. Tests:
  `camouflage_attack_strings_are_engine_live_attach_statements` (the
  `/* */` / `--` / `/*!*/` camouflaged strings proven engine-live ATTACH),
  `wire_fail_closed_refusal_then_inside_root_attach_executes` (typed
  refusal envelope + inside-root liveness),
  `wire_document_member_attach_refuses_typed`. This is the seam+wire
  output on the HARDENED classifier — the W15 predates-hardening objection
  is discharged.
- **INC1 → CLOSED.** Triad complete: (1) re-land `679299d` + `907e663`
  (witnessed by the prior entry's poll); (2) green courts = the G4 + SEC1
  receipts above; (3) the 01:55 addendum's blocking red witness resolved by
  rerun on the settled tree:
  `GRAPHLAW_WASM=priv/graphlaw.wasm CARGO_TARGET_DIR=target-coord cargo test --test graphlaw_run_eh_court -- --test-threads=1`
  → `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured;
  0 filtered out; finished in 21.23s`, exit 0 —
  `pending_artifact_pin_capabilities_frame_exit97_uninitialized_element`
  now exits 97 as specified; the 01:52 exit-98 fuel exhaustion no longer
  reproduces. Note: without `GRAPHLAW_WASM` set this court loud-SKIPs
  (`GRAPH LAW EH-COURT SKIP: no wasm artifact resolved`, exit 101); that
  skip is typed and was not counted as a witness.

Supporting runs, same window (not flips):
`cargo test --test abi_response_builders_court` → `test result: ok. 10
passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in
0.00s`, exit 0 — **G7 stays OPEN**: its named closing witness is "builders
court; mut rerun" and the mutation-court rerun is still owed.
`cargo test --test wasm_env_isolation_court` → `test result: ok. 3 passed;
0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 162.69s`,
exit 0 (two tests polled >60s under bounded observable polling; positive
control `checker_positive_control_detects_a_real_env_echo` passed, so the
court is non-vacuous).

Reopen conditions: any court above returning red on the same loci, or a
mutation of the `read_frame` bound / `leading_sql_token` classifier that
its court fails to invert on, reopens the row via a new append. Row count
unchanged (17); this entry appends only — no table row or prior log entry
rewritten.
