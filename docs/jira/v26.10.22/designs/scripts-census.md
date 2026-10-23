# Scripts Census — v26.10.22 (lane L93)

Date: 2026-10-22. Scope: every script must be runnable-with-receipt or loudly
retired. Verdicts: CURRENT / LINK-BLOCKED / BROKEN / ORPHAN.

## Verdicts

| Script | Verdict | Exit | Evidence |
|---|---|---|---|
| scripts/wasm_audit.sh | CURRENT | 3 (typed MISSING-SUBJECT; no wasm32-wasip1 artifact in target/) | non-vacuity: `WASM_AUDIT_FATFIXTURE=/tmp/fatfix.wasm` (67,108,865 B, header 0061736d) -> exit 1 BREACH. Gate can exit 1. Passes only on a real artifact. |
| scripts/wasm_unknown_unknown_smoke.sh | LINK-BLOCKED | 1 (typed artifact-missing) | no `target/wasm32-unknown-unknown/wasm/graphlaw_wasm.wasm`; loud fail with build command, never a silent pass. Unblocks at rung 6. |
| scripts/gen_wasm4pm_graphlaw_vectors.py | CURRENT | 0 | `--help` exit 0; `--out /tmp/glvec_smoke` after `cargo build --release -p graphlaw-eyeron` (exit 0) wrote 6 vectors incl. refusal vector `strips-unreachable-goal-refusal.json`; paper cross-check OK. Note: requires release-profile eyeron at target/release/ (docstring documents this). |
| mega/tc-mega-fibo-01/generators/gen_corpus.py | CURRENT | 0 | smoke `--legs 50 --lei 20 --holders 4 --outdir /tmp/glcensus_corpus` wrote smoke.ttl/trig/nq (sha256 printed per file); no in-tree writes. |
| mega/tc-mega-fibo-01/ontology/gen_shapes.py | CURRENT | 0 | no argparse; runs its court directly (printout: 367 shapes / 1,741 constraints / 127,325 B, twins benign=449 B mutated=451 B). Determinism: rerun -> identical aggregate sha256 75221d5d... . Writes only under mega/tc-mega-fibo-01/ontology/{shapes,twins}/ (untracked, coordinator owns git). |

## ORPHAN scan

No ORPHANs. All five are referenced: wasm_audit.sh (README.md:117,
CONTRIBUTING.md:96, CHANGELOG.md:153, docs/index.md:87);
wasm_unknown_unknown_smoke.sh (docs/WASM.md:88, docs/wasm-integration.md:124,
docs/index.md:87); gen_wasm4pm_graphlaw_vectors.py (docs/index.md:88);
gen_corpus.py + gen_shapes.py (COURT-SPEC.md:241-242, INDEX.md:97/112,
RECEIPTS.md:254, ontology/README.md:31, README.md:12/54).

## Not run (out of lane)

mutation_court.sh / spc_court.sh / doc_staleness_gate.sh /
receipt_chain_sync.sh / gen_w3c_manifest.sh are owned/current per dispatch —
not re-run here.

## Non-vacuity receipts

- wasm_audit.sh: fat-fixture probe -> `[FAIL] wasm size 67108865 exceeds
  ceiling 67108864 (64 MiB)`, `== wasm_audit: BREACH ==`, exit 1. Probe
  subject was a /tmp file; repo tree untouched.
- gen_shapes.py twins: benign 449 B vs mutated 451 B emitted per run.
- Disk receipt at census time: `df -h /` -> 14 GiB free (hook refused an
  earlier write at 11.2 GiB; stale target-lane* roots listed, coordinator
  to purge).
