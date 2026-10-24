# Fuel calibration receipt — FUEL_PER_REQUEST 10M → 100B (2026-10-24)

Subject: `/Users/sac/graphlaw` @ `main`, calibration commit
`c3e05bd111b0ad5b7768755ed5ec830be1bd346a` (on `main`, ancestor of HEAD
`4df19b5`). Landed value verified in the working tree at HEAD:
`src/bin/graphlaw_run_eh.rs:45` = `const FUEL_PER_REQUEST: u64 =
100_000_000_000;` (blob `4d40a4b821e9e24f…52379017`; pre-calibration
blob at `c3e05bd~1` = `d0a5b2af06e9db2e…9739dd028`). This receipt is a
read-and-register lane: no cargo, no git, no src edits.

## Calibration method — empirical witness ladder

The budget was not picked from a model. It was raised through rungs,
each rung admitted only by a real run of the EH court against the
pinned artifact: a rung is kept only if the witness output (exit code,
test counts, stderr text) shows the pinned-artifact workload admitted,
and a rung is rejected only if the output shows the typed fuel
refusal. Sequence: 10M (red) → 1B (red) → 100B (green). Every embedded
output below is verbatim from logs on disk at filing time, digested
with `b3sum` so the ladder is replayable against exact bytes.

## Rung 1 — 10M: RED, typed exit 98 on the pinned-artifact leg

Command (per the gap-register flip-log addendum): `cargo test --test
graphlaw_run_eh_court -- --test-threads=1`, log `/tmp/coord_t3.log`
(b3sum `e76ff73bb80eb811…fb904e470`), finished 01:52 PDT. Verbatim
failure block:

```
running 7 tests
test exit0_clean_empty_stdin ... ok
test exit84_digest_refusal ... ok
test exit90_eof_mid_frame ... ok
test exit95_invalid_wasm_artifact ... ok
test exit96_missing_abi_exports_refusal ... ok
test exit96_non_wasi_import_refusal ... ok
test pending_artifact_pin_capabilities_frame_exit97_uninitialized_element ... FAILED
...
graphlaw-run-eh: guest exhausted fuel budget 10000000

  left: 98
 right: 97

test result: FAILED. 6 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 116.04s

error: test failed, to rerun pass `--test graphlaw_run_eh_court`
```

Cargo process exit: 101 (test failure); runner exit witnessed: 98 (the
typed fuel refusal), asserted against expected 97. The same red
witness is recorded in the gap-register flip-log addendum
(`docs/reference/wasm-gap-register.md:92-101`), verbatim:

```
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
```

Era marker: `/tmp/mc5_run.log` (b3sum
`c206d619258b1b71…daf35137d09ad891`), verbatim, characterizes the
mutation-court lane of the same window (typed exit 4; flip + byte-exact
restore legs green in the same log; post-restore sanity FAILED on the
then-committed mutant shim — receipt-gated to its own post-fix rerun,
not part of this ladder):

```
mutation_court: log file: /tmp/mc5_000154.log
mutation_court: callsite-revert leg: court FLIPPED under shim (exit 101, expected)
mutation_court: shim output tail:
failures:
    transition_with_unknown_capability_refuses_typed_end_to_end

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s

error: test failed, to rerun pass `--test law_state`
mutation_court: callsite-revert restore byte-exact (sha256 5952cd0422b95397a3800555eaa14e6ca765974d5dc5ae7342585c4eeeb07bbe verified)
mutation_court: post-restore sanity FAILED: court does not pass on restored tree
mutation_court: tree restored byte-exact (sha256 verified, git status unchanged)
```

### tc08 egress leg — what the durable output actually shows

The ladder shorthand "10M: exit 98 on pinned-artifact capabilities +
tc08 egress" does NOT match the durable tc08 witnesses. Every tc08
run log from the window shows the typed DuckDb EINVAL envelope
(SQLRDF-27, musl↔wasi-libc fcntl mismatch), independent of fuel:

- `/tmp/coord_t1.log` (10M era, 01:38, b3sum
  `41286d426c3cf774…b0ad31e602`): `test tc_fibo_08_copy_executes_with_pid_keyed_egress ... FAILED`,
  `test result: FAILED. 3 passed; 1 failed; … finished in 524.91s`,
  error body `"IO Error: Could not set lock on file
  \"/tmp/fibo_accounts_93045.csv\": Invalid argument"`.
- `/tmp/witness_egress.log` (still 10M in tree, 02:06, b3sum
  `903ed63fb9d3e2c5…5dbe475f766`): same EINVAL, `finished in 131.67s`.
- `/tmp/coord_egress.log` (post-calibration, 02:14, b3sum
  `7bef2478330f8bd7…84c46137d0878267`): same EINVAL after the raise,
  `finished in 89.25s` — the fuel raise did not and cannot move this
  leg; its refusal is an egress-defect refusal, not a fuel refusal.

No durable output anywhere on disk shows tc08 egress exiting 98. The
tc08 leg is registered here at its witnessed strength: FAILING on
SQLRDF-27 at every rung, fuel-independent.

## Rung 2 — 1B: RED, witness strength = commit message only

Sole durable witness is the calibration commit's own message
(verbatim, `git log -1 c3e05bd`):

```
c3e05bd 2026-10-24 02:12:58 -0700 fix(v26.10.22): fuel calibrated to 100B/request — 10M and 1B both exhaust on pinned-artifact init+capabilities (witnessed exit 98); exit-0 flip + 7/0 eh court witnessed at 100B
```

No 1B output log survives on disk (`/private/tmp` and repo searched
2026-10-24 for `exhausted fuel budget 1000000000`; zero hits). This
receipt therefore records the 1B rung as **commit-message-witnessed,
output-unembedded**: sufficient to order the ladder, insufficient as
an embedded-output receipt. The margin statement below carries the
weaker bracket that results.

## Rung 3 — 100B: GREEN, exit-0 flip accepted, eh court 7/0

The calibration diff (`git show c3e05bd`), verbatim:

```diff
diff --git a/src/bin/graphlaw_run_eh.rs b/src/bin/graphlaw_run_eh.rs
index 47da3ef..f2eed38 100644
--- a/src/bin/graphlaw_run_eh.rs
+++ b/src/bin/graphlaw_run_eh.rs
@@ -42,7 +42,7 @@ const _: () = assert!(graphlaw::abi::MAX_REQUEST_BYTES <= WALL_BYTES as usize);
 /// Guest fuel budget per request (wasmtime fuel units, ~1 per instruction).
 /// A guest call that spends it all is a typed refusal (exit 98), never a raw
 /// trap or an unbounded hang. Reset before every guest execution.
-const FUEL_PER_REQUEST: u64 = 10_000_000;
+const FUEL_PER_REQUEST: u64 = 100_000_000_000;
```

Green witness, same court as rung 1 (`/tmp/coord_t3b.log`, 02:14,
b3sum `cb3b410720840ee6…f7810759bd73ef378721da3e2a7fd`), verbatim:

```
running 7 tests
test exit0_clean_empty_stdin ... ok
test exit84_digest_refusal ... ok
test exit90_eof_mid_frame ... ok
test exit95_invalid_wasm_artifact ... ok
test exit96_missing_abi_exports_refusal ... ok
test exit96_non_wasi_import_refusal ... ok
test pending_artifact_pin_capabilities_frame_exit97_uninitialized_element ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 26.06s
```

The court flipped red→green on the only changed variable (fuel), with
the pinned-artifact capabilities leg admitted end to end, including
`exit0_clean_empty_stdin ok` — the exit-0 flip is accepted by the same
court that refused at 10M. Surrounding post-calibration courts in the
same window (all exit 0): `wasm_fibo_fixtures_eh` 3/0 in 216.99s
(`/tmp/coord_t1b.log`), `wasm_duckdb_lifecycle_eh` 3/0 in 207.14s
(`/tmp/coord_t2b.log`), `attach_bypass_court` 3/0 in 0.37s
(`/tmp/coord_t4b.log`), `abi_response_builders_court` 10/0 in 0.01s
(`/tmp/coord_t5b.log`).

## Margin statement

Witnessed on the pinned-artifact init+capabilities leg:

- 10M exhausts (exit 98, embedded output above) → requirement > 10M.
- 1B exhausts (commit-message witness only) → requirement > 1B, taken
  at the weaker evidence strength.
- 100B admits (7/0, embedded output above) → requirement ≤ 100B.

So the witnessed bracket is **(1B, 100B]** at mixed evidence strength,
and **(10M, 100B]** at embedded-output strength. Headroom at the
landed setting is at most ~100× the bracket's lower bound and is not
pinned tighter by surviving output: the budget may sit as little as
just-above-1B from the true requirement. Wall-clock shape: the
exhaustion run took 116.04s to refuse; the green court full 7-test run
takes 26.06s — the fuel ceiling is not the latency floor at this
workload. Epoch caveat carried by the code itself
(`graphlaw_run_eh.rs`, comment above the constant): "No watchdog
increments the engine epoch yet, so the deadline cannot fire" — the
epoch deadline is therefore NOT an active second bound at this
calibration.

## Falsifiers

1. A legitimate (law-admitted, non-attacking) request that exhausts
   100B — i.e. a green-workload run ending in
   `guest exhausted fuel budget 100000000000` with exit 98 — falsifies
   this calibration: recalibrate by the same witness ladder.
2. A hostile request that completes inside 100B (admitted exit 0 or
   typed-success on attacker-controlled input) — the budget is useless
   as a DoS bound at that point; the response is revisiting the epoch
   deadline (which currently cannot fire) before raising further.
3. Any budget change landing without a rung witness of the strength
   embedded above falsifies the ladder method itself.

## Receipt fields

- **Identity**: subject = calibration commit `c3e05bd` on `main`
  (single-file diff, blob `d0a5b2af…` → `4d40a4b8…`); witness digests:
  `coord_t3.log` `e76ff73bb80eb8117657ccba6fafb294fb0eb26ccb90f01308d2c28fb904e470`,
  `coord_t3b.log` `cb3b410720840ee67ad9e8a21b2b5b1f9b6f7810759bd73ef378721da3e2a7fd`,
  `coord_t1.log` `41286d426c3cf77473233db776ed0bd329fe6b92b2cc654d6674bcb0ad31e602`,
  `witness_egress.log` `903ed63fb9d3e2c53193ffa85720092c792ec9a485b5df0df5b0b5dbe475f766`,
  `coord_egress.log` `7bef2478330f8bd7b51682c83cce03c46137d0878267caae0adfde71cc984235`,
  `mc5_run.log` `c206d619258b1b71f2537a32ee76720c872ef371c6d0c1a3daf35137d09ad891`.
- **Authority**: register lane, lease = this receipt file + one
  RECEIPTS.md pointer row only; no cargo, no git transitions, no src
  edits; witness logs are coordinator-era artifacts read in place.
- **Consequence**: one new receipt file; one appended RECEIPTS.md
  pointer row; zero rows rewritten in any register; the ladder's red
  rung stands as recorded in the gap-register addendum (already
  landed), the green rung lands here.
- **Replay**: `git show c3e05bd` (expect the one-line diff above);
  `grep -n "FUEL_PER_REQUEST: u64" src/bin/graphlaw_run_eh.rs` (expect
  `100_000_000_000` at :45); `b3sum` each witness log (digests above;
  `/tmp` copies may age out — digests are the durable identity);
  `b3sum` this file for the pointer row.
- **Standing**: calibration **LANDED** and **ALIVE** at 100B on
  embedded-output evidence for the 10M→100B transition; the 1B rung is
  **PARTIAL** (commit-message witness, no embedded output); tc08
  egress remains **OPEN on SQLRDF-27** (fuel-independent, typed EINVAL
  envelope at every rung). Drift pointer (not this lane's lease):
  `docs/reference/runner-exit-codes.md:82,89` still documents
  `FUEL_PER_REQUEST` = 10_000_000 — stale post-calibration, owner
  should regenerate/patch via its own lane.
