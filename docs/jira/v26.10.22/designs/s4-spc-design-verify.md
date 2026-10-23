# S4 Skeptic — adversarial verification of spc-red-remediation-design.md

Verifier: lane S4, read-only + this report. Repo /Users/sac/graphlaw.
Subject: docs/jira/v26.10.22/designs/spc-red-remediation-design.md (L169).
Method: receipt re-reads (all five on-disk receipts re-parsed), arithmetic
recomputed, every cited file:line opened, composite gate stress-tested.

## Claim 1 — two-population model, p99.9 shelf at 291-292 across three receipts

**CONFIRMED with two corrections (strengthens the claim).**

Receipt re-reads (exit 0, python json parses of receipts/spc-*.json):

| receipt | mean | stddev | cpk(recomputed==receipt) | p50 | p95 | p99 | p999 | max | rule1 n |
|---|---|---|---|---|---|---|---|---|---|
| spc-2026-10-22.json | 187.979 | 70.117 | 0.2948 | 167 | 209 | 250 | 292 | 29,759 | 70 |
| T173758 | 180.476 | 5152.504 | 0.0045 | 167 | 208 | 291 | 292 | 4,075,519 | 18 |
| T173809 | 182.680 | 7116.309 | 0.0032 | 167 | 209 | 292 | 292 | 6,639,615 | 16 |
| T184929 | 168.670 | 48.847 | 0.555 | 167 | 208 | 250 | 292 | 39,615 | 580 |
| T191548 | 185.686 | 21610.119 | 0.001 | 166 | 208 | 209 | 250 | 21,413,887 | 27 |

- p999 = 292 in FOUR receipts (design says three); T173809's p99 is 292,
  not the 291 printed in the design table (p99 291 is T173758 only).
  The shelf is real and stronger than claimed (4 independent runs).
- Design table "rule1 hits n/a / n/r" for T173758/T173809 is wrong: both
  receipts carry shewhart.rule1_beyond_3sigma.count = 18 and 16.
- §1's "p99=208-250 across all five receipts" is wrong: two receipts have
  p99 = 291/292 (the tail population already dominates p99 there). The
  correct intrinsic-body claim is p95=208-209 and p99 ≤ 250 in 3/5.
- "Tail fraction beyond ±3σ is 27-580 per million" → actual range is
  16-580 per million (16 and 18 exist).
- All five cpk fields recomputed as (usl−mean)/(3·σ) and match receipts
  exactly (0.2948, 0.0045, 0.0032, 0.555, 0.001).
- n = 999,999 samples per receipt confirmed (iterations=1,000,000).

## Claim 2 — intrinsic-Cpk arithmetic

**CONFIRMED.** Recomputed (python):
- Normal-fit sigma: (208−167)/1.645 = 24.924 ≈ 25 ns ✓
- Intrinsic Cpk: (250−170)/(3×25) = 80/75 = 1.0667 ≈ 1.07 ✓
- Tail-removal ceiling: sigma ≤ (250−170)/(3×1.33) = 20.05 ns, below the
  ~25 ns intrinsic spread → "raw per-sample Cpk ≥ 1.33 is not reachable by
  tail removal alone" is arithmetically sound (mean=170 is the stated
  working assumption; a lower mean would weaken it — at mean 166 the
  ceiling is 21.05 ns, still < 25, so the conclusion is robust to the
  166-170 mean range).
- sigma(batch p50) ≈ 25/√10000 = 0.25 ns ✓

Caveat (assumption, not error): the 25 ns comes from a two-quantile normal
fit; if the intrinsic body is not normal the 1.07 is an estimate. The design
states the fit explicitly, so this is disclosed, not hidden.

## Claim 3 — composite gate loophole stress

Gate: cpk_batch ≥ 1.33 AND tail_fraction ≤ 1e-4 AND p99 ≤ 250 (+ inject leg).

- **Redundancy, not loophole**: p99 ≤ 250 is strictly implied by
  tail_fraction ≤ 1e-4 (100/1M beyond USL ⇒ p99.99 ≤ 250 ⇒ p99 ≤ 250).
  Term 3 carries no independent information while term 2 exists. Harmless.
- **LOOPHOLE (real): magnitude-blind tail gate.** tail_fraction ≤ 1e-4
  bounds *how many*, never *how bad*. A run with 60 stalls of 21 ms each
  per 1M (tail_fraction 6e-5, well under 1e-4) plus a clean body passes ALL
  THREE terms while a real caller eats 21 ms stalls — T191548's max is
  21,413,887 ns, so this shape exists on disk today. p99 ≤ 250 does not
  catch it (p99 = body). Missing term: a max/far-tail fence (e.g.
  p99.99 ≤ F ns or max ≤ some multiple of USL), or a per-tail-sample
  magnitude budget.
- **Laundering channel the design cannot close: "quiet" is self-reported.**
  The load-context field (`load: "quiet" | "loaded"`) is written by the same
  harness that benefits from the label; there is no concurrent-lane
  detection, so a loaded run can self-label quiet and the body gate passes
  on confounded data. The tail gate still bites (tails grow under load), but
  the quiet/loaded distinction in the receipt is assertion, not evidence.
  The design's own honesty condition needs an enforcement story (e.g.
  load-probe before/after the run) or must be marked unenforced.
- Bimodal body (60% @100 ns / 40% @249 ns): batch p50s tight, cpk_batch
  high, tail_fraction 0, p99 249 ≤ 250 → PASS, but every statistic is
  within spec by definition; not a gate defect, noted for completeness.
- Anti-vacuity leg is sound: a 2 µs injected outlier trips rule1 AND
  tail_fraction (2 µs > 250 ns), so SPC_INJECT_OUTLIER=1 still flips the
  composite verdict. A gate that can exit 1 is preserved.

## Claim 4 — charter citations

- §5.2 quiet-machine close condition: **CONFIRMED verbatim** —
  CHARTER-DFLSS.md:97-99: "The 184–216 ns isolated baseline was
  contention-sensitive; intrinsic fold cost is 250–300 ns ... Close
  condition: quiet-machine re-measurement, no concurrent lanes."
  §1.2 CTQ "Ledger write latency — dispatch-carried 0.25 µs" also confirmed
  (CHARTER-DFLSS.md:28). hot-path-baseline.md CORRECTION quote confirmed
  (lines 33/39: "the earlier '184-216ns met' reading was MIN-of-batches").
  DFLSS-DOSSIER §7 close condition: not independently re-opened (time-box);
  the charter §5.2 primary citation carries the claim on its own.
- **Ratio-fence citation: REFUTED as cited.** The design attributes the
  ratio-fence law to "the charter" ("The charter's ratio-fence law ... per
  the no-sleep doctrine the charter inherits"). CHARTER-DFLSS.md contains
  NO ratio-fence or no-sleep text (grep over the file: zero hits for
  ratio/ratio-fence/sleep). The law lives in
  ~/.claude/rules/no-sleep-testing.md ("relative fence against an
  independent, sleep-free-by-construction baseline — paired rounds,
  min-of-N"), and the design reproduces its form faithfully — but the
  citation is to the wrong document. Fix: cite the no-sleep rule, not the
  charter.

## Line-number citations in the design (checked)

- scripts/spc_court.sh — USL/CPK_MIN defaults are at :31-32, NOT :18-24/:37
  as cited; :18-24 is the comment header. The `passed` python line is :107
  (design's "python passed line" is right in substance).
- benches/tc_hot_01.rs — BUDGET_NS:47 ✓, VARIANCE_BUDGET_PCT:298 ✓,
  batch p50 push :342 ✓ (all three confirmed).

## Verdict summary

| # | Claim | Verdict |
|---|---|---|
| 1 | Two populations; 292 ns p99.9 shelf | CONFIRMED (4 receipts, not 3; 3 table/label errors noted) |
| 2 | Intrinsic Cpk ≈ 1.07; tail removal insufficient | CONFIRMED (recomputed; robust over mean 166-170) |
| 3 | Composite gate sound | PARTIAL — magnitude-blind tail gate is a real loophole; "quiet" self-reported, unenforced; term 3 redundant |
| 4 | §5.2 quiet-machine close condition | CONFIRMED verbatim |
| 4b | Ratio-fence as "the charter's" law | REFUTED (it is the global no-sleep rule's law; charter has no such text) |

Bottom line: the measurement model and arithmetic stand. The composite gate
needs one more term (tail-magnitude fence) and either an enforcement story
or an honest "unenforced" marking on the quiet-host label before it is
implemented as designed.
