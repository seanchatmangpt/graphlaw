# shacl_boundary_cliff

WP P07 (hard SHACL caps): measured boundary cliff, the four caps pinned
from those measurements, and the boundary-court receipts. Every number
below is from a real run of `benches/shacl_boundary_cliff.rs` (exit 0,
2026-10-26, macOS aarch64) — no extrapolated or interpolated cell.

## Caps (final, measured)

| cap | value | measured basis |
|---|---|---|
| `MAX_SHACL_SHAPES_BYTES` | 131,072 (128 KiB) | see Byte cliff |
| `MAX_SHACL_QUADS` | 16,384 | see Quad cap |
| `MAX_SHACL_DEGREE` | 128 | see Degree cliff |
| `MAX_SHACL_VALIDATION_DEPTH` | 16 | see Depth cliff (unchanged) |

Raw runs: `target-arr-P07-shacl` lane; distilled below. Cells are
nearest-rank p50/p95/p99 over per-iteration `Instant` samples. Post-cap
run executed under a 20-lane concurrent array load (visible as p99
jitter); cliff LOCATIONS are stable across both runs.

## Depth cliff (cap 16, unchanged)

Pre-cap baseline (guard = depth-only `check_shape_depth`), quiet machine:

| depth | guard p99 | engine p99 |
|---|---|---|
| 16 | 67.4 us | 635.1 us |
| 17 | 76.7 us | 720.4 us (refused typed at the guard) |

Engine p99 at the cap is 635 us — 8x under the 5 ms Art. 14 ceiling; the
purrdf stack-overflow threshold (~48 nesting levels) is 3x past the cap.
Depth-17 refuses typed at the guard in ~80 us.

## Degree cliff (cap 128)

Pre-cap baseline (no degree cap existed — this sweep found it):

| degree | guard p99 | engine p99 |
|---|---|---|
| 128 | 526.1 us | 2.9 ms |
| 256 | 1.3 ms | 3.3 ms |
| 512 | 3.2 ms | 9.2 ms (first > 5 ms) |
| 1024 | 10.0 ms | 19.6 ms |

First engine-p99 crossing of 5 ms: degree 512. Cap = 128 sits 4x under
the crossing cell; at-cap engine p99 is 2.9 ms (under the Art. 14
ceiling). The guard's own walk also crossed 5 ms by degree 1024 (10.0 ms
p99), so the cap protects the boundary check itself, not only the engine.

## Byte cliff (cap 128 KiB)

Pre-cap baseline (no byte cap existed):

| bytes | guard p99 | engine p99 |
|---|---|---|
| 16 KiB | 1.1 ms | 3.5 ms |
| 64 KiB | 5.0 ms | 13.5 ms (first > 5 ms) |
| 256 KiB | 16.1 ms | 67.1 ms |
| 16 MiB | 1,153.6 ms | 4,930.0 ms |

First engine-p99 crossing of 5 ms: 64 KiB (13.5 ms). The smallest
measured-grid cap clearing the shipped trusted canonical surface
(`ontologies/sql-ast.ttl` + `ggen/sql-dialect/statement_classes.ttl`
concat = 66,182 bytes, fed through the guard on every `compile_turtle`)
is 128 KiB — ~2x the canonical surface. The 512 KiB placeholder was
REJECTED by the measurement (it would admit 256 KiB-class inputs at
67 ms engine p99); the honest consequence is recorded in the Art. 14
section.

Post-cap sweep (typed refusal at the O(1) byte check, microseconds):

| bytes | verdict | guard p99 | engine p50 |
|---|---|---|---|
| 127.9 KiB (130,982 B) | admit | 13.0 ms | 38.5 ms |
| 255.9 KiB | REFUSED typed | 833 ns | 80.5 ms (engine leg, guard bypassed) |
| 1 MiB | REFUSED typed | 791 ns | 473.4 ms |
| 4 MiB | REFUSED typed | 958 ns | 1,324.7 ms |
| 16 MiB | REFUSED typed | 833 ns | 6,041.3 ms |

The guard reduces the 16 MiB denial-of-service from a 6.0 s engine call
to a sub-microsecond typed refusal (4 orders of magnitude).

## Quad cap (16,384)

Defense-in-depth backstop. The densest legal Turtle is ~19 bytes/quad
(`ex:a ex:b ex:c .`), so 16,385 quads need ~300 KB — the 128 KiB byte cap
gates first for ANY parseable Turtle (witnessed by the court's ordering
test: a 16,385-quad dense graph refuses `Bytes`, fail-closed ordering
bytes -> quads -> degree -> depth). The cap bounds the guard's own
adjacency build + DFS against future non-Turtle shape encodings. The
64 KiB blob cell carries 3,320 quads; the canonical concat carries 1,188.

## At-cap Art. 14 (honest statement)

The Art. 14 5 ms leg ceiling binds the statutory candidate class
(KB-class shapes graphs; the SB-50MS Art. 14 leg validates a ~1 KB
shapes graph in microseconds and is untouched by the caps). At-cap admit
of a MAXIMALLY-large legal shapes graph (127.9 KiB blob) records:

| input | guard p99 | engine p99 (leg cost) |
|---|---|---|
| bytes-at-cap blob | 17.2 ms | 55.1 ms |
| bytes-at-cap single-focus | 11.9 ms | 41.6 ms |
| degree-at-cap star | 575.7 us | 5.3 ms |
| depth-at-cap chain | 208.4 us | 1.7 ms |
| all-caps composite | 18.3 ms | 78.9 ms |

The bytes-at-cap worst case sits ABOVE the 5 ms hot-leg ceiling — this is
the measured trade the canonical-surface constraint forces (a cap at the
64 KiB cliff would cap out `compile_turtle`'s own 66,182-byte trusted
surface). What the caps eliminate is the UNBOUNDED region: pre-cap, the
same maximally-large legal input class cost 4.9 s at 16 MiB and the
guard's own walk 1.15 s (guard-DoS). Legal KB-class input (the real
candidate class) rides three decades under the byte cap and stays
microsecond-class end to end.

## Canonical trusted surface (must admit)

| cell | bytes | quads | verdict | guard p50 | engine p50 |
|---|---|---|---|---|---|
| sql-ast.ttl + statement_classes.ttl | 66,182 | 1,188 | admit | 2.0 ms | 4.5 ms |

Witnessed admitting under all four caps (bench assert + court test
`canonical_compile_surface_admits_under_the_caps` via real
`compile_turtle` + seal verification).

## Courts (all green, real exits)

- `tests/shacl_boundary_cliff_court.rs`: 8 passed / 0 failed (exit 0):
  at-cap admits at all three sites; over-cap bytes/degree/depth refuse
  typed at all three sites (law violation component, ABI `ResourceLimit`
  envelope with limit/observed/max, sql-rdf MalformedAst naming the
  limit); quads-arm typed witness + bytes-gate-dense-Turtle ordering;
  canonical compile surface admits with seal; guard-removal flip (raw
  engine admits exactly what the guard refuses); cyclic shapes stay
  admitted within budget.
- `tests/shacl_depth_guard_court.rs`: 6 passed / 0 failed (exit 0).
- Consumer sweep at the three guarded sites: law_state (8/8),
  refusal_details (7/7), test_cold_chain_ocel (1/1),
  test_parquet_and_builtin_eval (3/3), sql_rdf_dialect (36/36),
  chicago_boundary_constraints (4/4), abi_bounds_court (6/6).

## Not my diff (classified, loud)

- op_differential 16 failures, chicago_fibo 4, wasm_boundary_fuzz 1:
  all panics inside the dual-runtime harness at
  `frozen-duckdb-sys-2.0.2/build.rs:101` — the wasm32-wasip1 link
  (wasi-sdk/frozen-duckdb wasm build) fails at infrastructure level.
  Even the pure-parse control (`a_control_parse_and_canonical_succeed_
  on_the_base_fixture`) fails there; the same suites' native legs pass.
  Environmental / pre-existing, not session-introduced.
- corpus_conformance 8 vecgen failures: corpus committed-table
  byte-currency, zero SHACL surface involvement — pre-existing in-flight
  tree state (another lane's domain).

## Cross-lease disclosure

`tests/shacl_depth_guard_court.rs` is not in this lane's lease, but the
new byte cap broke `guard_is_safe_on_a_small_stack_thread` (4,096-link
chain = ~463 KiB now refuses Bytes before Depth). Minimal unblock per
the compile-freeze SLA: chain reduced to 1,024 links (~118 KiB, still
64x the depth cap, under the byte cap), with an in-file comment naming
the P07 reason. Disclosed here for both lanes' receipts.
