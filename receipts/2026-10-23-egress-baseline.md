# Egress baseline receipt — sql_rdf_egress (2026-10-23)

Subject: /Users/sac/graphlaw @ main (post 0d11bc3), lane build root `target-lane-eg`.

## Command + exit

```
CARGO_TARGET_DIR=target-lane-eg cargo bench --bench sql_rdf_egress
# exit code: 0
# build: Finished `bench` profile [optimized] target(s) in 10m 23s
# runner: target-lane-eg/release/deps/sql_rdf_egress-a8e2d5de9bab70ef
# Timer precision: 41 ns
```

## REAL Divan output (verbatim)

```
sql_rdf_egress            fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ columnar_digest_proxy  1.08 ms       │ 120.6 ms      │ 1.496 ms      │ 3.074 ms      │ 100     │ 100
│                         148.1 MB/s    │ 1.326 MB/s    │ 106.9 MB/s    │ 52.03 MB/s    │         │
├─ egress_result_triples  1.298 ms      │ 7.384 ms      │ 1.354 ms      │ 1.437 ms      │ 100     │ 100
│                         1.386 GB/s    │ 243.7 MB/s    │ 1.328 GB/s    │ 1.252 GB/s    │         │
╰─ ntriples_format        8.376 ms      │ 16.05 ms      │ 9.246 ms      │ 9.387 ms      │ 100     │ 100
                          214.8 MB/s    │ 112.1 MB/s    │ 194.6 MB/s    │ 191.7 MB/s    │         │
```

## Baseline figures (medians, 100 samples × 100 iters)

- N-Triples render (`ntriples_format`): **9.246 ms** median (8.376 ms fastest)
- Raw materialization (`egress_result_triples`): **1.354 ms** median (1.298 ms fastest)
- Columnar digest proxy (`columnar_digest_proxy`): **1.496 ms** median
- Columnar-vs-N-Triples ratio: 9.246 / 1.354 = **6.83×**

## Audit-finding reconciliation

The prose figures (8.46 ms / 1.34 ms / 6.31×) do NOT reproduce on this subject:

| metric | prose claim | measured median | delta |
|---|---|---|---|
| ntriples_format | 8.46 ms | 9.246 ms | +9.3% |
| egress_result_triples | 1.34 ms | 1.354 ms | +1.0% |
| ratio | 6.31× | 6.83× | +8.2% |

Raw materialization matches within noise; the N-Triples render median (and therefore
the ratio) is slower than claimed. Actual ratio on this subject is **6.83×**, not
6.31×. Coordinator should correct the whitepaper figure or cite the machine + sample
condition of the original 6.31× run.

## Standing

ALIVE for the measured numbers above (real Divan output, exit 0, 100/100 samples).
The 6.31× prose figure is UNVERIFIED-as-of-this-receipt; superseded by 6.83× here.
