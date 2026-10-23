# S13 skeptic — verification of Amendment 1 (spc-red-remediation-design.md:193-290) vs s4-spc-design-verify.md

Coordinator-filed from lane S13's inline report (the lane's own write was
DISK-REFUSED at 18.1 GiB free; contents landed verbatim-class here).
Method: all six on-disk receipts re-parsed (python json, exit 0),
arithmetic recomputed, cited script lines read. No gate runs.

## Verdicts per correction

| # | S4 correction | Incorporation | Verdict |
|---|---|---|---|
| 1 | Shelf 292 ns in 4 receipts, not 3 | A1.3 item 1 | ACCURATE |
| 2 | Tail 16-580/1M, rule1 18/16 exist | A1.3 item 2 | ACCURATE |
| 3 | p99=208-250 across five is wrong | A1.3 item 3: "p95=208-209 in all five; p99≤250 in 3/5" | ACCURATE |
| 4 | Ratio-fence = no-sleep rule's, not charter's | A1.3 item 4: attribution corrected | ACCURATE |
| 5 | spc_court.sh :31-32 / :107 | A1.3 item 5 | ACCURATE (re-verified on disk) |
| 6 | Term-3 redundancy | A1.4 drops term 3 | ACCURATE, consistent |
| 7 | Magnitude-blind tail gate | A1.1 max fence term | CORRECT with 1 defect (below) |
| 8 | Self-reported quiet label | A1.2 Option B | HONEST |

Receipt re-parse (exit 0):

| receipt | p95 | p99 | p999 | max_ns | rule1 |
|---|---|---|---|---|---|
| spc-2026-10-22.json | 209 | 250 | 292 | 29,759 | 70 |
| T173758 | 208 | 291 | 292 | 4,075,519 | 18 |
| T173809 | 209 | 292 | 292 | 6,639,615 | 16 |
| T184929 | 208 | 250 | 292 | 39,615 | 580 |
| T191548 | 208 | 209 | 250 | 21,413,887 | 27 |
| -inject.json | 292 | 333 | 1167 | 12,435,455 | 120 |

On-disk line check: scripts/spc_court.sh:31-32 is
USL_NS="${SPC_USL_NS:-250}" / CPK_MIN="${SPC_CPK_MIN:-1.33}"; :107 is
passed = cpk >= cpk_min and p99 <= usl. Both match A1.3 item 5.

Arithmetic (python, exit 0):
1. Motivating shape: 60×21.4 ms per 1M → tail_fraction 6e-5 ≤ 1e-4,
   passes tail gate — stress shape sound.
2. Fence catches the witness: 21,413,887 ns > 2500 ns default → FAIL.
3. DEFECT (A1.1 anti-vacuity leg): "SPC_INJECT_OUTLIER=1 (2 µs) also
   trips this term" is arithmetically false as stated —
   scripts/spc_court.sh:59 wires TC_HOT_INJECT_OUTLIER_NS=2000, and
   2000 ns < 2500 ns fence. It holds only accidentally: the on-disk
   inject receipt max is 12,435,455 ns (spin overshoot past the 2 µs
   target). Fix: (a) default fence < 2000 ns, (b) raise inject above
   fence (e.g. 10 µs), or (c) re-word to cite the observed 12.4 ms
   overshoot, not the 2 µs nominal.
4. Conjunction consistency: net spec 1/2/4/5 internally consistent
   (original §3 was four terms; fence = term 5; term 3 dropped). No live
   prose still treats term 3 as operative.
5. A1.4 redundancy argument sound: tail_fraction ≤ 1e-4 over 1M ⇒
   ≤100 beyond USL ⇒ p99.99 ≤ USL ⇒ p99 ≤ USL.

A1.2 honesty: Option B carries load_provenance: "self-reported"
(L247-248), conditional standing, Option A re-run path. Honest.

## Bottom line

All five S4 corrections incorporated accurately; reduced conjunction
internally consistent; fence catches the T191548 witness (21.4 ms ≫
2500 ns). One real defect: A1.1's anti-vacuity arithmetic (2 µs vs
2500 ns fence) is false and holds only via spin overshoot. Owner: the
hot-path lane, at implementation time.
