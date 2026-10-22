# Defect Pareto — witnessed vs charter (DFLSS Define-lane B record)

The charter's four-family Pareto (48.2/27.4/16.1/8.3) is ASSERTED, not
witnessed. Real ledger: 22 witnessed defects this window (every one with an
on-disk witness), classified:

| Family | n | Real % | Charter % | Verdict |
|---|---|---|---|---|
| Injection | 10 | 45.5% | 48.2% | NOT WITNESSED (ordinal survives: injection #1; ~2.7pts overstated) |
| Schema drift | 4 | 18.2% | 27.4% | REFUTED (~1.5x over; strict reading = 9.1%) |
| Parser stalls | 3 | 13.6% | 16.1% | NOT WITNESSED (~2.5pts inflated) |
| Audit | 1 | 4.5% | 8.3% | REFUTED (~1.8x over) |
| Other (test infra) | 4 | 18.2% | — | STRUCTURAL MISS: charter has no bucket for it (2 stale targets, mutation-court pre-abort, conformance flip fixture) |

## Denominator correction
The wave receipt's "9 defects found + fixed" is a FIXED-ONLY count. True
witnessed population: 22. OPEN (6): C2 raw-SQL bypass (SQLRDF-17), E2 escape
(no close receipt), digest-pin staleness (re-pins at integration),
mutation-court abort class, conformance-flip fixture, tpch generator drift
(filed). Fix rate: 16/22 = 72.7%.

## Sensitivity
Classification granularity moves families ±1-2 items (±5pts), but under any
defensible bucketing injection stays <48% and audit stays <8.3% — the two
REFUTED claims cannot be rescued.

## Note on the ring restart-watermark deviation (accepted)
The ring lane implemented restart-gap reconciliation with
watermark = restart_seq instead of the prescribed live seq AtomicU64 — the
prescribed form double-covered 911 buffered sequences (falsified by the
joint court's gap-coverage delta). Empirical output over prescription,
witness recorded adjacent to the code. Accepted.

## Pointer
final consolidated census: see [DFLSS-DOSSIER.md §7](../DFLSS-DOSSIER.md) (41 unique = 16 fixed + 6 ticketed-open + 3 open-by-design + 16 open-unticketed).
