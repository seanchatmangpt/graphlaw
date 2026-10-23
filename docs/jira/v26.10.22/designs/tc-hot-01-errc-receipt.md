# TC-HOT-01 one-update fold receipt (2026-10-22)

**Verdict: FAIL.** Clone-path p99 is inside 250 ns and the refusal leg passes 100 ns. The 10M combined gate exits 1 because batch-p50 variance is 26.91% against a 5% budget. The SPC smoke exits 1 because Cpk is 0.001 against 1.33. Neither miss is restated as a pass. TC-HOT-02 was not run: `target-lane92/tc-hot-25m` is absent. The 150 ns fold row has no gate.

## What changed before this run

`fold_params_anchored` copies `h_ast` and the inline param bytes into one `[u8; 96]` and calls `Hasher::update` once. The eval path passes the pinned `h_ast` through with no extra 32-byte copy. The timed refusal leg copies the static class label and `RefusalMessage::Static` with two `write_str` calls. `Display` stays outside that leg. Budgets were not changed.

## Bit-identity

`CARGO_TARGET_DIR=/tmp/gl-target cargo test --lib receipt_state_clone_path_bit_identical_to_reference --offline`

Exit 0. `receipt_state_clone_path_bit_identical_to_reference` ... ok. 1 passed.

## TC-HOT-01 full (10M) — exit 1

`CARGO_TARGET_DIR=/tmp/gl-target TC_HOT_FULL=1 cargo bench --bench tc_hot_01 --offline`

One run. Not retried.

```
iterations=10000000 canon_bytes=649 params=5 bit_identity=VERIFIED
VARIANCE_SELF_CHECK batches=3 batch_iters=3333333 batch_p50s=[125, 166, 166] spread_pct=26.91% budget=5% : VARIANCE-EXCEEDED
re_absorb_baseline: p50=1125ns p95=1625ns p99=1625ns p99.9=3667ns
clone_path: p50=125ns p95=208ns p99=209ns p99.9=250ns
refusal_invariant: p50=41ns p95=42ns p99=42ns p99.9=83ns
REFUSAL_INVARIANT display_render="EngineRejectedtc-hot-01 invariant probe" carries_class_and_message=true
GATE clone_path p99(209ns) <= 250ns AND refusal invariant AND variance<5% : FAIL
GATE refusal_string_path p50(41ns)/p99(42ns) <= 100ns (0.10us PRD) : PASS
GATE param_integration small net(-52.05ns) / large amortized net(-68.24ns) <= 40ns : PASS
GATE timestamp_source 2.7ns < 10ns AND monotonic : PASS
```

The param nets are negative because that run's scaffold batch (168.87 ns) was slower than the small and amortized batches (116.82 ns and 100.63 ns). The comparison is `<= 40 ns`, so the printed gate is PASS. It is not a claim that the fold takes negative time.

## SPC smoke — exit 1

`CARGO_TARGET_DIR=/tmp/gl-target bash scripts/spc_court.sh`

USL 250 ns, Cpk minimum 1.33, mode SMOKE, n = 1,000,000. New receipt: `receipts/spc-2026-10-22T191548.json`. Older `receipts/spc-2026-10-22.json` and `receipts/spc-2026-10-22-inject.json` were left in place.

```
VARIANCE_SELF_CHECK batches=3 batch_iters=333333 batch_p50s=[166, 166, 167] spread_pct=0.60% budget=5% : PASS
clone_path: p50=166ns p95=208ns p99=209ns p99.9=250ns
refusal_invariant: p50=41ns p95=42ns p99=42ns p99.9=125ns
GATE clone_path p99(209ns) <= 250ns AND refusal invariant AND variance<5% : PASS
GATE refusal_string_path p50(41ns)/p99(42ns) <= 100ns (0.10us PRD) : PASS
GATE param_integration small net(37.66ns) / large amortized net(13.11ns) <= 40ns : PASS
GATE timestamp_source 0.7ns < 10ns AND monotonic : PASS
```

SPC verdict from the receipt: mean 185.686 ns, stddev 21610.119 ns, p99 209 ns, max 21,413,887 ns, Cpk 0.001, reason `cpk 0.0010 < 1.33`. p99 meets the USL. Cpk does not. Shewhart rule 1 flagged 27 samples.

## TC-HOT-02

Removed. `benches/tc_hot_02.rs` timed a DuckDB parquet scan and a raw prepared statement. That is not a graphlaw benchmark.
