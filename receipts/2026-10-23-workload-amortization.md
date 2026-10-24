# Receipt: workload_amortization bench run — 2026-10-23

Subject: `/Users/sac/graphlaw` @ main (0d11bc3 tree, lane `target-lane-abm`)
Scope: measure `cargo bench --bench workload_amortization` (cold-fraction sweep, boot tax, JSON payload).

## Verdict

**BLOCKED (deterministic bench RED)** — the bench panics before emitting any
measurement. This is not a transient shared-compile RED; no retry was run per
task law (retry reserved for transient compile failures only).

## Real command + output

```
$ CARGO_TARGET_DIR=target-lane-abm cargo bench --bench workload_amortization 2>&1 | tail -40
    Finished `bench` profile [optimized] target(s) in 8m 13s
     Running benches/workload_amortization.rs (target-lane-abm/release/deps/workload_amortization-2751869f7c0c1c89)

thread 'main' (58226726) panicked at benches/workload_amortization.rs:152:9:
assertion `left == right` failed: cache class must match the schedule (salt 0 tid 0)
  left: true
 right: false
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: bench failed, to rerun pass `--bench workload_amortization`
```

(Compile phase showed 7 pre-existing `noop_method_call` warnings in
`src/bin/graphlaw_ocel_export.rs`; benign, unrelated.)

Exit: non-zero (panic in bench harness).

## Root cause (from source read, /Users/sac/graphlaw/benches/workload_amortization.rs)

Design contradiction between warm-fill and the miss schedule:

- `build_schedule` (lines 113-134) warm-fills every scheduled MISS via
  `db.eval_ast_query("urn:bench:warm", ...)` into the SAME engine/connection
  (line 125), outside the timed region. This is intended to strip compile
  overhead, but the C1 cache is keyed on BLAKE3(Canon(G_AST) || P_params)
  (doc comment, line 11-13) and lives on the connection — so the warm-fill
  POPULATES the cache entry for the miss instance.
- `run_mix` (lines 138-164) then replays those identical queries on the same
  db. Every "guaranteed miss" first touch arrives as a C1 HIT.
- The in-loop non-vacuity assert (lines 152-156) catches it on the very first
  query of the first fraction (f=1.0, salt 0, tid 0): `receipt.cached == true`
  vs scheduled `expect_hit == false`. The court's non-vacuity fence fired
  exactly as designed — the failure is the schedule's, not the gate's.

Consequence: as written, the bench cannot produce a cold-fraction sweep at
any f < 1.0; the measured "amortized µs/query" would be a hot-only floor for
every fraction.

## Not produced (therefore UNVERIFIED, not measured)

- Cold-fraction sweep table (fraction → amortized µs/query, hit/miss counts): NOT MEASURED.
- Boot tax + break-even query count: NOT MEASURED (boot_tax() never reached).
- JSON payload: NOT WRITTEN. Payload is written only after the full sweep
  (line 216-217, `$CARGO_TARGET_TMPDIR/amortization-payload.json`); verified
  absent: `ls target-lane-abm*/amortization-payload.json` → no matches (exit 1).
- Whitepaper cross-check: not possible — no measured medians exist.
- Court flip-fence note (3x fence, twin ~5.5x bump-eviction): not exercised
  this run; the bench died before the divan sweep.

## Minimal fix direction (for the owning lane; this lane edits nothing)

Warm-fill must not populate the timed region's cache class, e.g. fill then
evict/clear the entry, use a separate connection for warm compile-check, or
schedule first-touch misses WITHOUT pre-warm (accepting compile in the timed
region) — any of which restores `receipt.cached == expect_hit`.

Disk at run start: 59 GiB free (`df -h /`), above the 20 GiB gate.
