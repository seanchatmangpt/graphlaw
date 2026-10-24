# Code-Partition Census (first run, lane L119)

Court: `bash scripts/partition_court.sh` — exit **1** on this first run.
That is the honest verdict: G3 counts 44 unadmitted architecture leaks, and the
court refuses while any unadmitted leak exists. The leak list IS the codegen
roadmap. Not green-washed.

## Command receipts

```text
$ bash scripts/partition_court.sh; echo "EXIT=$?"
== code-partition court (subject: /Users/sac/graphlaw) ==
-- G1 COMPLETENESS --
G1 PASS: all 81 real .rs files classified exactly once
-- G2 AXIOM-IMMUTABILITY --
G2 PASS: 14 AXIOMATIC-KERNEL digests byte-identical to manifest and pins (b3sum)
-- G3 PROJECTION-PROVENANCE --
projection surface: 47 files | generator-backed: 3 | admitted leaks: 0
G3 ARCHITECTURE-LEAK: 44 handwritten projection files ...
GATE-FAIL: G3: 44 unadmitted architecture leaks (exit 1 while any unadmitted leak exists)
-- G4 DELTA-PURITY --
G4-JUSTIFIED src/evidence_ring.rs — 2 io hits covered by manifest io_justification
G4 PASS: all DELTA-SLAB files pure or justification-recorded
-- G5 RATIO --
S_handwritten = 19999 | S_derived = 17340 | S_delta = 15027 | total = 52366
ratio S_handwritten / S_derived = 1.1533
G5 MEASURED (informational)
-- ANTI-VACUITY SELF-CHECK --
ANTI-VACUITY leg1 FLIPPED: dropped 'src/bin/smon_broaden_topic.rs' from a /tmp manifest copy -> G1 refused it
ANTI-VACUITY leg2 FLIPPED: corrupted blake3 pin for 'crates/graphlaw-eyeron/src/ast.rs' -> G2 refused it
VERDICT: FAIL (1 gate violations)
EXIT=1
```

## Per-partition census (81 files, 52,531 lines)

| Partition | Files | Lines | Share |
|---|---|---|---|
| AXIOMATIC-KERNEL | 14 | 17,060 | 32.5% |
| COMPILED-PROJECTION | 47 | 20,444* | 39.0% |
| DELTA-SLAB | 20 | 15,027 | 28.7% |
| total | 81 | 52,531 | 100% |

\* projection lines split: generator-backed 445 / handwritten 19,999.

## The measured ratio (G5)

Operator law: S_handwritten ≪ S_derived. Today:

- S_handwritten (unadmitted projection leaks) = **19,999**
- S_derived (axioms 17,060 + generator-backed 445 + admitted leaks 0) = **17,505**
- ratio = **1.1425** — handwritten projection still exceeds derived code.
  The inversion of this number is the program's goal, not today's state.
  (Numbers move run-to-run only when an axiom file itself is edited — see Notes.

## Unadmitted-leak inventory (the codegen roadmap, 44 files)

Highest-value generation targets (structural derivability from the ontology):

1. `src/abi.rs` + the 6 unwired `src/abi_split/*` slices (~2.6k lines) — ABI
   dispatcher; `wasm/src/{ffi,abi_meta}.rs` already prove the ggen
   rust-wasi-wasmex-pack can render this surface from `ontologies/graphlaw-wasm.ttl`.
2. `src/bin/*` (9 CLI bridges, ~2.3k lines) — uniform argv/stdio/exit plumbing.
3. Serializer/emitter family: `src/registry_split/emit_{ttl,json}.rs`,
   `src/receipt_split/{record,store}.rs`, `src/sql_split/serializer.rs`,
   `crates/graphlaw-eyeron/src/{printing,proof_writer}.rs`.
4. Refusal tables + Display: `src/dialect_split/{refusal,sniff}.rs`,
   `src/sql_split/refusals.rs`, `crates/graphlaw-eyeron/src/error.rs`.
5. Ledger/storage bridges: `src/query_ledger*`, `src/receipt_store.rs`,
   `src/receipt_chain.rs`, `src/registry.rs`.

## Gate verdicts

| Gate | Verdict |
|---|---|
| G1 completeness | PASS (81/81 exactly once) |
| G2 axiom-immutability | PASS (14 digests byte-identical) |
| G3 projection-provenance | **FAIL** — 44 unadmitted leaks (expected, honest) |
| G4 delta-purity | PASS (1 io_justification recorded: `src/evidence_ring.rs` drainer seam) |
| G5 ratio | MEASURED = 1.1533, printed, no pass claimed |
| anti-vacuity | both legs FLIPPED (leg1 dropped-entry, leg2 corrupted-digest) |

## Notes

- `src/duckdb_eval.rs` drifted TWICE during this lane (a concurrent lane is
  editing that axiom file live in the shared checkout): once between manifest
  pin and first court run, once between the first and final receipt runs. Each
  re-pin is a court-revision event per G2's law; the court refused with
  `G2: AXIOM DRIFT src/duckdb_eval.rs` both times — the gate fires on real churn.
  Final receipt run: G2 PASS on the re-pinned digests.
- A projection file graduates out of the leak list by (a) acquiring a
  generator header + manifest `generator` field, or (b) a typed entry in
  `courts/code-partition/admitted-leaks.json` with reason + target milestone.
- Files: `courts/code-partition/manifest.json`,
  `courts/code-partition/axioms.blake3`,
  `courts/code-partition/admitted-leaks.json`, `scripts/partition_court.sh`,
  `scripts/partition_court_selfcheck_g1.py`,
  `scripts/partition_court_selfcheck_g2.py`.

## Post-wave refresh (Lane N21, 2026-10-22)

Command: `CARGO_TARGET_DIR=/tmp/gl-target bash scripts/partition_court.sh`
exit 1, `VERDICT: FAIL (5 gate violations)`.

| Gate | Verdict this run | First run | Delta |
|---|---|---|---|
| G1 completeness | PASS (81/81 exactly once) | PASS (81/81) | unchanged |
| G2 axiom-immutability | **FAIL** — 4 AXIOM DRIFT | PASS (14 digests identical) | 4 axioms drifted since pin |
| G3 projection-provenance | **FAIL** — 44 leaks | FAIL — 44 leaks | count unchanged |
| G4 delta-purity | PASS (`src/evidence_ring.rs` 2 io hits justified) | PASS | unchanged |
| G5 ratio | 19999/17505 = **1.1425** | 1.1533 | −0.0108 (handwritten lines shrank) |
| anti-vacuity | both legs FLIPPED | both FLIPPED | unchanged |

G2 drifted axioms (manifest pin vs actual, truncated digests as printed):
- `src/duckdb_eval.rs`: cc199cf615186a59.. → e04f67764cd789da..
- `src/law.rs`: 3143efeda6c2f6f8.. → 1392d5499533b58e..
- `src/plan.rs`: 7287b986a2561a81.. → 0cf1ac8931592ea3..
- `src/sql_split/ast.rs`: 748a957b9b358348.. → 4541822fba81b652..

G1 scope check (dispatch question): the manifest's owned trees are
`src`, `crates/graphlaw-eyeron/src`, `wasm/src`, `src/bin` — `examples/` and
`tests/` are not court scope, so the post-wave files
`examples/refusal_probe.rs` and `tests/wasm_duckdb_op_wired.rs` are correctly
out of scope; G1 did not and should not flag them. No unclassified files
appeared in the owned trees. Not classified here (out of lane ownership).

G3 leak set is byte-identical in count (44) to the first run; the codegen
roadmap buckets above still cover the full list. G5 improvement is line-count
shrinkage of leaked handwritten files, not leak-list graduation — no file left
the leak set.

Anti-vacuity self-check ran and flipped both legs this run (leg1: dropped
`src/bin/smon_broaden_topic.rs` from a /tmp manifest copy → G1 refused;
leg2: corrupted blake3 pin for `crates/graphlaw-eyeron/src/ast.rs` → G2
refused). The G2/G3 FAILs are therefore real signal, not a vacuous gate.

Standing: court FAIL is the honest post-wave state. Per G2's law, each drift
is a court-revision event requiring a re-pin receipt by the lanes that own
those axiom files — not silently re-pinned in this lane.

## Switchover note (2026-10-22)

The `*_split` paths named in the leak inventory and in the N21 G2 drift row
were staging folders. They are deleted. The same text now sits beside the
parents. This court has not been re-run since that move, so the 44-leak count
and the 1.1425 ratio remain the N21 receipt.

| Census path | Current path |
|---|---|
| `src/abi_split/` | `src/abi/ops/`, `src/abi/lease.rs` |
| `src/dialect_split/{refusal,sniff}.rs` | `src/dialect/{refusal,sniff}.rs` |
| `src/sql_split/{ast,refusals,serializer}.rs` | `src/sql_ast/{ast,refusals,serializer}.rs` |
| `src/query_ledger_split/` | `src/query_ledger/{batch,storage}.rs` |
| `src/receipt_split/{frame,record,store}.rs` | `src/receipt_chain/{frame,record,store}.rs` |
| `src/registry_split/{definitions,emit_json,emit_ttl}.rs` | `src/registry/{definitions,emit_json,emit_ttl}.rs` |
| eyeron `parser_split/`, `reasoner_split/` | `parser/{cursor,grammar}.rs`, `reasoner/{limits,solve_context,builtins/}` |

The operator brief that classifies axiomatic kernel, compiled projection, and
constrained delta slab is filed at
[deterministic-agent-admission.md](deterministic-agent-admission.md). The
module-split reconciliation is
[core-team-decomposition.md](core-team-decomposition.md).
