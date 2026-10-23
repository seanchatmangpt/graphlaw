# Lane M08 — receipt replay court (K7 guard builder)

- Date: 2026-10-23 (rotation seed `2026-10-23`)
- Subject: `/Users/sac/graphlaw` @ main `dab6a73`, working tree dirty (multi-lane wave; lane build `CARGO_TARGET_DIR=target-laneM08` and court-owned `/tmp/receipt-replay-court.*`).
- Lease: `scripts/receipt_replay_court.sh` (new), this file. One disclosed out-of-lease edit, below.
- Defect class: K7 — the blake3 chain proves byte-immutability, not truth; a
  fabricated receipt with never-executed output verifies clean against the chain.

## Court design (`scripts/receipt_replay_court.sh`)

- Pool: `receipts/*.md` carrying at least one cleanly-PAIRED (embedded
  `cargo test --test X` command, subsequent embedded `test result: ok. N passed; M failed` line).
  Pairing is load-bearing: `receipts/registry-courts-2026-10-22.md` carries two
  commands (registry_artifacts FAILED 17/5, registry_dispatch ok 14/0) — naive
  first-match extraction pairs command 1 with command 2's counts, which would
  itself fabricate. The first court prototype did exactly this; fixed.
- Sampling: N=5, deterministic rotation by `sha256(date)` over the pool;
  candidates whose `tests/<target>.rs` is mid-edit in the working tree loud-SKIP
  (`[SKIP] ... re-sampling`) and the next rotation slot replays; <5 clean replays
  = FAIL (never a silent pass).
- Replay: each embedded command re-executed with a court-owned
  `CARGO_TARGET_DIR`; embedded passed/failed counts compared to observed.
  Divergence → exit 1 naming the receipt.
- Selfcheck (anti-vacuity): a fabricated receipt (42 passed planted via sed into
  a copy of a sampled receipt) must be flagged; an unflagged fabrication exits
  `VACUOUS GATE` 1.

## Receipt (real run, 2026-10-23)

```
$ bash scripts/receipt_replay_court.sh; echo "COURT_EXIT=$?"
== sampled (rotation seed 2026-10-23), replaying until N=5 match:
[SKIP] /Users/sac/graphlaw/receipts/chicago-validation-2026-10-22.md: tests/chicago_validation.rs is mid-edit in the working tree — replay would conflate staleness with fabrication; re-sampling
[SKIP] /Users/sac/graphlaw/receipts/duckdb-statement-kinds-2026-10-22.md: tests/duckdb_statement_kinds.rs is mid-edit in the working tree — replay would conflate staleness with fabrication; re-sampling
-- replay /Users/sac/graphlaw/receipts/gws-10-e2e-coldchain-wasm.md : cargo test --test wasm_coldchain_e2e (embedded 3p/0f)
   match: 3p/0f
-- replay /Users/sac/graphlaw/receipts/gws-9-wasm-boundary-fuzz.md : cargo test --test wasm_boundary_fuzz (embedded 2p/0f)
   match: 2p/0f
-- replay /Users/sac/graphlaw/receipts/query-cache-court-2026-10-22.md : cargo test --test query_cache_court (embedded 8p/0f)
   match: 8p/0f
-- replay /Users/sac/graphlaw/receipts/query-ledger-court-2026-10-22.md : cargo test --test query_ledger_court (embedded 9p/0f)
   match: 9p/0f
-- replay /Users/sac/graphlaw/receipts/registry-courts-2026-10-22.md : cargo test --test registry_dispatch (embedded 14p/0f)
   match: 14p/0f
== PASS: 5 sampled receipts reproduce their embedded output byte-for-count
== selfcheck: planted fabricated receipt (42 passed) for /Users/sac/graphlaw/receipts/gws-10-e2e-coldchain-wasm.md
   flagged: fabricated 42p/0f vs observed 3p/0f
== selfcheck PASS: court flags fabricated counts (exit 1 class exercised)
COURT_EXIT=0
```

Also witnessed on this tree (same session, pre-fix prototypes):

```
RECEIPT-REPLAY-COURT FAIL: /Users/sac/graphlaw/receipts/chicago-validation-2026-10-22.md: COUNT DIVERGENCE — embedded 9p/0f vs observed 11p/0f (fabricated or stale receipt)
```

- The divergence is STALE, not fabricated: the K-lane's in-flight diff added 2
  tests to `tests/chicago_validation.rs` after the receipt was minted. This
  witness motivated the loud-SKIP-on-mid-edit rule.

## Disclosed out-of-lease edit (compile-freeze SLA, minimal unblock)

`tests/chicago_validation.rs` (K-lane in-flight file): a new helper
`column_values` matched `purrdf::TermValue` with only the `Literal` arm —
E0004 non-exhaustive, broke the shared compile (and therefore every replay).
Minimal fix applied:

```rust
.filter_map(|t| match &t.object {
    purrdf::TermValue::Literal { lexical_form, .. } => Some(lexical_form.clone()),
    // Non-literal objects carry no column value; skip (compile-freeze
    // unblock, lane M08 disclosure: K-lane in-flight file, minimal fix).
    _ => None,
})
```

Verified compile-only (target is K-lane-owned, not run by this lane):

```
$ CARGO_TARGET_DIR=target-laneM08 cargo check --test chicago_validation
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 33.66s
```

A second shared-compile break (`src/bin/graphlaw_ocel_export.rs`, 13 E0277/E0308
errors from another lane's `id()` type change) healed by its owning lane while
this lane polled `cargo check --bin graphlaw_ocel_export` — no edit by this lane.

## Falsifier

`bash scripts/receipt_replay_court.sh selfcheck` on a tree where the comparator
is broken must exit `VACUOUS GATE` 1; the check mode must exit 1 naming any
receipt whose replayed counts diverge.
