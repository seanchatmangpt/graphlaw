# P10 W3 pytest-oracle battery receipt (execution lane L6, 2026-10-24) — BLOCKED

- Date: 2026-10-24 (local)
- Subject: /Users/sac/graphlaw @ main `a00770b4998a55ed4ee397e3e2a1f5a880c51db0` (+ cross-repo `/Users/sac/ggen-marketplace/packages/pm4pytest`)
- Pins:
  - `courts/code-partition/axioms.blake3` b3 `2add10ce9ff782e33b75de099cba8dc8a114ab759855f93a7be85abb18581506`
  - `courts/code-partition/manifest.json` b3 `cb122a7b57fe8b4db62f81edca7a552ee2a0dbb43157b8c829c35f2309c6155d`
  - `priv/graphlaw.wasm` b3 `58a4409505f32b5d9aebc8c0ce2e2adeaba05c8c4b55f951a244ab4ad7d2280a` (== dispatched `58a44095…`)
- Logs (b3):
  - `/tmp/bat-L6-pm4pytest.log` `11ba4093f8810a0ff38d03724d009fa325875d2f5ecd0230a20172cc9cd888d7`
  - `/tmp/bat-L6B-build.log` `c3f0a88d4a110f8946ff2643e22bb8130829375492b23eb0cf23185b0c2e8499` (dispatched command)
  - `/tmp/bat-L6B-build2.log` `675e973fb571dfb2582a18fa5d1114c5553c6bb9613e061b07be4d37b67257aa` (corrected command)
  - `/tmp/bat-L6-bcbs.log` `8415ce837832dbac54678138a5b606e7142c6257cbe6937098a47a1ec312b8e9`
  - `/tmp/bat-L6-testspy.log` — **does not exist** (leg B pytest never ran; absence witnessed by b3sum os error 2)

Standing: **BLOCKED (shared compile break)** — zero oracle tests executed. Not
PASS, not CLASSIFIED.

## W0 preflight (all PASS)

| check | observed | verdict |
|---|---|---|
| `df -h /System/Volumes/Data` | 60 GiB avail (926Gi/844Gi, 94%) | PASS (≥20 GiB) |
| `b3sum priv/graphlaw.wasm` | `58a44095…d2280a` | PASS (matches pin) |
| `python3 -c "import pm4py"` | pm4py 2.7.22.1 | PASS |

## W3 A — pm4pytest (77 expected) — BLOCKED

Command:

```
cd /Users/sac/ggen-marketplace/packages/pm4pytest && \
GRAPHLAW_REPO=/Users/sac/graphlaw GRAPHLAW_TARGET_DIR=/Users/sac/graphlaw/target/debug \
python3 -m pytest graphlaw/tests -v > /tmp/bat-L6-pm4pytest.log 2>&1; echo "EXIT:$?" >> log
```

Verbatim splice:

```
INTERNALERROR>   File "/Users/sac/ggen-marketplace/packages/pm4pytest/graphlaw/conftest.py", line 54, in pytest_configure
INTERNALERROR>     raise SystemExit(f"conftest: build failed: {' '.join(args)}")
INTERNALERROR> SystemExit: conftest: build failed: cargo build --bins --features pack-tools
error: could not compile `purrdf-sparql-eval` (lib) due to 174 previous errors
EXIT:3
```

pytest exit 3 = INTERNALERROR during conftest `pytest_configure`: the conftest's
own `cargo build --bins --features pack-tools` against GRAPHLAW_REPO failed on
the shared break. **0 of 77 expected tests ran.** The pm4pytest suite itself
was never reached.

## W3 B — tests_py (14 expected) — BLOCKED twice over

Stage 1 (dispatched build command) verbatim:

```
error: the package 'graphlaw' does not contain this feature: disclose
EXIT:101
```

Typed dispatch-command defect: `disclose` is not a feature of the `graphlaw`
package. Verified against `Cargo.toml` `[features]` (lines 75-96): only
`evidentiary-ring`, `abi` (default), `pack-tools`, `wasi-patched-deps`,
`runner-eh`; `grep -rn disclose Cargo.toml crates/*/Cargo.toml` = no matches.
The conftest itself (tests_py/conftest.py:30) builds with
`--features runner-eh` only.

Stage 2 (corrected command, `--features runner-eh`, one rerun under a new
hypothesis: "disclose was stage 1's only blocker") verbatim:

```
error[E0433]: cannot find `vocab` in `purrdf_iri`
error[E0432]: unresolved import `purrdf_xsd::ieee`
error: could not compile `purrdf-sparql-eval` (lib) due to 174 previous errors
EXIT:101
```

So leg B is blocked independently by the same shared compile break; the
pytest stage was never started (`/tmp/bat-L6-testspy.log` absent).

## W3 C — BCBS aggregation (3 expected) — BLOCKED

Command:

```
GRAPHLAW_TARGET_DIR=/Users/sac/graphlaw/target python3 -m pytest tests/test_bcbs_aggregation.py -v \
  > /tmp/bat-L6-bcbs.log 2>&1; echo "EXIT:$?" >> /tmp/bat-L6-bcbs.log
```

Verbatim splice:

```
_______ ERROR at setup of test_tc_bcbs_pm4py_01_compliant_fold_conforms ________
graphlaw-wasm build failed (exit 101):
error[E0220]: associated type `ReadError` not found for `DatasetView`
============================== 3 errors in 58.08s ==============================
EXIT:1
```

All 3 tests ERROR at setup: the fixture builds `graphlaw-wasm` (real
wasm32-wasip1 cdylib) and captures exit 101 on the shared purrdf break.
**0 of 3 expected tests ran.**

## Shared root cause + owner evidence

`vendor/purrdf-sparql-eval` (lib) fails with 174 unresolved-import errors
against its crates.io sibling crates (`purrdf_iri::vocab`, `purrdf_xsd::ieee`,
`purrdf_core::TermBox`, `purrdf_hash`, `purrdf_lex`, `purrdf_stack`,
`purrdf_sparql_algebra::{Child,Chain,NodeRef,…}`). Same error signature in all
five cargo-backed logs. `vendor/purrdf-sparql-eval/src/lib.rs` last write
`1792873263`, unchanged through `1792874430` (≈19.5 min, zero further writes;
5-minute bounded mtime poll saw none) — the ~10-minute compile-freeze SLA
expired with the tree left broken. Owner: whichever lane is mid-flight on
`vendor/purrdf-sparql-eval` (58 files, +55878/−11677 vs HEAD).

## PASS/CLASSIFIED/SKIP ledger

| leg | expected | ran | ok | verdict |
|---|---|---|---|---|
| W3 A pm4pytest | 77 | 0 | 0 | BLOCKED |
| W3 B build stage 1 | build ok | — | — | FAIL (typed: nonexistent feature `disclose`) |
| W3 B build stage 2 (corrected) | build ok | — | — | BLOCKED (shared break) |
| W3 B tests_py | 14 | 0 | 0 | BLOCKED |
| W3 C bcbs | 3 | 0 | 0 | BLOCKED |

No SKIP rows: nothing was skipped — every leg either ran to a typed failure or
never reached its tests and says so loudly.

## Falsifiers

1. After the `vendor/purrdf-sparql-eval` break is repaired and the battery is
   rerun on the healed tree: pm4pytest must reach 77 (or fail per-test, not at
   conftest), tests_py must reach 14, bcbs must reach 3 — superseding this
   receipt.
2. A rerun where the conftest still fails at `pytest_configure`/setup means the
   break is not repaired; this receipt stands.
3. If a healed-tree rerun of W3 B stage 2 (no `disclose`) succeeds but
   tests_py fails on a missing `disclose`-gated behavior, the dispatched
   command's feature set — not this lane's correction — was wrong and the
   dispatch defect escalates to the coordinator.
