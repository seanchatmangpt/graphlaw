# SPC Red Remediation Design (Lane L169)

Status: DESIGN — gate change owned by the hot-path lane
(`scripts/spc_court.sh`, `benches/tc_hot_01.rs`). This file is the design
record only; no gate code was touched by this lane.

## Problem statement

The production bar is Cpk >= 1.33 over the receipt fold path against
USL=250 ns (`scripts/spc_court.sh:18-24`, `cpk_min` default at
`scripts/spc_court.sh:37`). The gate is red. The red is a variance failure,
not a location failure: p50 sits at 166-167 ns (receipts
`receipts/spc-2026-10-22T184929.json`, `...T191548.json`), far under the USL,
while sigma is 48.8-21610 ns across the five on-disk receipts
(`stddev_ns` fields). The sigma is dominated by a scheduler-tail population:

| receipt | mean | stddev | cpk | p50 | p99 | p99.9 | max | rule1 hits |
|---|---|---|---|---|---|---|---|---|
| `spc-2026-10-22.json` | 187.98 | 70.12 | 0.295 | 167 | 250 | 292 | 29,759 | 70 |
| `...T173758.json` | 180.48 | 5152.50 | 0.0045 | 167 | 291 | 292 | 4,075,519 | n/a |
| `...T173809.json` | 182.68 | 7116.31 | 0.0032 | 167 | 291 | 292 | 6,639,615 | n/r |
| `...T184929.json` | 168.67 | 48.85 | 0.555 | 167 | 250 | 292 | 39,615 | 580 |
| `...T191548.json` | 185.69 | 21610.12 | 0.001 | 166 | 209 | 250 | 21,413,887 | 27 |

(n=999,999 samples per receipt; rule1 = Shewhart beyond mean±3σ,
`scripts/spc_court.sh` python block. Operator-cited 254/1M beyond ±3σ and max
326 µs is as-of-date 2026-10-22; the on-disk maxima above are the standing
receipts and span 39 µs to 21 ms — same phenomenon, run-dependent magnitude.)

## 1. Variance decomposition

The sample is a two-population mixture, and the receipts let us split it.

- **Intrinsic population** (the fold itself): p50=166-167, p95=208-209,
  p99=208-250 across all five receipts. Taking p50=167 and p95=208 as the
  intrinsic body: a normal fit gives intrinsic sigma ≈ (208-167)/1.645 ≈
  25 ns. Intrinsic-only Cpk ≈ (250-170)/(3×25) ≈ **1.07** — better, but
  still short of 1.33 on the raw per-sample statistic. Note this honestly:
  even with the tail removed entirely, per-sample Cpk 1.33 requires
  sigma <= (250-mean)/(3×1.33); with mean 170 ns that is sigma <= 20 ns,
  below the observed intrinsic body spread of ~25 ns. **Raw per-sample Cpk
  >= 1.33 is not reachable by tail removal alone.**
- **OS scheduler tail**: samples from ~292 ns (the p99.9 shelf, identical
  291/292 across three receipts — a real process boundary, not noise) up to
  21.4 ms max. Tail fraction beyond ±3σ is 27-580 per million in the receipts
  (operator figure 254/1M, as-of-date). p99.9 stuck at exactly 292 ns across
  independent runs is the boundary between the two populations: everything
  above ~292 ns is scheduler, not fold.
- Charter adjudication agrees:
  [CHARTER-DFLSS.md §5.2](../CHARTER-DFLSS.md) — "intrinsic fold cost is
  250-300 ns" under load, "the 184-216 ns isolated baseline was
  contention-sensitive", close condition "quiet-machine re-measurement, no
  concurrent lanes".

Split summary: intrinsic body ~99.9% of samples at sigma ≈ 25 ns, tail
~0.01-0.06% carrying essentially all of the variance. A single-population
Cpk over this mixture measures the scheduler, not the fold.

## 2. Options

### (a) Quiet-machine measurement protocol

**For**: the charter's own §5.2 close condition is "quiet-machine
re-measurement, no concurrent lanes" — the charter already sanctions this.
DFLSS-DOSSIER §7 records the same close condition. It removes the *known*
confound (fan-out contention), matching what Cpk is supposed to measure:
process capability, not host-load sensitivity.

**Against (laundering risk)**: Cpk over an idle host certifies the bench in
isolation, not the production process under concurrent load. The charter
itself witnessed that the isolated reading was the inflated one
("184-216 ns ... contention-sensitive"). A gate that only ever runs quiet
launders load-sensitivity into a PASS — exactly the move the hot-path
baseline CORRECTION already refuted once ("The earlier '184-216ns met'
reading was MIN-of-batches ... the p99 is the PRD's own criterion and it is
not met", `designs/hot-path-baseline.md` CORRECTION section).

**Recommendation: lawful WITH a tail guard.** Quiet host is the correct
*measurement protocol* for the location/scale statistics (mean, batch Cpk),
because the scheduler tail is an environment property, not a code property.
It is laundering only if the tail is silently discarded. So: run quiet, keep
the full raw sample in the receipt, and gate the tail explicitly (see the
composite verdict below). The receipt must record the load context so a
quiet PASS is distinguishable from a loaded PASS.

### (b) Min-of-N batch statistic (charter ratio-fence form)

The charter's ratio-fence law (relative fence against an independent,
sleep-free-by-construction baseline; paired rounds, min-of-N — per the
no-sleep doctrine the charter inherits) answers this exactly: fence the
statistic, not the raw samples. The bench already computes per-batch p50s
with a spread budget (`VARIANCE_BUDGET_PCT = 5.0` at `benches/tc_hot_01.rs:298`,
batch p50 collection at `:342`), so the machinery exists.

**What N makes Cpk honest**: Cpk over per-batch p50 statistics. With batch
size B, the p50 estimator's sigma shrinks ~√B; with B=10,000 and the observed
intrinsic sigma ≈ 25 ns, sigma(batch p50) ≈ 25/√10000 ≈ 0.25 ns plus
batch-level drift. Cpk >= 1.33 then requires (250 − p50̄)/(3·σ_batch) with
σ_batch near drift-only scale — computable, but it certifies the *body* of
the distribution, which is precisely what min-of-N is for (per the doctrine:
min-of-N fences the intrinsic claim, the raw sample fences the tail claim).

**Recommendation: adopt, paired with an explicit tail gate.** Proposed N:
K=100 batches × B=10,000 iterations (= 1M total, matching the current
n=999,999), statistic = per-batch p50, gate on Cpk over the K batch p50s.
Honesty condition: the same run's raw sample must separately satisfy a
tail-fraction budget (below), or the batch Cpk is a laundered body-only
verdict.

### (c) Thread pinning / isolated core

**Against, primary**: it changes what is measured, not how the measurement is
taken — an isolated core is a production-behavior claim (deployment topology)
smuggled into a measurement harness. The gate would then certify capability
only on hardware no consumer runs. It also fails portability: `std::thread`
exposes no affinity; macOS needs `thread_policy_set` mach calls, Linux needs
`sched_setaffinity` — new unsafe/platform FFI in a bench for a property the
charter does not require.

**For**: it reduces genuine run-to-run sigma of the gate itself, which is a
real cost (the receipts' stddev spans 48 ns to 21.6 µs run to run).

**Recommendation: out of scope for the gate.** Record as an optional
measurement-protocol appendix at most (document the quiet-host + no-lane
protocol instead); if it is ever wanted, it belongs in the release
qualification runner, not `spc_court.sh`.

### (d) USL renegotiation

**Provenance**: USL=250 ns is `BUDGET_NS` (`benches/tc_hot_01.rs:47`), named
by `spc_court.sh:18-20` as "the TC-HOT-01 BUDGET_NS, sql-rdf Rec. section
1.4 receipt hash budget", and descends from the charter §1.2 CTQ
"Ledger write latency — dispatch-carried 0.25 µs"
([CHARTER-DFLSS.md §1.2](../CHARTER-DFLSS.md)). The verbatim charter text is
NOT-MET on disk (charter header: "No verbatim charter text is on disk in
this repo") — the 0.25 µs figure is dispatch-carried and already adjudicated
there as MET-smoke / UNMET-as-gated.

**Against renegotiating now**: the USL's own authority chain is
dispatch-carried with no verbatim source on disk; renegotiating a number
whose provenance cannot be re-derived would mint a new unfalsifiable budget.
Also, 250 ns is *adjacent* (p99 208-250 in four of five receipts), so the
number is not obviously wrong — the variance model is.

**Recommendation: do not renegotiate this window.** Reopen only if the
verbatim charter text lands and shows the 0.25 µs was a location budget
(mean/p50), not a dispersion budget (p99/Cpk) — in which case the correct fix
is to gate p50 against 250 and gate dispersion separately, not to move the
number.

## 3. Recommendation — composite verdict, exact gate change

Keep USL=250 ns and cpk_min=1.33. Change *what the statistic is computed
over*, and add a tail gate so quiet-host batch Cpk cannot launder the tail.

**Gate verdict = PASS iff ALL of:**

1. **Body capability**: Cpk >= 1.33 computed over per-batch p50 statistics
   (K=100 batches × B=10,000 iters, `SPC_BATCH_STAT=1`). Quiet host, no
   concurrent lanes (charter §5.2 close condition).
2. **Tail gate**: raw-sample tail fraction P(sample > USL) <= 10^-4 over the
   full 1M sample (<= 100 of 1M beyond 250 ns). This is the anti-laundering
   term; the receipts show p99.9 = 292 ns shelf, so today this gate is
   honestly red — that red is real and must not be hidden by (1).
3. **p99 <= USL** (unchanged from the current gate,
   `scripts/spc_court.sh` python `passed` line).
4. **Anti-vacuity leg unchanged**: `SPC_INJECT_OUTLIER=1` must still flip
   the verdict (a 2 µs injected outlier both trips Shewhart rule 1 and
   breaks the tail gate), so the composite verdict stays non-vacuous.

**Exact change** (hot-path lane implements; design contract):
`scripts/spc_court.sh` python block — export batch boundaries from the bench
(`TC_HOT_SPC_EXPORT` gains `batch_p50s_ns: [u64; K]` from the existing batch
loop at `benches/tc_hot_01.rs:342`), compute
`cpk_batch = (usl - mean(batch_p50s)) / (3*stddev(batch_p50s))`, compute
`tail_fraction = count(samples_ns > usl)/len(samples_ns)`, add both to the
receipt JSON with the run's load-context field (`load: "quiet" | "loaded"`),
and gate on `cpk_batch >= cpk_min AND tail_fraction <= 1e-4 AND p99 <= usl`.
No sleeps; the bench loop is unchanged in timing behavior (batching already
exists at `benches/tc_hot_01.rs:298-350`).

**What this does not claim**: the process is not capable today. Tail
fraction > 10^-4 fails term (2) under any protocol. The gate change makes
the red *specific* (body-capable, tail-red) instead of the current
undifferentiated Cpk 0.001-0.555 red, and gives the perf owners a separate
close condition per population: intrinsic sigma to <= 20 ns for body Cpk
1.33, and scheduler-tail fraction to <= 10^-4 for the tail term — the
latter being an environment/protocol property, closed by the quiet-host
protocol, not by code.

---

## Amendment 1 — S4 verification corrections (2026-10-22)

Source: [s4-spc-design-verify.md](s4-spc-design-verify.md) (lane S4,
adversarial verification of this file; all five on-disk receipts re-parsed,
arithmetic recomputed, every file:line citation opened). The original
sections above stand unmodified; this amendment supersedes them where they
conflict. Per correction:

### A1.1 Tail-magnitude loophole — add a far-tail/max fence term

S4 stress-test (verify doc, Claim 3): `tail_fraction <= 1e-4` bounds *how
many*, never *how bad*. A run with 60 stalls of 21 ms each per 1M
(tail_fraction 6e-5, well under budget) plus a clean body passes every term
of the §3 composite while a real caller eats 21 ms stalls. This shape exists
on disk today: receipt `...T191548.json` max = 21,413,887 ns (the motivating
witness). p99 <= USL does not catch it (p99 sits in the body).

**The §3 verdict spec gains a new term (after the original four; term 3 is
simultaneously dropped per A1.4, so the net spec is 1, 2, 4, 5):**

> 5. **Far-tail magnitude fence**: `max_sample <= tail_max_budget_ns`, where
>    `tail_max_budget_ns` is an operator parameter of the gate (proposed
>    default: 10x USL = 2,500 ns, i.e. a max 10x the location budget; the
>    value is set at invocation, not baked into the gate). `SPC_INJECT_
>    OUTLIER=1` (2 µs) also trips this term, so the anti-vacuity leg covers
>    it.

Gate conjunction becomes: `cpk_batch >= cpk_min AND tail_fraction <= 1e-4
AND max_sample <= tail_max_budget_ns AND p99 <= usl` — with p99 <= usl now
dropped as redundant (A1.4 below), the implemented conjunction is three
independent terms plus the inject leg.

### A1.2 Quiet-host label: enforcement choice

S4 (Claim 3): the §2(a)/§3 `load: "quiet" | "loaded"` field is written by
the same harness that benefits from the label — a loaded run can self-label
quiet and term (1) passes on confounded data. Two lawful resolutions:

- **Option A — enforce**: add a concurrent-lane detection probe around the
  bench (e.g. sample OS load / run-queue length or a canary-latency probe
  before and after the run; if the probe reads loaded, the receipt is
  stamped `loaded` regardless of the harness's own claim, and term (1) is
  voided). Cost: a probe in `spc_court.sh`, plus its own non-vacuity court.
- **Option B — mark unenforced honestly**: keep the self-reported field but
  annotate the receipt and this design: the quiet label is assertion, not
  evidence; term (1) standing is conditional on operator discipline (charter
  §5.2 protocol), and any dispute re-runs under Option A.

**Recommendation: Option B for this window, Option A as the follow-up.**
The tail gate (term 2) and the new magnitude fence (A1.1) already bite under
load — tails grow under contention — so a laundered quiet label cannot
produce a full PASS on confounded data; it can only inflate the body
statistic, which the re-run condition covers. Option A's probe is real
machinery that should land with its own court, not ride along in this
change; until it lands the receipt must say `load_provenance: "self-
reported"`.

### A1.3 Citation corrections

Per the verify doc (Claims 1 and 4, and the line-number check):

1. **Shelf breadth**: p99.9 = 292 ns in **four** receipts, not three
   (spc-2026-10-22, T173758, T173809, T184929; T191548's p999 is 250). The
   §1 shelf sentence reads "identical 291/292 across three receipts"; the
   corrected claim is 292 in 4/5 independent runs — the shelf is stronger
   than originally claimed.
2. **Tail-fraction range**: beyond ±3σ the receipts span **16-580 per
   million**, not 27-580 (T173809 = 16, T173758 = 18; the design table's
   "n/a / n/r" cells were wrong — both receipts carry
   `shewhart.rule1_beyond_3sigma.count`). The §1 "27-580" and the table
   n/a cells are superseded by this range.
3. **Intrinsic-body claim**: §1's "p99=208-250 across all five receipts" is
   wrong — T173758/T173809 have p99 = 291/292 (the tail already dominates
   p99 there). Corrected form: p95 = 208-209 in all five; p99 <= 250 in 3/5.
4. **Ratio-fence attribution**: the ratio-fence law ("relative fence against
   an independent, sleep-free-by-construction baseline — paired rounds,
   min-of-N") lives in `~/.claude/rules/no-sleep-testing.md`, NOT in
   CHARTER-DFLSS.md (verify doc grep: zero hits for ratio/sleep in the
   charter). §2(b)'s "The charter's ratio-fence law" is refuted as cited;
   read it as "the no-sleep rule's ratio-fence law". The reproduced fence
   form itself is faithful and stands.
5. **Line numbers**: `scripts/spc_court.sh` USL/CPK_MIN defaults are at
   :31-32 (not :18-24/:37; :18-24 is the comment header); the python
   `passed` line is :107. The `benches/tc_hot_01.rs` citations (:47, :298,
   :342) were confirmed as-is.
6. **Arithmetic**: all five cpk fields recomputed and match exactly
   (0.2948, 0.0045, 0.0032, 0.555, 0.001); intrinsic-Cpk math (25 ns fit,
   1.07, 20 ns ceiling, 0.25 ns batch-p50 sigma) confirmed and robust over
   the mean 166-170 range. Unchanged.

### A1.4 Term 3 dropped as redundant

S4 (Claim 3): `p99 <= USL` is strictly implied by `tail_fraction <= 1e-4`
(100/1M beyond USL puts p99.99 <= 250, hence p99 <= 250); while term 2
exists, term 3 carries no independent information. The §3 note "(unchanged
from the current gate)" on term 3 is dropped from the implemented contract —
term 3 is removed, not carried, and the §3 "Exact change" conjunction
loses the `AND p99 <= usl` clause accordingly.
