# Capability-surface benchmark

Measure-lane census of every registered ABI capability plus the native
`duckdb` modes. Chicago: real `graphlaw::abi::call`, positive-control
workloads from `registry/op-examples.json`, no mocks.

## Command

```bash
CARGO_TARGET_DIR=target-cap-bench \
CAP_BENCH_JSON=benches/reports/capability_surface.json \
CAP_BENCH_WARMUP=5 CAP_BENCH_ITERS=25 CAP_BENCH_BATCHES=10 \
cargo bench --bench capability_surface -- --nocapture
```

## Machine

`x86_64 / linux / 4 cores / rustc 1.98.0 (88d9e12ae 2026-08-18)`  
graphlaw `26.10.28` · plan `warmup=5 iters/batch=25 batches=10` (min-of-batches)

## Verdict

**COVERED 18 legs (14 registry ops + 4 duckdb modes); all ok.**  
Fastest: `sniff.turtle` **2 078 ns/op**.  
Slowest: `duckdb.exec.create` **14 115 943 ns/op** (~14.1 ms).  
Span: **6793×**.

DuckDB mode legs time only the mode call; session open / table seed / close
sit outside the stopwatch.

Industry reading (CTQs, statutory ceilings, peer bands):  
[`docs/reference/capability-surface-industry-context.md`](../../docs/reference/capability-surface-industry-context.md).

## Rank (fastest → slowest)

| # | Leg | ns/op | req B |
|---|---|---:|---:|
| 1 | sniff.turtle | 2 078 | 63 |
| 2 | duckdb.stream.page | 7 116 | 121 |
| 3 | n3.reason | 9 558 | 97 |
| 4 | capabilities.default | 11 015 | 21 |
| 5 | policy.accepted | 15 021 | 418 |
| 6 | parse.turtle | 23 018 | 118 |
| 7 | canonical.flat | 24 763 | 131 |
| 8 | convert.turtle | 25 692 | 134 |
| 9 | shex.shexc | 27 251 | 237 |
| 10 | entail.simple | 43 303 | 187 |
| 11 | datalog.ancestor | 63 152 | 304 |
| 12 | sparql.select | 105 921 | 201 |
| 13 | shacl.conforming | 129 002 | 317 |
| 14 | law.shacl-n3 | 226 310 | 400 |
| 15 | hooks.inline-pack | 891 146 | 540 |
| 16 | duckdb.close.session | 1 364 813 | 62 |
| 17 | duckdb.open | 11 594 843 | 58 |
| 18 | duckdb.exec.create | 14 115 943 | 386 |

## Embedded run output

```
=== GraphLaw capability-surface benchmark ===
[MACHINE] x86_64 / linux / 4 cores / rustc 1.98.0 (88d9e12ae 2026-08-18)
[PLAN] warmup=5 iters/batch=25 batches=10 (min-of-batches)
[CMD] CARGO_TARGET_DIR=target-cap-bench cargo bench --bench capability_surface
[SOURCE] registry/op-examples.json positive_control ok + duckdb open/exec/stream/close
[capabilities.default  ]      11015 ns/op  (positive_control; req 21 B)
[sniff.turtle          ]       2078 ns/op  (positive_control; req 63 B)
[parse.turtle          ]      23018 ns/op  (positive_control; req 118 B)
[convert.turtle        ]      25692 ns/op  (positive_control; req 134 B)
[canonical.flat        ]      24763 ns/op  (positive_control; req 131 B)
[sparql.select         ]     105921 ns/op  (positive_control; req 201 B)
[shacl.conforming      ]     129002 ns/op  (positive_control; req 317 B)
[shex.shexc            ]      27251 ns/op  (positive_control; req 237 B)
[n3.reason             ]       9558 ns/op  (positive_control; req 97 B)
[entail.simple         ]      43303 ns/op  (positive_control; req 187 B)
[datalog.ancestor      ]      63152 ns/op  (positive_control; req 304 B)
[hooks.inline-pack     ]     891146 ns/op  (positive_control; req 540 B)
[law.shacl-n3          ]     226310 ns/op  (positive_control; req 400 B)
[policy.accepted       ]      15021 ns/op  (positive_control; req 418 B)
[duckdb.open           ]   11594843 ns/op  (mode call only; seed outside stopwatch; req 58 B)
[duckdb.exec.create    ]   14115943 ns/op  (mode call only; seed outside stopwatch; req 386 B)
[duckdb.stream.page    ]       7116 ns/op  (mode call only; seed outside stopwatch; req 121 B)
[duckdb.close.session  ]    1364813 ns/op  (mode call only; seed outside stopwatch; req 62 B)

=== rank (fastest → slowest, min-of-batches ns/op) ===
 1. sniff.turtle                 2078 ns/op  (63 B req)
 2. duckdb.stream.page           7116 ns/op  (121 B req)
 3. n3.reason                    9558 ns/op  (97 B req)
 4. capabilities.default        11015 ns/op  (21 B req)
 5. policy.accepted             15021 ns/op  (418 B req)
 6. parse.turtle                23018 ns/op  (118 B req)
 7. canonical.flat              24763 ns/op  (131 B req)
 8. convert.turtle              25692 ns/op  (134 B req)
 9. shex.shexc                  27251 ns/op  (237 B req)
10. entail.simple               43303 ns/op  (187 B req)
11. datalog.ancestor            63152 ns/op  (304 B req)
12. sparql.select              105921 ns/op  (201 B req)
13. shacl.conforming           129002 ns/op  (317 B req)
14. law.shacl-n3               226310 ns/op  (400 B req)
15. hooks.inline-pack          891146 ns/op  (540 B req)
16. duckdb.close.session      1364813 ns/op  (62 B req)
17. duckdb.open              11594843 ns/op  (58 B req)
18. duckdb.exec.create       14115943 ns/op  (386 B req)

[VERDICT] COVERED 18 legs (14 registry ops + 4 duckdb modes); all ok
[VERDICT] fastest=sniff.turtle (2078 ns)  slowest=duckdb.exec.create (14115943 ns)  span=6793.2x
[NOTE] duckdb mode legs time only the mode call; session open/seed/close sit outside the stopwatch.
[WROTE] /workspace/benches/reports/capability_surface.json
```

## Machine receipt

`benches/reports/capability_surface.json` (`schema: graphlaw.capability-surface-bench/1`).
