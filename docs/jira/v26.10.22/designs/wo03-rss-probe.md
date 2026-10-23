# WO-03 RSS Expansion Probe

## What this is — and is not

Measurement of resident-set size during graphlaw-eyeron's forward-chained
N3/Datalog deductive closure. **Honesty scope: this probe does NOT witness
the "4.5B virtual triples" claim.** It witnesses the per-rule / per-fact
marginal RSS that any extrapolation rests on. The 4.5B projection below is
LINEAR EXTRAPOLATION ONLY, spanning ~5.6 orders of magnitude beyond the
largest measured rung.

- Probe: `/Users/sac/graphlaw/examples/rss_expansion_probe.rs`
- Subject: `graphlaw-eyeron` `PreparedReasoner::reason` (real forward
  reasoner, real parsed N3, no mocks).
- Host: macOS (Darwin 25.2.0). RSS via std shell-out `ps -o rss= -p <pid>`;
  page granularity 16 KB.

## Method

Synthetic rules `{ ?x :p<i> ?y } => { ?x :q<i> ?y } .` — rule i activates
only on predicate `:p<i>`, so the ladder design isolates each axis:

- Ladder A (per-rule cost): rules 100 -> 1,000 -> 10,000; facts fixed at
  1,000 with only 100 active premises => closure fixed at 2,000 triples.
- Ladder B (per-fact cost): facts 100 -> 1,000 -> 10,000 over a fixed
  100-rule program => closure tracks the fact count.

Run as ONE process (cumulative RSS; per-rung deltas are within-rung). All
rungs must report `CompletionStatus::Complete` or the probe exits 1.

## Commands and exits

```
CARGO_TARGET_DIR=target-lane-wo3 cargo run --example rss_expansion_probe
# exit 0 (run twice; final run captured below)
CARGO_TARGET_DIR=target-lane-wo3 cargo check --example rss_expansion_probe
# exit 0
```

```
label | rules | facts | rss_before_kb | rss_during_kb | rss_after_kb | delta_kb | closure | status
A1 rules=100    facts=1000 active=100 | 100 | 1000 | 4880 | 6928 | 6928 | 2048 | 2000 | Complete
A2 rules=1000   facts=1000 active=100 | 1000 | 1000 | 9488 | 10416 | 10416 | 928 | 2000 | Complete
A3 rules=10000  facts=1000 active=100 | 10000 | 1000 | 39360 | 44000 | 44016 | 4640 | 2000 | Complete
B1 rules=100    facts=100  active=100 | 100 | 100 | 44032 | 44048 | 44048 | 16 | 200 | Complete
B2 rules=100    facts=1000 active=100 | 100 | 1000 | 44112 | 44112 | 44112 | 0 | 2000 | Complete
B3 rules=100    facts=10000 active=100 | 100 | 10000 | 48096 | 62240 | 62256 | 14144 | 20000 | Complete
    derived=1000 limits=[] errors=[]   (per rung; all limits empty)
EXIT=0
```

## Marginal cost

Per rule (Ladder A, rss_before deltas across rungs, ~fixed closure):

- 100 -> 1,000 rules: (9488 - 4880) KB / 900 = 5.12 KB/rule
- 1,000 -> 10,000 rules: (39360 - 9488) KB / 9,000 = 3.32 KB/rule
- **~3.3-5.1 KB per rule** (parsed program + agenda index), stable across
  one decade of scaling.

Per fact (Ladder B, rss_before deltas, fixed program):

- 100 -> 1,000 facts: below 16 KB page granularity (0 delta measured).
- 1,000 -> 10,000 facts: (48096 - 44112) KB / 9,000 = 0.44 KB/fact
  (explicit + 1 derived per explicit, plus Document/index overhead).
- **~0.4 KB per fact** (lower rungs noisy at 16 KB page granularity;
  dev-profile overhead inflates absolute values, marginal is the finding).

## Typed-limit witness (cartesian rung, discarded from the ladder)

An earlier shape of this probe ran rules x facts directly (1,000 x 1,000,
closure 1,000,000): the reasoner returned INCOMPLETE with
`limits_reached=[Iterations]` (`max_iterations` default 1,000,000),
`errors=[]`, peak RSS 597,008 KB, probe exit 1. The engine stops at a
typed limit, not an OOM — the 4.5B-scale safety property is
`CompletionStatus != Complete` + a named limit, not memory exhaustion.

## Projection to 4.5B triples — EXTRAPOLATION ONLY

- Linear, from the largest measured rung (10^4) to 4.5 x 10^9: a span of
  ~5.6 decades with NO measured point above 10^4.
- Materialized closure facts at ~0.4 KB each: 4.5B facts ~ 1.8 TB RSS;
  upper band at the worst per-fact marginal observed in dev (~1.5 KB)
  => ~6.8 TB.
- Rule-program storage at ~3.5 KB/rule does not scale the triple count:
  4.5B virtual triples would need materialization at ~TB scale (ruled out
  on any host) or a lazy/virtual representation the current engine does
  not have (the forward chainer materializes `derived` facts).
- Error bars, disclosed: (1) linearity assumed, not measured beyond 10^4;
  (2) index/matcher state may scale superlinearly and is not separated
  from the fact marginal; (3) dev-profile build inflates absolute RSS (a
  `--release` confirmation was DISK-BLOCKED, below); (4) 16 KB page
  granularity makes small-rung deltas read as zero.

## BLOCKED sub-run

`cargo run --release --example rss_expansion_probe` was refused by the
disk gate: `DISK-REFUSED(6.4 GiB free < 20 GiB)` (later 2.4 GiB free).
Release-profile confirmation of the marginals remains OPEN work once
stale lane build roots are purged. `df -h /`: 926Gi total, 11Gi used,
6.1Gi free at first refusal.

## Standing

- ALIVE (local): probe exits 0, all rungs Complete, real command output
  above.
- EXTRAPOLATION-ONLY: every statement about 4.5B triples in this receipt.
