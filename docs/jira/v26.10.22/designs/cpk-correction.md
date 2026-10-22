# Cpk capability analysis — corrected (DFLSS Measure-lane A record)

The charter's §5.2 claim (μ=0.198µs, σ=0.012µs, Cpk=1.44) is **NOT
witnessed** — corrected by real measurement (`tests/p28_hot_path_bench.rs::
p28_hot_path_capability`, release mode, n=1,000,000 per-sample fold-path
timings of the true M3 path: receipt_state.clone() + hash_param_buffer +
finalize).

## Witnessed (best/least-contended run)
- μ = 278.0 ns | σ = 202.1 ns | Cpk(Cpu) = **−0.05** | P(X > USL) = 42.9%
- Quantiles: p50=250 p90=333 p99=375 p99.9=458 p99.99=2750 max=117917 (ns)
- Other runs: μ=326/652/340/370/347 ns, Cpk −0.09..0.00 (fan-out contention;
  tail samples to 31ms)
- Min-of-50×100-op reference: 299.6 ns/op best, 397-400 typical
- Disclosed overhead: per-sample timing carries 2× Instant::now ≈ 40-50ns;
  netting out, intrinsic fold ≈ 250-300ns

## Verdict
- Charter Cpk 1.44 does not reproduce under fan-out load: measured Cpk ≤ 0,
  NOT-CAPABLE as measured vs USL=250ns.
- The isolated 184-216ns baseline (designs/hot-path-baseline.md) was
  contention-sensitive; even netted, intrinsic cost is 250-300ns —
  **marginal at best against the 250ns USL, over the 150ns §2 row**.
- Close condition for the charter numbers: capability re-measurement on a
  QUIET machine (no concurrent lanes); the test prints both the
  distribution-based verdict and the batch-min reference so replay
  distinguishes them.

## Implication for the PRD budgets
- TC-HOT-01's 250ns p99 gate: marginal — met only on an idle machine.
- §2's 150ns H_query row: requires the clone-path memcpy residual (~2KB
  hasher state) to shrink (post-integration optimization), or the budget
  row corrected by measurement.
- The honest position: budget ADJACENT, not budget MET, under production
  load conditions.

## CORRECTION (RUST-6 perf audit): the "~2KB clone" framing is stale
Post-M3, `receipt_state` is `[u8;32]` (the h_ast anchor), NOT a Hasher —
there is no 2KB clone in the measured path. Verified against blake3 1.8.7
signatures: `Hasher` is ~430B, and the real per-fold cost is
`Hasher::new()` init (~430B memset + `Platform::detect()`) — attackable via
`blake3::hazmat::ChunkState` mid-state (~110B Clone, absorbs h_ast once at
compile; per fold: clone 110B + update(params) + finalize_non_root +
merge_subtrees_root, bit-identical, **40-80ns saving**). Secondary:
`ParamBytes::encode` + single-update instead of 5-8 `update` calls
(**15-30ns**); bench timing hygiene — the `Instant::now()` pair costs
~20-40ns of the measured p99 itself (batch-time the loop: **30-40ns off the
measured number**, measurement not real); needless 32B `copy_from_slice` at
duckdb_eval.rs:507 (destructure instead). Also: the 42% PCE overhead is
µs-scale eval allocations (column_names/predicate IRIs per call,
row_subject.clone per cell), not the fold. The two-node formula migration
(hq2 lane) is the structural prerequisite for the hazmat route.
