# S11 skeptic verification — N22 restore claim, benches/tc_hot_02.rs

Subject: /Users/sac/graphlaw @ main (read-only lane). Date 2026-10-22.

## (1) /tmp/tc_hot_02_persist.rs vs benches/tc_hot_02.rs — PASS
`/tmp/tc_hot_02_persist.rs` exists (13960 bytes, Oct 22 19:29).
`diff` exit 0: byte-identical. N22's restore claim verified byte-level.

## (2) git diff HEAD delta — PASS
Exactly the knob, no drift. Hunk 1 (line ~98): TC_HOT_CORPUS_PATH env parse
(+4 lines incl. comment). Hunks 2-3: the two interpolation sites switch from
`dir/telemetry` to `corpus_dir` (net zero after the +4; ~5 changed lines as
claimed). No other hunks.

## (3) File status — PASS
`git status --porcelain` → ` M benches/tc_hot_02.rs` (modified, NOT deleted).

## (4) Documented line refs — PASS (roughly)
Actual: env parse at line 99 (claimed ~131), branch/use at 116 (claimed ~152),
view-register gate at 125-126 (claimed ~190). All refs point at the right
constructs; absolute offsets are ~30-65 lines lower than documented — off-by-drift
in the writeup, not in the code. Semantics as described: when TC_HOT_CORPUS_PATH
is set and non-empty, the corpus persists there; otherwise the per-pid temp dir.
Note: this is persist-or-temp only; there is no explicit load-vs-persist branch —
reuse happens implicitly because the glob points at the knob path.

Extra: 0 occurrences of Process.sleep/std::thread::sleep in the file (no-sleep law holds).

## Verdicts
1. byte-identical restore: PASS (real diff, exit 0)
2. delta == knob only: PASS (git diff HEAD receipt above)
3. ` M` not ` D`: PASS
4. line refs ~match: PASS with noted offset (constructs correct, numbers stale)

All four checks PASS. No drift, no deletion, knob semantics match the claim.
