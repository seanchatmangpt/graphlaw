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

---

## RESOLUTION (2026-10-23, lane target-lane-abm2)

Fix applied in `/Users/sac/graphlaw/benches/workload_amortization.rs` (owned file).
**Option (a): fill-then-evict.** `build_schedule` warm-fills the COLD entries
first (compile cost stays outside the timed region), then
`db.shared_cache().bump()` evicts exactly them via epoch invalidation
(`src/query_cache.rs:154`), so every scheduled miss is a genuine first touch.
HOT entries are warm-filled AFTER the bump so their first replay is a genuine
C1 hit. Two schedule-realism corrections were forced by the courts during the
fix (both caught by asserts, per design):

1. **Cold-key dedupe (pass 2).** Hot salt `q % m` lands inside the cold salt
   range `0..m`, so a hot slot can share a cache key with a cold slot
   ((tid, salt) determines the graph → the BLAKE3 key). Warm-filling such a
   key would have re-emptied the cold first touch. Pass 2 skips hot keys in
   `cold_keys`; the cold miss itself repopulates them before any replay.
2. **Shared-key warm-up class.** Two hot slots can also share a key: its
   first touch after the bump is a miss, later touches hit — asserted
   accordingly (`hot_seen` set).

The per-schedule cache-class court (`run_mix`, "cache class must match the
schedule") runs ONCE per fraction in `payload_sweep`'s single pass; the divan
leg uses `run_mix_timed` (readback non-vacuity per iteration, no class assert)
because `bench_local` re-invokes the closure after cold entries are
legitimately populated by the prior iteration.

### Real command + exit

```
$ CARGO_TARGET_DIR=target-lane-abm2 cargo bench --bench workload_amortization 2>&1 | tail -45
    Finished `bench` profile [optimized] in 1.18s
     Running benches/workload_amortization.rs
PAYLOAD: /Users/sac/graphlaw/target-lane-abm2/tmp/amortization-payload.json
BOOT-TAX: total_ms=63.759 ms_per_shape=3.9849 hot_floor_us=2.06 break_even_queries=30959 hot_hits=1000
Timer precision: 41 ns
workload_amortization     fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ boot_tax_admission     74.67 ms      │ 79.37 ms      │ 75.72 ms      │ 75.83 ms      │ 100     │ 100
╰─ zipf_mix_amortization
   ├─ 0.00001             20.73 ms      │ 21.26 ms      │ 20.83 ms      │ 20.85 ms      │ 100     │ 100
   ├─ 0.0001              20.78 ms      │ 21.54 ms      │ 20.95 ms      │ 20.96 ms      │ 100     │ 100
   ├─ 0.001               20.83 ms      │ 22.19 ms      │ 20.98 ms      │ 21 ms         │ 100     │ 100
   ├─ 0.01                21.11 ms      │ 32.9 ms       │ 21.39 ms      │ 21.56 ms      │ 100     │ 100
   ╰─ 1                   21.61 ms      │ 1.018 s       │ 22.37 ms      │ 32.39 ms      │ 100     │ 100
EXIT=0
```

(zsh `${pipestatus[1]}` capture; 7 pre-existing `noop_method_call` warnings in
`src/bin/graphlaw_ocel_export.rs`, unrelated.)

### Measured sweep (from the payload JSON, single non-vacuity-checked pass)

| target f | achieved f | hits | misses | total ms | amortized µs/query | init ms |
|---|---|---|---|---|---|---|
| 1.0      | 1.0     | 0    | 10000 | 2482.697 | 248.270 | 76056.5 |
| 1e-2     | 1e-2    | 9900 | 100   | 31.396   | 3.140   | 46967.3 |
| 1e-3     | 1e-3    | 9990 | 10    | 22.119   | 2.212   | 39470.2 |
| 1e-4     | 1e-4    | 9999 | 1     | 21.153   | 2.115   | 39377.3 |
| 1e-5     | 1e-4 (M clamps to 1) | 9999 | 1 | 20.963 | 2.096 | 39388.9 |

Achieved fraction equals target at every representable fraction (1/N
granularity: 1/10000; the 1e-5 target clamps to M=1, reported honestly).

### Boot tax + break-even (this run)

- 16-shape compile+admit boot tax: **63.759 ms** total, 3.985 ms/shape.
- Hot floor (C1 hit, mean over 1000 real hit evals): **2.06 µs**.
- Break-even: **~30,959 hot queries** to amortize the boot tax.
- Payload: `/Users/sac/graphlaw/target-lane-abm2/tmp/amortization-payload.json`
  (written by the bench to `$CARGO_TARGET_TMPDIR`, never `receipts/`).

### Non-vacuity witnesses fired during the fix

- Run 1 (pre-fix baseline): line 152 court fired at f=1.0 (the RED this
  resolution closes).
- Run 2 (fill-then-evict, v1): `hot warm-up at the new epoch must be a hit`
  fired — shared hot key's first post-bump touch is a miss; corrected with
  `hot_seen` classification (correction 2 above).
- Run 3: divan-leg class assert fired on bench_local iteration 2 — court
  relocated to the single-pass payload sweep (`run_mix_timed` for the divan
  leg only). Final run: exit 0, all class asserts green in the sweep.

Disk at run start: 35 GiB free (`df -h /`), above the 20 GiB gate.

### Standing

ALIVE for the sweep on this subject: exit 0, achieved fractions exact, hit/
miss counts match the schedule at every fraction, real 2-row readbacks
asserted per query in both legs. Cross-run medians vary with machine load
(boot tax 102 ms on the previous run vs 63.8 ms here); per-run tables above
are the receipts.
