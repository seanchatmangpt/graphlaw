# P10 W2 wasm/eh battery receipt (execution lane L5, 2026-10-24) — BLOCKED

- Date: 2026-10-24 (local)
- Subject: /Users/sac/graphlaw @ main `a00770b4998a55ed4ee397e3e2a1f5a880c51db0`
- Pins:
  - `courts/code-partition/axioms.blake3` b3 `2add10ce9ff782e33b75de099cba8dc8a114ab759855f93a7be85abb18581506`
  - `courts/code-partition/manifest.json` b3 `cb122a7b57fe8b4db62f81edca7a552ee2a0dbb43157b8c829c35f2309c6155d`
  - `priv/graphlaw.wasm` b3 `58a4409505f32b5d9aebc8c0ce2e2adeaba05c8c4b55f951a244ab4ad7d2280a` (== dispatched `58a44095…`)
- Logs (b3):
  - `/tmp/bat-L5a.log` `106156ed46d7166d6a5f620d9fbaa5a77c93045bd5af4fb2909d95a21816e9d9`
  - `/tmp/bat-L5b.log` `775685f27dace9b6b8399323e692a9b360209c11dbb3d32bbc1611393d85113c`
- Standing: **BLOCKED (shared compile break)** — zero tests executed across both legs. Not PASS, not CLASSIFIED.

## W0 preflight (all PASS)

| check | observed | verdict |
|---|---|---|
| `df -h /System/Volumes/Data` | 60 GiB avail (926Gi/844Gi used, 94%) | PASS (≥20 GiB) |
| `b3sum priv/graphlaw.wasm` | `58a44095…d2280a` | PASS (matches pin) |
| `python3 -c "import pm4py"` | pm4py 2.7.22.1 | PASS |

## Root cause (shared, both legs)

Working tree does not compile: `vendor/purrdf-sparql-eval` (lib) fails with **174
errors** of unresolved imports against its crates.io sibling crates
(`purrdf_iri::vocab`, `purrdf_xsd::ieee`, `purrdf_core::TermBox`,
`purrdf_hash`, `purrdf_lex`, `purrdf_stack`,
`purrdf_sparql_algebra::{Child,Chain,NodeRef,…}`). Every cargo leg on this tree
hits the same wall; the log-level error signature is identical across L5a, L5b,
and the W3 build logs.

Owner evidence: `vendor/purrdf-sparql-eval/src/lib.rs` last write `1792873263`,
still unchanged through `1792874430` (≈19.5 min, no further writes; a 5-minute
bounded mtime poll saw zero new writes). The compile-freeze SLA window
(~10 min) expired with the tree left broken — owned by whichever lane is
mid-flight on `vendor/purrdf-sparql-eval` (58 files, +55878/−11677 vs HEAD per
`git diff --stat HEAD -- vendor/`).

## Leg 5a — BLOCKED

Command (one invocation, as dispatched):

```
GRAPHLAW_WASM=/Users/sac/graphlaw/priv/graphlaw.wasm \
CARGO_TARGET_DIR=/Users/sac/graphlaw/target-battery-finance \
cargo test --locked --test wasm_abi --test wasm_duckdb_op --test wasm_memory_wall \
  --test wasm_boundary_fuzz --test wasm_n3_admission --test wasm_governance_preflight \
  --test attach_bypass_court --test wasm_env_isolation_court --test wasm_frame_bound_court \
  --test sniff_court > /tmp/bat-L5a.log 2>&1; echo "EXIT:$?" >> /tmp/bat-L5a.log
```

Verbatim result splice:

```
error[E0433]: cannot find `vocab` in `purrdf_iri`
error[E0432]: unresolved import `purrdf_xsd::ieee`
error: could not compile `purrdf-sparql-eval` (lib) due to 174 previous errors
Some errors have detailed explanations: E0220, E0405, E0425, E0432, E0433, E0603.
EXIT:101
```

`grep -c "^test "` on the log = **0**. No test binary was reached.

| target | expected (dispatch) | observed | verdict |
|---|---|---|---|
| wasm_abi | 25 ok | 0 ran | BLOCKED |
| wasm_duckdb_op | 9 ok | 0 ran | BLOCKED |
| wasm_memory_wall | 8 ok | 0 ran | BLOCKED |
| wasm_boundary_fuzz | 2 ok | 0 ran | BLOCKED |
| wasm_n3_admission | 3 ok | 0 ran | BLOCKED |
| wasm_governance_preflight | 2-3 CLASSIFIED-RED | 0 ran | BLOCKED |
| attach_bypass_court | 3 ok | 0 ran | BLOCKED |
| wasm_env_isolation_court | 3 ok | 0 ran | BLOCKED |
| wasm_frame_bound_court | 4 ok | 0 ran | BLOCKED |
| sniff_court | 12-14 ok | 0 ran | BLOCKED |

Boundary re-probes of the same root cause (each a dedicated invocation, exit
read without a pipe): `cargo check --locked` EXIT:101 (same 174 errors) at
clock `1792873917` and again at `1792874060`; unchanged failure → no third
rerun without a new hypothesis.

## Leg 5b — BLOCKED (plus disclosed dispatch arg defect)

Dispatched command was malformed: `--test-threads=1` is a libtest flag, not a
cargo flag. First attempt verbatim:

```
error: unexpected argument '--test-threads' found
EXIT:1
```

EXIT:1 there is an argument-parse failure, **not** a test outcome; nothing was
built or run and that attempt's log was overwritten by the corrected run
(`b3sum` above covers the corrected run only). Corrected command (same intent,
standard cargo idiom):

```
GRAPHLAW_WASM=/Users/sac/graphlaw/priv/graphlaw.wasm \
CARGO_TARGET_DIR=/Users/sac/graphlaw/target-battery-finance \
cargo test --locked --test graphlaw_run_eh_court --test wasm_duckdb_lifecycle_eh \
  --test wasm_fibo_fixtures_eh --test wasm_fibo_egress_eh -- --test-threads=1 \
  > /tmp/bat-L5b.log 2>&1; echo "EXIT:$?" >> /tmp/bat-L5b.log
```

Verbatim result splice:

```
error: could not compile `purrdf-sparql-eval` (lib) due to 174 previous errors
EXIT:101
```

`grep -c "^test "` = **0**. `GRAPHLAW_DIAG_NO_EH` was never set.

| target | expected (dispatch) | observed | verdict |
|---|---|---|---|
| graphlaw_run_eh_court | 7 ok / 0 failed (typed exit-98 = REAL red, STOP) | 0 ran | BLOCKED |
| wasm_duckdb_lifecycle_eh | 3-4 ok / 0 failed | 0 ran | BLOCKED |
| wasm_fibo_fixtures_eh | 3 ok / 0 failed | 0 ran | BLOCKED |
| wasm_fibo_egress_eh | tc08 CLASSIFIED-RED | 0 ran | BLOCKED |

Validity leg ("≥1 artifact-instantiating ok"): **NOT MET** — zero tests ran on
both legs.

## Falsifiers

1. If the `vendor/purrdf-sparql-eval` break is repaired (by its owning lane or
   the coordinator) and Leg 5a is rerun on the healed tree, it must produce the
   dispatched expected counts (25/9/8/2/3/2-3 CLASSIFIED-RED/3/3/4/12-14) —
   this receipt's BLOCKED standing is then superseded by the rerun receipt.
2. If a rerun of Leg 5a on a claimed-green tree still yields 0 tests compiled,
   the break is not repaired and this receipt stands.
3. Leg 5b's exit-98 stop rule was never exercised (nothing ran); a rerun on a
   healed tree must show graphlaw_run_eh_court 7/0 or stop the battery per the
   dispatched rule.
