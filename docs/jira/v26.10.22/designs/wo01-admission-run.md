# WO-01 — full-shape-corpus admission run (lane WO-01, 2026-10-22)

Subject: /Users/sac/graphlaw @ main 615dbfa (working tree, unstaged). Shapes:
docs/jira/v26.10.22/mega/tc-mega-fibo-01/ontology/shapes/ (7 files).
Corpus: docs/jira/v26.10.22/mega/tc-mega-fibo-01/corpus/.

## Machine disclosure

```
arch           = aarch64
os             = macos
cores          = 16
rustc          = 1.97.0 (2d8144b78 2026-07-07)   # NOTE: deviates from COURT-SPEC witness 1.96.0; PATH-provided toolchain
benchmark_mode = debug (TYPED REFUSAL class per COURT-SPEC 0 — release gate DISK-BLOCKED, see below)
```

## Status: PARTIAL -> ALIVE (release witnessed). Corpus-side twin FINDING stands.

All numbers below are debug-build wall-clock (Instant) from
/tmp/wo1run (scratch harness, path-dep on graphlaw, repo untouched except this receipt).
Release numbers below (ALIVE); debug-class numbers retained at the bottom for the
disk-pressure audit trail.

## Release run (benchmark_mode = release, EXIT=0, two runs)

| Scale | parse | parse_shapes | validate | total | conforms |
|---|---|---|---|---|---|
| smoke 1k-leg (301,699 B) | 10 ms | 8 ms | 2 ms | 21 ms | true, 0 violations |
| full 100k-leg (24,790,702 B) | 962–965 ms | 9 ms | 232–242 ms | 1,207–1,213 ms | true, 0 violations |

- Gate call validate_ast_shacl(smoke, shapes): ok=true, 27.9–29.5 ms release.
- Derived cold-path per-leg mean, full: **12.07–12.13 µs/leg** (parse 9.6 µs/leg,
  SHACL validate 2.3 µs/leg). FIRST 100k-scale cold admission witnessed. NOT CTQ M1
  (0.082 µs is per-admission hot-path fold — different operating point; M1 stays
  UNVERIFIED).
- Shapes parse 8–9 ms once per process, ~0.09 µs/leg amortized at 100k.

## Debug-class run (typed-refusal class per COURT-SPEC 0; audit trail)

## Commands + exits

- harness: `/tmp/wo1run` (`cargo build` exit 0; `./target/debug/wo1run` EXIT=0, two runs)
- calls: `graphlaw::sql_ast::validate_ast_shacl(corpus, shapes)` (the repo admission path;
  shapes ARE a parameter — NO shapes-graph API gap found at src/sql_ast.rs:1031)
- per-phase: graphlaw::purrdf::parse_dataset / graphlaw::shacl::engine::parse_shapes /
  validate_dataset (the exact primitives validate_ast_shacl uses)

## Shape counts (byte-derived, re-derived not trusted)

- shapes_bytes = 131,161 (7 files concatenated in numbered order)
- node_shapes = 367 (matches generator claim)
- constraints = 1,741 (matches generator claim; keys = sh:minCount/maxCount/datatype/
  pattern/in/node/class/closed/minInclusive/maxInclusive/qualifiedValueShape)

## Admission numbers (wall-clock, two runs)

| Scale | parse | parse_shapes | validate | total | conforms |
|---|---|---|---|---|---|
| smoke 1k-leg (301,699 B) | 93–96 ms | 127–131 ms | 28 ms | 248–256 ms | true, 0 violations |
| full 100k-leg (24,790,702 B) | 7.89–7.94 s | 127–129 ms | 2.21–2.24 s | 10.23–10.30 s | true, 0 violations |

- Gate call validate_ast_shacl(smoke, shapes): ok=true, 292–312 ms.
- Derived per-leg cold-path mean, full: 10.23–10.30 s / 100,000 legs = **102–103 µs/leg**
  (parse-dominated: parse 7.9 s = 79 µs/leg; SHACL validate 22 µs/leg). This is the FIRST
  100k-scale cold admission witnessed; it is NOT CTQ M1 (0.082 µs = hot-path per-admission,
  different operating point). Honest status: cold-path per-leg 102–103 µs at debug class;
  M1 stays UNVERIFIED.
- shapes parse is once-per-process (127 ms), amortized to ~1.3 µs/leg at 100k.

## Twin verdicts

- Designed falsifier (ontology/twins vs anchor shape 51_precision_anchor_41209.ttl):
  benign ok=TRUE; mutated ok=FALSE with typed refusal:
  `SHACL AST structural violation: Node <reg#Leg-41209>: USD monetary amounts carry at most 2
  decimal places; leg #41209 must not exceed ISO 4217 minor units`
  → the anchor court FLIPS. Non-vacuous.
- **FINDING (WO-01 gate defect — corpus-side twin is vacuously admitted)**:
  corpus/antivacuity-leg-41209-malformed.ttl validates ok standalone AND appended to the
  smoke corpus (ok=true, 0 violations). Cause: vocabulary mismatch — all shapes target
  `reg:` classes (sh:targetClass reg:LEIHolder / reg:MoneyAmount_* / ...), but the corpus
  and the corpus twin use only `tcg:` classes (tcg:CDS_TRANCHE etc.). Zero focus nodes are
  selected → trivial conforms. The corpus malformed twin (tcg:notional "1234567.8901",
  4 fractional digits) escapes enforcement entirely. The malformed twin as it stands
  cannot flip any shape → the corpus-side anti-vacuity twin is currently VACUOUS.
  Fix direction: either the corpus must assert reg: types (reg:MoneyAmount_USD nodes with
  reg:amount), or gen_shapes.py must emit tcg:-targeting shapes.

## Blockers

1. **DISK-BLOCKED (release gate)**: PreToolUse DISK-REFUSED hook fired twice
   (19.8 GiB → 16.7 GiB free < 20 GiB threshold; now 2.4 GiB free, 83% capacity).
   df receipt: `/dev/disk3s1s1 926Gi 11Gi 2.4Gi 83%`. Release build of graphlaw +
   frozen-duckdb (~3-4 GiB) refused. Proposed purge candidates checked for staleness:
   target-lane159 (1.4G, idle 2.5h, no referencing process) and target-lane-v6 (301M,
   idle 1h15m, no referencing process) — purge was DENIED by permissions; coordinator
   must own this deletion.
2. Release-mode rerun of this exact harness is the only remaining step to convert this
   receipt from PARTIAL to ALIVE.

## Honesty notes

- constraints counted from bytes with the README\'s declared key list — matches 1,741.
- parse_dataset timed once per scale; shapes parsed once per admit() call — real cost.
- numbers are two-run range, not median-of-N; cold path, page cache warm.
