# Receipt — wasmi retirement step-2 green witness (pytest over graphlaw-run-eh)

- **Lane**: finB implementation lane (lease: tests_py, src/bin/dl_dualroute.rs, examples/*.rs x6, .claude/hooks/graphlaw_admission_guard.py, new receipts)
- **Date**: 2026-10-24 (run window ended 2026-10-24T17:21Z)
- **HEAD**: `6da1d33a93f61068fa9b033bec353db78d335460`
- **Standing**: PARTIAL_ALIVE — suite green on the dirty tree at 6da1d33; manifest commit is coordinator-owned.

## Exact commands

```bash
# Flip verification (zero non-eh hits):
grep -rn 'graphlaw-run' tests_py/ | grep -vc 'graphlaw-run-eh'   # -> 0
grep -rn -- '--features' tests_py/ | grep -v 'runner-eh'         # -> 0 lines (exit 1)

# Build (exit 0, warnings only; Finished dev profile in 2m 04s):
CARGO_TARGET_DIR=/Users/sac/graphlaw/target-lane-finB \
  cargo build --bins --example blackbox_fixture --features runner-eh

# Suite:
GRAPHLAW_TARGET_DIR=/Users/sac/graphlaw/target-lane-finB/debug \
GRAPHLAW_WASM=/Users/sac/graphlaw/priv/graphlaw.wasm pytest tests_py -v
```

## Flip verification (verbatim)

- `grep -rn 'graphlaw-run' tests_py/ | grep -vc 'graphlaw-run-eh'` printed `0`.
- `grep 'features runner\b' tests_py/` → no matches, exit 1. The only
  `--features` occurrences in tests_py are `--features runner-eh`
  (conftest.py:30, conftest.py:223).
- Full `graphlaw-run` reference list witnessed: conftest.py:36,40,80,90,223;
  test_blackbox_ops.py:74,156,172,176,255,290,295,358 — every occurrence is
  `graphlaw-run-eh` or `runner-eh`.

## Build tail (verbatim)

```
warning: `graphlaw` (bin "graphlaw_ocel_export") generated 7 warnings (run `cargo fix --bin "graphlaw_ocel_export" -p graphlaw` to apply 7 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 04s
BUILD_EXIT=0
```

## Pytest output (verbatim, full session block)

```
============================= test session starts ==============================
platform darwin -- Python 3.13.11, pytest-9.0.3, pluggy-1.6.0 -- /opt/homebrew/opt/python@3.13/bin/python3.13
cachedir: .pytest_cache
benchmark: 5.2.3 (defaults: timer=time.perf_counter disable_gc=False min_rounds=5 min_time=0.000005 max_time=1.0 calibration_precision=10 warmup=False warmup_iterations=100000)
rootdir: /Users/sac/graphlaw
plugins: benchmark-5.2.3, mock-3.15.1, Faker-40.15.0, cov-7.1.0, xdist-3.8.0, timeout-2.4.0, asyncio-1.3.0, typeguard-4.4.3, anyio-4.13.0
asyncio: mode=Mode.STRICT, debug=False, asyncio_default_fixture_loop_scope=None, asyncio_default_test_loop_scope=function
collecting ... collected 19 items

tests_py/test_blackbox_ops.py::test_verify_happy_path_admitted PASSED    [  5%]
tests_py/test_blackbox_ops.py::test_verify_receipt_tamper_is_typed_refusal_exit_1 PASSED [ 10%]
tests_py/test_blackbox_ops.py::test_verify_untrusted_signer_is_typed_refusal PASSED [ 15%]
tests_py/test_blackbox_ops.py::test_verify_missing_trusted_key_is_usage_exit_2 PASSED [ 21%]
tests_py/test_blackbox_ops.py::test_run_packet4_eval_turtle_law_doc PASSED [ 26%]
tests_py/test_blackbox_ops.py::test_run_packet4_two_frames_one_process PASSED [ 31%]
tests_py/test_blackbox_ops.py::test_run_digest_mismatch_is_loud_refusal_exit_84 PASSED [ 36%]
tests_py/test_blackbox_ops.py::test_run_eof_midframe_is_typed_exit_90 PASSED [ 42%]
tests_py/test_blackbox_ops.py::test_run_malformed_envelope_is_typed_exit_93 PASSED [ 47%]
tests_py/test_blackbox_ops.py::test_refusal_probe_smoke_exit_zero PASSED [ 52%]
tests_py/test_blackbox_ops.py::test_refusal_class_fires_pinned_register_message[ATTEST-REFUSED-class admission-seal ReceiptError] PASSED [ 57%]
tests_py/test_blackbox_ops.py::test_refusal_class_fires_pinned_register_message[NO-CANONICAL-GRAPH hand-built CompiledQuery] PASSED [ 63%]
tests_py/test_blackbox_ops.py::test_refusal_class_fires_pinned_register_message[REFUSAL-04 non-document-backed reader] PASSED [ 68%]
tests_py/test_blackbox_ops.py::test_refusal_class_fires_pinned_register_message[ReceiptError canonicalization refused] PASSED [ 73%]
tests_py/test_blackbox_ops.py::test_refusal_class_fires_pinned_register_message[UNCLASSIFIED unknown function IRI] PASSED [ 78%]
tests_py/test_blackbox_ops.py::test_refusal_class_fires_pinned_register_message[direct validate_ast_shacl] PASSED [ 84%]
tests_py/test_blackbox_ops.py::test_wire_canonicalization_refused_on_malformed_turtle PASSED [ 89%]
tests_py/test_ubo_sanctions.py::test_ubo_pm4pytest_sanctions_unrolling PASSED [ 94%]
tests_py/test_ubo_sanctions.py::test_ubo_pm4pytest_session_isolation PASSED [100%]

======================== 19 passed in 72.40s (0:01:12) =========================
PYTEST_RECAP_EXIT=0
```

Two runs on the same tree and binaries: the first witnessed run (17:2xZ,
cold caches) returned **19 passed in 438.80s (0:07:18), exit 0**; the block
embedded above is the 17:3xZ re-capture (19 passed in 72.40s, exit 0),
spliced byte-exactly from `/tmp/r1-pytest.log` so the receipt carries raw
bytes rather than retyped output. Both runs: 19/19 PASSED (17 blackbox +
2 ubo_sanctions).

## Artifact identity

```
$ /opt/homebrew/bin/b3sum priv/graphlaw.wasm
58a4409505f32b5d9aebc8c0ce2e2adeaba05c8c4b55f951a244ab4ad7d2280a  priv/graphlaw.wasm
```

58,124,612 bytes. Matches the shipped-artifact pin on all 4 surfaces
(live b3sum; `priv/graphlaw.wasm.blake3`; `registry/ARTIFACTS.blake3:13`
`wasm 58a44095…d2280a 58124612 graphlaw_wasm.wasm`;
`js/npm/src/generated/artifact-meta.ts:3`).

## ADR order-inversion disclosure (dated 2026-10-24)

6da1d33 (HEAD) deleted `src/bin/graphlaw_run.rs` (651 lines) one commit
ahead of this step-2 witness: the ADR's canonical order is
flip-then-witness, but the runner source deletion landed before this witness
existed. Disclosed as executed: the witness ran on a tree where the old
runner source is already absent from HEAD, and no further deletion lands on
an unrun flip — the tests_py flip itself was already present in the working
tree at dispatch time and is witnessed here as RUN, not merely inspected.
6da1d33 also touched `src/bin/graphlaw_run_eh.rs` (+11/−) in the same
commit. Pre-delete blob: `git show 6da1d33~1:src/bin/graphlaw_run.rs` is
25,746 bytes; digest recorded in
`receipts/2026-10-24-wasmi-retirement-deletion.md`.

## Dirty-file disclosure

Full `git status --porcelain` capture at 2026-10-24T17:3xZ, HEAD 6da1d33,
spliced verbatim below: 38 modified paths (in-flight wasmi retirement,
load-bearing, do not revert — the count grew from 30 M at the first pytest
run window earlier this session; the tree is live with concurrent lanes) and
419 untracked paths (RECEIPTS.md, receipts/ artifacts, transition JSONs):

```
 M .claude/hooks/graphlaw_admission_guard.py
 M .github/workflows/ci.yml
 M .github/workflows/js.yml
 M .github/workflows/mutation.yml
 M .github/workflows/release.yml
 M CHANGELOG.md
 M CLAUDE.md
 M Cargo.lock
 M Cargo.toml
 M LICENSE.md
 M README.md
 M SECURITY.md
 M crates/graphlaw-eyeron/Cargo.toml
 M crates/graphlaw-eyeron/UPSTREAM.md
 M deny.toml
 M docs/PHD_THESIS_GRAPHLAW_WASM_SQLRDF.md
 M docs/WASM.md
 M docs/how-to/first-in-wasm-sql-lifecycle.md
 M docs/how-to/verify-without-trusting-the-operator.md
 M docs/jira/v26.10.22/RECEIPTS.md
 M docs/jira/v26.10.22/RELEASE-NOTES-26.10.22.md
 M docs/reference/refusal-catalog.md
 M docs/reference/runner-exit-codes.md
 M docs/reference/sql-rdf-rec.md
 M docs/reference/wasm-gap-register.md
 M docs/refusals.md
 M examples/asp_tableaux_vector_gen.rs
 M examples/bayes_pop_vectors.rs
 M examples/breed_oracle_gen.rs
 M examples/event_calculus_vectors.rs
 M examples/naive_physics_vectors.rs
 M examples/plan_replay_oracle_gen.rs
 M js/npm/CHANGELOG.md
 M receipts/CHAIN.blake3
 M src/bin/dl_dualroute.rs
 M tests/sql_rdf_conformance.rs
 M tests_py/conftest.py
 M tests_py/test_blackbox_ops.py
?? .github/actions/
?? LICENSE-P28-SIL.md
?? RECEIPTS-INDEX.md
?? audit.toml
?? receipts/2026-10-24-r1-rungs.md
?? receipts/2026-10-24-r2-deny-decision.md
?? receipts/2026-10-24-spc-full.md
?? receipts/2026-10-24-spc-solo-rerun.md
?? receipts/2026-10-24-wasmi-retirement-deletion.md
?? receipts/2026-10-24-wasmi-retirement-step2-green-witness.md
?? receipts/spc-2026-10-24T032511.json
?? receipts/spc-2026-10-24T102226.json
?? receipts/transition-2026-10-24T064943Z-Edit-73e954f4.json
?? receipts/transition-2026-10-24T065056Z-Edit-5ed3d77b.json
?? receipts/transition-2026-10-24T065257Z-Edit-e3d5764d.json
?? receipts/transition-2026-10-24T065352Z-Edit-f0ec8294.json
?? receipts/transition-2026-10-24T065507Z-Edit-02418967.json
?? receipts/transition-2026-10-24T065507Z-Write-59f58baa.json
?? receipts/transition-2026-10-24T065508Z-Write-071f7ec8.json
?? receipts/transition-2026-10-24T065508Z-Write-e3ace3e6.json
?? receipts/transition-2026-10-24T065533Z-Write-11be2605.json
?? receipts/transition-2026-10-24T065542Z-Write-9fee8b29.json
?? receipts/transition-2026-10-24T065552Z-Write-6ee692f4.json
?? receipts/transition-2026-10-24T065706Z-Edit-4778a473.json
?? receipts/transition-2026-10-24T065711Z-Edit-31efb521.json
?? receipts/transition-2026-10-24T065712Z-Edit-d8b80e86.json
?? receipts/transition-2026-10-24T065714Z-Edit-ba2c7c6c.json
?? receipts/transition-2026-10-24T065721Z-Edit-90026c64.json
?? receipts/transition-2026-10-24T065723Z-Edit-74090340.json
?? receipts/transition-2026-10-24T065731Z-Edit-01bdbd59.json
?? receipts/transition-2026-10-24T065739Z-Edit-1bb45701.json
?? receipts/transition-2026-10-24T065747Z-Edit-79b9039b.json
?? receipts/transition-2026-10-24T065755Z-Edit-d04ea382.json
?? receipts/transition-2026-10-24T065800Z-Write-16154e6d.json
?? receipts/transition-2026-10-24T065800Z-Write-d323fbf1.json
?? receipts/transition-2026-10-24T065803Z-Edit-5381fa20.json
?? receipts/transition-2026-10-24T065810Z-Write-90e34fbb.json
?? receipts/transition-2026-10-24T065811Z-Edit-00cc7b1c.json
?? receipts/transition-2026-10-24T065826Z-Edit-fb2979fd.json
?? receipts/transition-2026-10-24T065830Z-Edit-77ef93ab.json
?? receipts/transition-2026-10-24T065837Z-Edit-5d5c8a54.json
?? receipts/transition-2026-10-24T065842Z-Write-37454294.json
?? receipts/transition-2026-10-24T065857Z-Edit-af7ae15e.json
?? receipts/transition-2026-10-24T065904Z-Edit-a821dae0.json
?? receipts/transition-2026-10-24T071811Z-Edit-a72c2759.json
?? receipts/transition-2026-10-24T071832Z-Edit-27d2286b.json
?? receipts/transition-2026-10-24T071931Z-Edit-3678f7ba.json
?? receipts/transition-2026-10-24T072429Z-Write-1e97da4c.json
?? receipts/transition-2026-10-24T072433Z-Write-e299fe55.json
?? receipts/transition-2026-10-24T072535Z-Edit-203fcff3.json
?? receipts/transition-2026-10-24T072603Z-Edit-3034cb57.json
?? receipts/transition-2026-10-24T072629Z-Edit-a3f4ab0e.json
?? receipts/transition-2026-10-24T072653Z-Edit-241d3955.json
?? receipts/transition-2026-10-24T072704Z-Edit-15b464be.json
?? receipts/transition-2026-10-24T072707Z-Edit-2b2c2ef0.json
?? receipts/transition-2026-10-24T072723Z-Edit-de844dfb.json
?? receipts/transition-2026-10-24T072735Z-Edit-b676d5dd.json
?? receipts/transition-2026-10-24T072742Z-Edit-5dcc08ba.json
?? receipts/transition-2026-10-24T072745Z-Edit-97599e9c.json
?? receipts/transition-2026-10-24T072801Z-Edit-ccb81eec.json
?? receipts/transition-2026-10-24T072803Z-Edit-18d4a79f.json
?? receipts/transition-2026-10-24T072822Z-Edit-5bd3aef9.json
?? receipts/transition-2026-10-24T072825Z-Edit-72ece8d8.json
?? receipts/transition-2026-10-24T072826Z-Edit-db45857c.json
?? receipts/transition-2026-10-24T072835Z-Edit-158652f7.json
?? receipts/transition-2026-10-24T072851Z-Edit-8f53ddd1.json
?? receipts/transition-2026-10-24T072854Z-Edit-292b5bb5.json
?? receipts/transition-2026-10-24T072859Z-Edit-f5704fd6.json
?? receipts/transition-2026-10-24T072909Z-Edit-d88c8135.json
?? receipts/transition-2026-10-24T072911Z-Edit-0c63f595.json
?? receipts/transition-2026-10-24T072914Z-Edit-a0234562.json
?? receipts/transition-2026-10-24T072923Z-Edit-03bb6041.json
?? receipts/transition-2026-10-24T072925Z-Edit-89590ebf.json
?? receipts/transition-2026-10-24T072930Z-Edit-3fe869ba.json
?? receipts/transition-2026-10-24T072931Z-Edit-f63f24b4.json
?? receipts/transition-2026-10-24T072932Z-Edit-4f9fd1b4.json
?? receipts/transition-2026-10-24T072945Z-Write-10d29a61.json
?? receipts/transition-2026-10-24T072946Z-Edit-8fe97ddc.json
?? receipts/transition-2026-10-24T072953Z-Edit-b421c34b.json
?? receipts/transition-2026-10-24T073009Z-Edit-a4429305.json
?? receipts/transition-2026-10-24T073012Z-Edit-22b705aa.json
?? receipts/transition-2026-10-24T073023Z-Edit-245b41a7.json
?? receipts/transition-2026-10-24T073023Z-Edit-8562c3e4.json
?? receipts/transition-2026-10-24T073031Z-Edit-9fd5b1ea.json
?? receipts/transition-2026-10-24T073031Z-Write-b665932d.json
?? receipts/transition-2026-10-24T073054Z-Edit-3dc92aea.json
?? receipts/transition-2026-10-24T073101Z-Edit-533f66e4.json
?? receipts/transition-2026-10-24T073207Z-Write-32bcbf40.json
?? receipts/transition-2026-10-24T073300Z-Write-42a17255.json
?? receipts/transition-2026-10-24T073336Z-Write-07ed07e5.json
?? receipts/transition-2026-10-24T073352Z-Edit-939c442a.json
?? receipts/transition-2026-10-24T073439Z-Write-7f50b269.json
?? receipts/transition-2026-10-24T073537Z-Write-531411e7.json
?? receipts/transition-2026-10-24T073750Z-Write-54c399d6.json
?? receipts/transition-2026-10-24T073811Z-Write-e353ce91.json
?? receipts/transition-2026-10-24T074658Z-Edit-1c35ada8.json
?? receipts/transition-2026-10-24T074658Z-Write-6f42c992.json
?? receipts/transition-2026-10-24T074723Z-Edit-c99758de.json
?? receipts/transition-2026-10-24T074922Z-Edit-0fb0bc99.json
?? receipts/transition-2026-10-24T075334Z-Write-54be8021.json
?? receipts/transition-2026-10-24T075338Z-Edit-58a1f2c3.json
?? receipts/transition-2026-10-24T075338Z-Edit-aac7f856.json
?? receipts/transition-2026-10-24T075346Z-Edit-f660877c.json
?? receipts/transition-2026-10-24T075354Z-Edit-3a2a3b1c.json
?? receipts/transition-2026-10-24T075410Z-Edit-89e98e1b.json
?? receipts/transition-2026-10-24T075419Z-Edit-a150caed.json
?? receipts/transition-2026-10-24T075429Z-Edit-1dd760da.json
?? receipts/transition-2026-10-24T075434Z-Edit-24fc5bd1.json
?? receipts/transition-2026-10-24T075442Z-Edit-0c3f6088.json
?? receipts/transition-2026-10-24T075445Z-Edit-9931a06e.json
?? receipts/transition-2026-10-24T075450Z-Edit-0b37e66a.json
?? receipts/transition-2026-10-24T075502Z-Edit-641f00d8.json
?? receipts/transition-2026-10-24T075510Z-Edit-ebaa45fd.json
?? receipts/transition-2026-10-24T075519Z-Edit-577d3c24.json
?? receipts/transition-2026-10-24T075906Z-Write-a568a275.json
?? receipts/transition-2026-10-24T075942Z-Edit-06a96e74.json
?? receipts/transition-2026-10-24T075955Z-Edit-0c18cda4.json
?? receipts/transition-2026-10-24T080006Z-Edit-ee19eb1f.json
?? receipts/transition-2026-10-24T080018Z-Edit-9556d794.json
?? receipts/transition-2026-10-24T080038Z-Edit-41b5f7a1.json
?? receipts/transition-2026-10-24T080038Z-Write-ae1b7c40.json
?? receipts/transition-2026-10-24T080047Z-Edit-5bd1c325.json
?? receipts/transition-2026-10-24T080049Z-Edit-73ea2ef3.json
?? receipts/transition-2026-10-24T080208Z-Edit-6fda8456.json
?? receipts/transition-2026-10-24T080208Z-Edit-ff55f31b.json
?? receipts/transition-2026-10-24T080224Z-Edit-42ee2aaf.json
?? receipts/transition-2026-10-24T080624Z-Write-a8b1cd27.json
?? receipts/transition-2026-10-24T080633Z-Edit-f22d3856.json
?? receipts/transition-2026-10-24T080700Z-Edit-71e90446.json
?? receipts/transition-2026-10-24T080749Z-Edit-9e6ea0d3.json
?? receipts/transition-2026-10-24T080749Z-Edit-a03c72b5.json
?? receipts/transition-2026-10-24T080749Z-Edit-bc49736c.json
?? receipts/transition-2026-10-24T080758Z-Edit-4475fd2d.json
?? receipts/transition-2026-10-24T080758Z-Edit-7247da4c.json
?? receipts/transition-2026-10-24T080806Z-Edit-f9ca0ba6.json
?? receipts/transition-2026-10-24T080815Z-Edit-5431ee0e.json
?? receipts/transition-2026-10-24T080858Z-Edit-d2bae342.json
?? receipts/transition-2026-10-24T080912Z-Edit-dbae8763.json
?? receipts/transition-2026-10-24T080941Z-Edit-5b57c9b2.json
?? receipts/transition-2026-10-24T080951Z-Edit-56edfc87.json
?? receipts/transition-2026-10-24T081129Z-Edit-08593452.json
?? receipts/transition-2026-10-24T081129Z-Edit-5f117d87.json
?? receipts/transition-2026-10-24T081142Z-Edit-effa4344.json
?? receipts/transition-2026-10-24T081152Z-Edit-31dbdebb.json
?? receipts/transition-2026-10-24T081203Z-Edit-d0a60da9.json
?? receipts/transition-2026-10-24T081216Z-Edit-d0a5b2af.json
?? receipts/transition-2026-10-24T081242Z-Edit-99fb3320.json
?? receipts/transition-2026-10-24T081253Z-Edit-56827d4d.json
?? receipts/transition-2026-10-24T081434Z-Edit-7fa120a9.json
?? receipts/transition-2026-10-24T081550Z-Edit-b725a7ed.json
?? receipts/transition-2026-10-24T081557Z-Edit-8eb5ba0e.json
?? receipts/transition-2026-10-24T081634Z-Edit-aca98bb1.json
?? receipts/transition-2026-10-24T081838Z-Edit-cf7bc533.json
?? receipts/transition-2026-10-24T082741Z-Edit-01364e83.json
?? receipts/transition-2026-10-24T082808Z-Edit-f30edffc.json
?? receipts/transition-2026-10-24T082816Z-Edit-8231583b.json
?? receipts/transition-2026-10-24T082846Z-Edit-4b40375c.json
?? receipts/transition-2026-10-24T082914Z-Edit-f139cb7a.json
?? receipts/transition-2026-10-24T084326Z-Edit-fd8f261f.json
?? receipts/transition-2026-10-24T084452Z-Edit-c56ffd9d.json
?? receipts/transition-2026-10-24T090139Z-Edit-2e6c2b1b.json
?? receipts/transition-2026-10-24T102021Z-Write-86fae728.json
?? receipts/transition-2026-10-24T102040Z-Edit-1a82fc74.json
?? receipts/transition-2026-10-24T102132Z-Edit-fc355693.json
?? receipts/transition-2026-10-24T102138Z-Write-ff3c94c8.json
?? receipts/transition-2026-10-24T102148Z-Edit-18d9a9f6.json
?? receipts/transition-2026-10-24T102207Z-Edit-7a088d58.json
?? receipts/transition-2026-10-24T102536Z-Edit-cfa8705b.json
?? receipts/transition-2026-10-24T102544Z-Edit-7189c17f.json
?? receipts/transition-2026-10-24T102601Z-Edit-3bb4cedb.json
?? receipts/transition-2026-10-24T102609Z-Edit-48226818.json
?? receipts/transition-2026-10-24T102634Z-Write-339e017e.json
?? receipts/transition-2026-10-24T102650Z-Edit-944e3e21.json
?? receipts/transition-2026-10-24T102653Z-Write-65293e48.json
?? receipts/transition-2026-10-24T102659Z-Edit-b35468f2.json
?? receipts/transition-2026-10-24T102709Z-Edit-aa8c8933.json
?? receipts/transition-2026-10-24T102717Z-Edit-b8481cc1.json
?? receipts/transition-2026-10-24T102718Z-Edit-1f67ebe5.json
?? receipts/transition-2026-10-24T102746Z-Edit-24fe5be9.json
?? receipts/transition-2026-10-24T102756Z-Edit-52c80ef0.json
?? receipts/transition-2026-10-24T102758Z-Edit-69774668.json
?? receipts/transition-2026-10-24T102806Z-Edit-893e4066.json
?? receipts/transition-2026-10-24T102814Z-Edit-2957771a.json
?? receipts/transition-2026-10-24T102834Z-Write-71512d89.json
?? receipts/transition-2026-10-24T102841Z-Edit-273ef6a4.json
?? receipts/transition-2026-10-24T102852Z-Edit-0f520a63.json
?? receipts/transition-2026-10-24T102853Z-Edit-586f56e3.json
?? receipts/transition-2026-10-24T102856Z-Edit-5db5e75a.json
?? receipts/transition-2026-10-24T102900Z-Edit-f122b291.json
?? receipts/transition-2026-10-24T102902Z-Edit-23f899ac.json
?? receipts/transition-2026-10-24T102908Z-Edit-61878052.json
?? receipts/transition-2026-10-24T102923Z-Edit-c5f649a4.json
?? receipts/transition-2026-10-24T102927Z-Write-c3b0d7dc.json
?? receipts/transition-2026-10-24T102929Z-Edit-7e0d1531.json
?? receipts/transition-2026-10-24T102931Z-Edit-3fc43203.json
?? receipts/transition-2026-10-24T102940Z-Edit-facda25f.json
?? receipts/transition-2026-10-24T102948Z-Edit-8945a6ef.json
?? receipts/transition-2026-10-24T102959Z-Edit-f4681904.json
?? receipts/transition-2026-10-24T103007Z-Edit-6ee565c5.json
?? receipts/transition-2026-10-24T103013Z-Edit-ff76944e.json
?? receipts/transition-2026-10-24T103014Z-Edit-bb35d0c0.json
?? receipts/transition-2026-10-24T103017Z-Write-134b6579.json
?? receipts/transition-2026-10-24T103028Z-Edit-c9ca80d0.json
?? receipts/transition-2026-10-24T103031Z-Edit-a0dda79d.json
?? receipts/transition-2026-10-24T103040Z-Edit-eace9da8.json
?? receipts/transition-2026-10-24T103050Z-Edit-0dab4c3d.json
?? receipts/transition-2026-10-24T103100Z-Edit-06997a16.json
?? receipts/transition-2026-10-24T103113Z-Edit-294f7ffe.json
?? receipts/transition-2026-10-24T103123Z-Edit-44360950.json
?? receipts/transition-2026-10-24T103127Z-Edit-7ffadd8b.json
?? receipts/transition-2026-10-24T103132Z-Edit-2061e169.json
?? receipts/transition-2026-10-24T103137Z-Edit-1d37c253.json
?? receipts/transition-2026-10-24T103154Z-Edit-7ffadd8b.json
?? receipts/transition-2026-10-24T103204Z-Edit-8aaace58.json
?? receipts/transition-2026-10-24T103216Z-Edit-094e136f.json
?? receipts/transition-2026-10-24T103233Z-Edit-352b555c.json
?? receipts/transition-2026-10-24T103237Z-Edit-49ca2464.json
?? receipts/transition-2026-10-24T103245Z-Edit-44b6e337.json
?? receipts/transition-2026-10-24T103310Z-Write-e2a69319.json
?? receipts/transition-2026-10-24T103326Z-Edit-5f02487d.json
?? receipts/transition-2026-10-24T103328Z-Edit-3e030d23.json
?? receipts/transition-2026-10-24T103331Z-Edit-7bb0b17f.json
?? receipts/transition-2026-10-24T103341Z-Edit-44cca60c.json
?? receipts/transition-2026-10-24T103358Z-Edit-ac3bc08e.json
?? receipts/transition-2026-10-24T103407Z-Edit-12431ff0.json
?? receipts/transition-2026-10-24T103410Z-Edit-7c2fecba.json
?? receipts/transition-2026-10-24T103426Z-Edit-0b2af556.json
?? receipts/transition-2026-10-24T103429Z-Edit-6b243250.json
?? receipts/transition-2026-10-24T103435Z-Edit-740931a5.json
?? receipts/transition-2026-10-24T103440Z-Edit-296a346d.json
?? receipts/transition-2026-10-24T103451Z-Edit-04c5c61f.json
?? receipts/transition-2026-10-24T103500Z-Edit-8ac597ee.json
?? receipts/transition-2026-10-24T103509Z-Edit-b96f0d35.json
?? receipts/transition-2026-10-24T103542Z-Edit-31682882.json
?? receipts/transition-2026-10-24T103604Z-Edit-505fc3fd.json
?? receipts/transition-2026-10-24T103611Z-Edit-35127a0d.json
?? receipts/transition-2026-10-24T103618Z-Edit-95a700ac.json
?? receipts/transition-2026-10-24T103626Z-Edit-00c32cf4.json
?? receipts/transition-2026-10-24T103626Z-Edit-de7713b1.json
?? receipts/transition-2026-10-24T103635Z-Edit-77286217.json
?? receipts/transition-2026-10-24T103645Z-Edit-9da27fb9.json
?? receipts/transition-2026-10-24T103650Z-Edit-aba0a5e9.json
?? receipts/transition-2026-10-24T103658Z-Edit-6e1bf8e1.json
?? receipts/transition-2026-10-24T103708Z-Edit-e4df6170.json
?? receipts/transition-2026-10-24T103716Z-Edit-d4352b42.json
?? receipts/transition-2026-10-24T103744Z-Edit-c8518aa1.json
?? receipts/transition-2026-10-24T103817Z-Edit-98a9ef79.json
?? receipts/transition-2026-10-24T103831Z-Write-1f214b0a.json
?? receipts/transition-2026-10-24T103838Z-Edit-b1201217.json
?? receipts/transition-2026-10-24T103847Z-Edit-20f31d97.json
?? receipts/transition-2026-10-24T103847Z-Edit-57cc71cc.json
?? receipts/transition-2026-10-24T103853Z-Edit-afb71f1c.json
?? receipts/transition-2026-10-24T103902Z-Edit-3190949d.json
?? receipts/transition-2026-10-24T103903Z-Edit-5c55cb35.json
?? receipts/transition-2026-10-24T103907Z-Edit-e5516263.json
?? receipts/transition-2026-10-24T103911Z-Edit-41090f9e.json
?? receipts/transition-2026-10-24T103918Z-Edit-268da0e3.json
?? receipts/transition-2026-10-24T103928Z-Edit-b518a549.json
?? receipts/transition-2026-10-24T103937Z-Edit-64da7a93.json
?? receipts/transition-2026-10-24T103946Z-Edit-1d51150c.json
?? receipts/transition-2026-10-24T104007Z-Edit-8ee68a90.json
?? receipts/transition-2026-10-24T104032Z-Edit-828315a3.json
?? receipts/transition-2026-10-24T104132Z-Edit-62ba53ec.json
?? receipts/transition-2026-10-24T104142Z-Edit-9506eafa.json
?? receipts/transition-2026-10-24T104144Z-Edit-c7b99e6a.json
?? receipts/transition-2026-10-24T104152Z-Edit-23373c9e.json
?? receipts/transition-2026-10-24T104152Z-Edit-480c088c.json
?? receipts/transition-2026-10-24T104210Z-Edit-57b7f7b3.json
?? receipts/transition-2026-10-24T104254Z-Edit-62952e08.json
?? receipts/transition-2026-10-24T104323Z-Edit-22d923cc.json
?? receipts/transition-2026-10-24T104337Z-Write-892efbcf.json
?? receipts/transition-2026-10-24T104352Z-Edit-af768c76.json
?? receipts/transition-2026-10-24T104414Z-Write-452f00c8.json
?? receipts/transition-2026-10-24T104438Z-Edit-d7481b59.json
?? receipts/transition-2026-10-24T104708Z-Write-d279333b.json
?? receipts/transition-2026-10-24T154027Z-Write-6503ec4f.json
?? receipts/transition-2026-10-24T154148Z-Edit-8f23b763.json
?? receipts/transition-2026-10-24T154210Z-Edit-79702016.json
?? receipts/transition-2026-10-24T154234Z-Edit-8098062c.json
?? receipts/transition-2026-10-24T154302Z-Edit-007f65c3.json
?? receipts/transition-2026-10-24T154416Z-Edit-61afdac7.json
?? receipts/transition-2026-10-24T154440Z-Edit-2b64423f.json
?? receipts/transition-2026-10-24T154502Z-Edit-8908b397.json
?? receipts/transition-2026-10-24T154537Z-Edit-d17f29f8.json
?? receipts/transition-2026-10-24T154600Z-Edit-2a5939ba.json
?? receipts/transition-2026-10-24T154621Z-Edit-85ecdf4d.json
?? receipts/transition-2026-10-24T154658Z-Edit-0420de8d.json
?? receipts/transition-2026-10-24T154909Z-Edit-bb56f9dd.json
?? receipts/transition-2026-10-24T154939Z-Edit-a821f2fc.json
?? receipts/transition-2026-10-24T155159Z-Edit-407ae8fb.json
?? receipts/transition-2026-10-24T155238Z-Edit-04c9f388.json
?? receipts/transition-2026-10-24T155749Z-Edit-a8239df3.json
?? receipts/transition-2026-10-24T160629Z-Edit-bb489bf8.json
?? receipts/transition-2026-10-24T161805Z-Edit-cbe92600.json
?? receipts/transition-2026-10-24T161935Z-Edit-313f2911.json
?? receipts/transition-2026-10-24T162523Z-Edit-010e6f10.json
?? receipts/transition-2026-10-24T162551Z-Edit-95ad493a.json
?? receipts/transition-2026-10-24T162621Z-Edit-ceb07861.json
?? receipts/transition-2026-10-24T162844Z-Edit-a1a6f587.json
?? receipts/transition-2026-10-24T162918Z-Edit-310ea5e9.json
?? receipts/transition-2026-10-24T162944Z-Edit-f9fbab29.json
?? receipts/transition-2026-10-24T163022Z-Edit-35b76075.json
?? receipts/transition-2026-10-24T163314Z-Edit-36097e27.json
?? receipts/transition-2026-10-24T163445Z-Edit-69e7f865.json
?? receipts/transition-2026-10-24T163706Z-Edit-1322c4c5.json
?? receipts/transition-2026-10-24T163926Z-Edit-17afc98a.json
?? receipts/transition-2026-10-24T164043Z-Edit-932cf46c.json
?? receipts/transition-2026-10-24T165615Z-Edit-41eab758.json
?? receipts/transition-2026-10-24T170603Z-Edit-1bc6c06c.json
?? receipts/transition-2026-10-24T170750Z-Write-9d6accd5.json
?? receipts/transition-2026-10-24T170811Z-Write-a30318c1.json
?? receipts/transition-2026-10-24T171009Z-Edit-7d76b5fe.json
?? receipts/transition-2026-10-24T171018Z-Edit-1b695fe8.json
?? receipts/transition-2026-10-24T171028Z-Edit-c2197788.json
?? receipts/transition-2026-10-24T171049Z-Edit-c8ffccc8.json
?? receipts/transition-2026-10-24T171213Z-Edit-8b8d3ea1.json
?? receipts/transition-2026-10-24T171221Z-Edit-a58818bf.json
?? receipts/transition-2026-10-24T171244Z-Edit-d4cffeaf.json
?? receipts/transition-2026-10-24T171259Z-Write-56630060.json
?? receipts/transition-2026-10-24T171352Z-Edit-866a555d.json
?? receipts/transition-2026-10-24T171411Z-Edit-11011c6c.json
?? receipts/transition-2026-10-24T171413Z-Write-1c84c52a.json
?? receipts/transition-2026-10-24T171437Z-Edit-1ed23667.json
?? receipts/transition-2026-10-24T171446Z-Edit-74a3d950.json
?? receipts/transition-2026-10-24T171501Z-Edit-01506fd3.json
?? receipts/transition-2026-10-24T171520Z-Edit-449d8eba.json
?? receipts/transition-2026-10-24T171539Z-Edit-819dd50a.json
?? receipts/transition-2026-10-24T171544Z-Edit-45c9011d.json
?? receipts/transition-2026-10-24T171547Z-Edit-e29be09f.json
?? receipts/transition-2026-10-24T171612Z-Edit-ce3baeda.json
?? receipts/transition-2026-10-24T171616Z-Edit-0dbcb4c8.json
?? receipts/transition-2026-10-24T171617Z-Write-e23ac78c.json
?? receipts/transition-2026-10-24T171621Z-Edit-2bf82187.json
?? receipts/transition-2026-10-24T171625Z-Edit-54cd2cd9.json
?? receipts/transition-2026-10-24T171628Z-Edit-ee2e59e1.json
?? receipts/transition-2026-10-24T171630Z-Edit-fa9487e1.json
?? receipts/transition-2026-10-24T171633Z-Edit-8c500504.json
?? receipts/transition-2026-10-24T171635Z-Edit-d1af4047.json
?? receipts/transition-2026-10-24T171638Z-Edit-567ad557.json
?? receipts/transition-2026-10-24T171640Z-Edit-954b59a9.json
?? receipts/transition-2026-10-24T171643Z-Edit-6e3eaf07.json
?? receipts/transition-2026-10-24T171648Z-Edit-74f017ab.json
?? receipts/transition-2026-10-24T171650Z-Edit-b81aed70.json
?? receipts/transition-2026-10-24T171653Z-Edit-35721b3c.json
?? receipts/transition-2026-10-24T171657Z-Edit-df452a0b.json
?? receipts/transition-2026-10-24T171659Z-Edit-3055b47d.json
?? receipts/transition-2026-10-24T171702Z-Edit-af8a0baa.json
?? receipts/transition-2026-10-24T171707Z-Edit-0cf27a11.json
?? receipts/transition-2026-10-24T171709Z-Edit-fae61c29.json
?? receipts/transition-2026-10-24T171718Z-Edit-b1e645fa.json
?? receipts/transition-2026-10-24T171737Z-Edit-28d4bb77.json
?? receipts/transition-2026-10-24T171740Z-Edit-fe7be34a.json
?? receipts/transition-2026-10-24T171742Z-Edit-25e33a1b.json
?? receipts/transition-2026-10-24T171746Z-Edit-e0d88bbf.json
?? receipts/transition-2026-10-24T171751Z-Edit-cffe3ec7.json
?? receipts/transition-2026-10-24T171756Z-Edit-fcc6b1e8.json
?? receipts/transition-2026-10-24T171800Z-Edit-117c0292.json
?? receipts/transition-2026-10-24T171801Z-Edit-5bb779a6.json
?? receipts/transition-2026-10-24T171812Z-Edit-3b5408e4.json
?? receipts/transition-2026-10-24T171817Z-Edit-21585472.json
?? receipts/transition-2026-10-24T171833Z-Edit-2d0d675e.json
?? receipts/transition-2026-10-24T171834Z-Edit-7f5188a3.json
?? receipts/transition-2026-10-24T171838Z-Edit-e60dc6ff.json
?? receipts/transition-2026-10-24T171850Z-Edit-b7e4b806.json
?? receipts/transition-2026-10-24T171907Z-Edit-40b9f869.json
?? receipts/transition-2026-10-24T171919Z-Edit-cbccfa37.json
?? receipts/transition-2026-10-24T171924Z-Write-2718531e.json
?? receipts/transition-2026-10-24T171947Z-Edit-abd5458b.json
?? receipts/transition-2026-10-24T171948Z-Edit-59f7f681.json
?? receipts/transition-2026-10-24T171954Z-Edit-2d1c0625.json
?? receipts/transition-2026-10-24T172017Z-Edit-40eb73c9.json
?? receipts/transition-2026-10-24T172017Z-Edit-99c79997.json
?? receipts/transition-2026-10-24T172050Z-Edit-014ff569.json
?? receipts/transition-2026-10-24T172052Z-Edit-70d26be0.json
?? receipts/transition-2026-10-24T172117Z-Edit-d26fd7c0.json
?? receipts/transition-2026-10-24T172122Z-Edit-d7ead016.json
?? receipts/transition-2026-10-24T172146Z-Edit-c5aa633a.json
?? receipts/transition-2026-10-24T172148Z-Edit-07e25f74.json
?? receipts/transition-2026-10-24T172203Z-Edit-e46b3bcf.json
?? receipts/transition-2026-10-24T172205Z-Edit-d866039f.json
?? receipts/transition-2026-10-24T172219Z-Edit-7898a023.json
?? receipts/transition-2026-10-24T172222Z-Write-a7a7d5d6.json
?? receipts/transition-2026-10-24T172240Z-Edit-e21d9ea4.json
?? receipts/transition-2026-10-24T172301Z-Edit-87ebeec4.json
?? receipts/transition-2026-10-24T172314Z-Write-011b7fc7.json
?? receipts/transition-2026-10-24T172317Z-Write-1b8b41f9.json
?? receipts/transition-2026-10-24T172332Z-Edit-b6fd8c7f.json
?? receipts/transition-2026-10-24T172338Z-Edit-107dc3fc.json
?? receipts/transition-2026-10-24T172343Z-Edit-05630d72.json
?? receipts/transition-2026-10-24T172353Z-Edit-8166e9af.json
?? receipts/transition-2026-10-24T172359Z-Edit-a0bc9dd0.json
?? receipts/transition-2026-10-24T172408Z-Edit-a6f8b4a0.json
?? receipts/transition-2026-10-24T172416Z-Edit-ee680f60.json
?? receipts/transition-2026-10-24T172422Z-Edit-c4bb9400.json
?? receipts/transition-2026-10-24T172428Z-Edit-77b56bab.json
?? receipts/transition-2026-10-24T172437Z-Edit-ca0a0bf0.json
?? receipts/transition-2026-10-24T172452Z-Edit-5a87bdcf.json
?? receipts/transition-2026-10-24T172452Z-Edit-78544c35.json
?? receipts/transition-2026-10-24T172524Z-Write-f848ee6b.json
?? receipts/transition-2026-10-24T172548Z-Edit-33774ee7.json
?? receipts/transition-2026-10-24T172602Z-Write-b279eed8.json
?? receipts/transition-2026-10-24T172603Z-Edit-fa07307b.json
?? receipts/transition-2026-10-24T172618Z-Edit-5244db04.json
?? receipts/transition-2026-10-24T172633Z-Edit-07ee4cdf.json
?? receipts/transition-2026-10-24T172652Z-Edit-294fc0e0.json
?? receipts/transition-2026-10-24T172708Z-Edit-d8da138e.json
?? receipts/transition-2026-10-24T172724Z-Edit-0e4d9562.json
?? receipts/transition-2026-10-24T172737Z-Edit-06be866c.json
?? receipts/transition-2026-10-24T172739Z-Edit-0f82f444.json
?? receipts/transition-2026-10-24T172759Z-Edit-f6ea8a72.json
?? receipts/transition-2026-10-24T172817Z-Edit-e90e101a.json
?? receipts/transition-2026-10-24T172832Z-Edit-947fd18b.json
?? receipts/transition-2026-10-24T172843Z-Edit-558148b3.json
?? receipts/transition-2026-10-24T172850Z-Edit-1c0ba557.json
?? receipts/transition-2026-10-24T172905Z-Edit-67449db4.json
?? receipts/transition-2026-10-24T172905Z-Edit-7246809d.json
?? receipts/transition-2026-10-24T172933Z-Edit-8d07ad96.json
?? receipts/transition-2026-10-24T172958Z-Edit-b07961b6.json
?? tests/docs_drift_courts.rs
```

(30 lines listed above; count taken from the listing, not recalled.)
Untracked: RECEIPTS.md, receipts/2026-10-24-spc-solo-rerun.md,
receipts/spc-2026-10-24T032511.json, and transition-receipt JSONs. No git
state changes were made by this lane.

## Verification-ladder position

Step-2 (suite) witness for ADR-wasmi-retirement. Deletion evidence in
`receipts/2026-10-24-wasmi-retirement-deletion.md`; R-1 gate rungs in
`receipts/2026-10-24-r1-rungs.md`.
