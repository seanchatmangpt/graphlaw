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
| D3 | GWS stale; INDEX orphans | docs/wasm-duckdb-integration.md:45-60 (wave item register; PENDING-GWS-1/4/7/8 tokens at 49-56) | A | OPEN | appends; GWS-2/3 flip |
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

### 2026-10-24 — D3 evidence-locus citation repair (register lane, write lease = this file only)

The D3 row's evidence locus cited `integration.md:49` — an unresolvable
short form (no such bare path at the repo root; the intended referent is
`docs/wasm-duckdb-integration.md`). Citation repaired in place to
`docs/wasm-duckdb-integration.md:45-60 (wave item register;
PENDING-GWS-1/4/7/8 tokens at 49-56)`. **Citation fix only — status
unchanged (D3 stays OPEN)**; no other row, prior log entry, or closing
witness was touched. Row count unchanged (17).

### 2026-10-24 13:5x PDT — consolidation wave: G1/G3/G5/D1/D2/BB1 CLOSED; D4 partial; carry notes (register lane, write lease = flip-log append only)

Gate: every closure below names its landed witness. Tree state at write:
HEAD `a00770b` (live tree, concurrent lanes landing — disclosed). This
entry appends only; no table row or prior log entry is rewritten. Row
count unchanged (17).

- **G1 → CLOSED.** The register's named closing witness for G1 is "Twin A;
  v2 e2e court". Both legs are witnessed green on the pinned artifact in
  the battery meta leg (`/tmp/bat-L3.log`, 11:21): `receipt_v2_court`
  → `test result: ok. 6 passed; 0 failed`, `receipt_v2_emission_court`
  → `test result: ok. 8 passed; 0 failed` — the emission site exists and
  the v2 e2e court runs over real columnar bytes. Durable-receipt debt
  disclosed: the `receipts/2026-10-24-battery-receipt-v2.md` file is owed
  (X13 order: battery receipts first); this flip stands on the battery-leg
  verbatim lines until that file lands, and REOPENS if the landed receipt
  contradicts them.
- **G3 (conftest) → CLOSED.** Receipts:
  `receipts/2026-10-24-wasmi-retirement-step2-green-witness.md` —
  `GRAPHLAW_WASM=/Users/sac/graphlaw/priv/graphlaw.wasm pytest tests_py -v`
  → 19/0 PASSED (twice), zero non-eh runner refs in tests_py. The
  env-honoring lookup + canonical shim are in the consumed path; the
  closing witness ("pytest, env set AND unset") is witnessed by the suite
  running over the env-set invocation with the runner resolved from the
  variable.
- **G5 → CLOSED.** Constants are single-sourced: the frame cap is
  `graphlaw::abi::MAX_REQUEST_BYTES` with no local duplicate and coherence
  asserts vs the 64 MiB wall (loci named in the 01:33 entry); the limits
  court `tests/abi_manifest_court.rs` is landed as the standing
  limits-table court, and the cap itself is exercised end to end by the
  03:20 receipt (`wasm_frame_bound_court` 8/0, exit 0). The drift the row
  described (two independent constants in different files) no longer
  exists.
- **D1 → CLOSED.** `SECURITY.md:85-92` — "Imported WASI capability
  inventory" section landed with the live-section-named import list ("the
  names below match the live section, no non-WASI import modules").
- **D2 → CLOSED.** `docs/WASM.md:237-240` — the 256 MiB figure is gone;
  the doc states the 16 MiB request cap (`MAX_REQUEST_BYTES`) and 16 MiB
  outstanding cap (`MAX_OUTSTANDING_ALLOC_BYTES`, `src/abi.rs:73`), uniform
  across targets.
- **BB1 → CLOSED.** Receipt:
  `receipts/2026-10-24-wasmi-retirement-deletion.md` — `cargo tree`
  inversion probes witness wasmi/wasmi_wasi absent from the dependency
  graph after the runner-source deletion in `6da1d33`; the host flip
  (tests_py blackbox → graphlaw-run-eh) is witnessed by the step-2 receipt
  (19/0 ×2).
- **D4 — PARTIAL (status stays OPEN).** The dialect-census witness class
  is now DURABLE: six `receipts/2026-10-24-dialect-{sql-rdf,turtle,n3,
  shacl,shex,datalog}.md` with embedded output. The fuzz / memory-wall /
  exit-codes classes remain transcript-class pending the L5-class re-runs
  on the final artifact identity.
- **UU1 — PENDING L5/L6 (fresh witness).** The battery L5a/L5b legs ran
  13:25-13:33 and are compile-abort RED (`purrdf_stack` unresolved, 174
  errors, EXIT:101); L6 legs EXIT:1/3 on the pytest conftest build —
  consistent with the in-flight vendor surgery, unadjudicated as battery
  legs. UU1's full-battery re-run stays pending until the tree settles.
- **G7 — CARRY (config gap FIXED this window).** `a00770b` puts
  `abi_response_builders_court` in the kill suite (the committed killer of
  the 11 SQLRDF-28 survivors could not run under the prior config) and
  widens examine_globs to `src/registry/*.rs` — **inventory 88 → 204**
  (definitions 15, emit_json 50, emit_ttl 51), GUARDED/GATED restore sets
  widened to match. The row's remaining witness is the solo mutation-court
  verdict over the 204-mutant inventory (GO-NO-GO-v26.10.24 §A A-1).
- **G8 — CARRY (fresh post-fix witness).** The sniff fixes landed
  (`6da1d33`: literal-strip, IRI-aware `#`, escape-aware `line_terms`,
  SNIFF-01 BareSql refusal, HEAD_BUDGET 8192 at `src/dialect/sniff.rs:25`;
  court `tests/sniff_court.rs`; refusal registered `docs/refusals.md:220`)
  and the corpus leg went green — but the battery L4 differential still
  disagreed native-vs-wasm (`sniff_dialect_agrees_with_parse_dialect_
  for_every_serialization` failed 15/1 at `tests/op_differential.rs:181`,
  `/tmp/bat-L4.log` 11:35, POST-fix). Fresh divergence class: sniff
  classifies NTriples natively, Turtle in-wasm, on at least one
  serialization. Carry: rebuild on the settled tree → re-witness → register
  row (fix or typed-divergence append).
- **G2 — CARRY (adoption half).** `src/egress/collector.rs` landed on disk
  (12,197 B, 2026-10-24); **no `egress_arrow` op** (`grep -rn egress_arrow
  src/abi.rs src/abi/` = no match at write). The row's closing witness
  (Twin B; pinned-view; FIBO) remains unmet.

### 2026-10-26 — WP P03 hard-enclosure landing (lane P03-wasm, write lease = this file + runner docs)

The runner docs line "a fired epoch deadline surfaces as an interrupt trap (97)
until a host watchdog increments the epoch" is FLIPPED: the watchdog is real
(`src/bin/graphlaw_run_eh.rs` `Watchdog`, one engine-epoch increment per 50 ms
until its stop flag), the deadline is re-armed per request cycle, and a fired
deadline is its own typed exit — **99** (`RunError::EpochExpired`, classified
after fuel 98) with diagnostic `guest epoch deadline expired after <ticks>
ticks (wall-clock strangle)`; `--epoch-ticks N` arms a real deadline and the
default 1e9 ticks is the disarmed production posture. The corresponding
"cannot fire today" comment in the runner is flipped to match. The shared
hard-enclosure host landed as `tests/common/enclosure.rs`
(`EnclosureHost`: exceptions + `consume_fuel` + `epoch_interruption` +
`OptLevel::None`, 64 MiB `WallLimiter`, `FuelPolicy::Unbounded | PerOp`,
seed-BOTH-budgets-before-instantiate, per-cycle reset, OutOfFuel-then-Interrupt
classification, host-synthesized `ResourceLimit` envelopes matching the module
`limit()` byte-shape). Court: `tests/wasm_enclosure_court.rs` (calibration +
envelopes + epoch stall + 9000-frame leaking campaign + 63/64 and 31/33 stack
probes + the runner exit-99 leg); witnessed boundary: formula nesting 63 admits
/ 64 refuses typed `EngineRejected` ("terms nested more than 64 levels deep")
on the current artifact — witnessed through the real runner before the court
encoded it.

Reopen conditions: any cited court returning red on the same loci; a landed
battery receipt contradicting a battery-leg verbatim line cited above; a
mutation of the single-sourced cap or the sniff fixes that its court fails
to invert on. Table cells untouched — the table is the register of record
at register-authoring time; this log carries the flips.
