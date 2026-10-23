# V2030-FS-BOUNDARY-SHAPES receipt (Lane V2030-2, Vision-2030 step 2)

Date: 2026-10-22. Subject: `/Users/sac/graphlaw/.claude/policy/fs-boundary-shapes.ttl`.

## Status

- Layer: SPECIFICATION ONLY. These shapes state the law; enforcement wiring is
  sequenced to P1 (V2030-1 seed graph / hook) and the sequenced hook extension.
  Nothing in this receipt claims a firing enforcer.
- P1's seed graph (`.claude/policy/graphlaw-seed-shapes.ttl`) did not exist on
  disk at write time — designed standalone. Merge point: shared IRI base
  `https://graphlaw.dev/policy#` and shared vocabulary (`policy:FileNode`,
  `policy:path`, `policy:class`, `policy:rule`); union-of-graphs merge, extend
  never re-declare.

## Structure

253 triples, 10 NodeShapes, 10/10 `sh:closed true` (rdflib 7 parse + pyshacl
0.31.0 validation).

Vocabulary: `policy:FileNode` (policy:path, policy:class, policy:rule);
`policy:boundaryClass` x8; `policy:BoundaryRule` x7.

Class coverage (8 boundary classes -> witnessed laws):

| Class | Path pattern | Rule |
|---|---|---|
| CoordinatorOwnedManifest | `^(Cargo\.toml|deny\.toml)$` | SameCommitLaw |
| GeneratedRegistry | `^registry/(ARTIFACTS\.blake3|capability-registry\.json)` | RegenerateNotHandEdit |
| GeneratedDocs | `^docs/reference-generated/` | RegenerateNotHandEdit |
| WasmArtifact | `^priv/graphlaw\.wasm(\.blake3)?$` | RebuildRepinOnly |
| ReceiptAppendOnly | `^receipts/` | AppendOnly / ImmutableOnceWritten |
| UnwiredDraft | `^(src/solve_context\|src/abi/lease\.rs)` | NotAdmittedCapability |
| RefusalAppendOnly | `^docs/refusals\.md$` | StableRefusalCodes |
| DatedReceiptImmutable | `^receipts/.*-YYYY-MM-DD\.(md\|json)$` | ImmutableOnceWritten |

Note: `src/solve_context` does not currently exist in the tree; the class
specifies the target path for the planned unwired draft. `src/abi/lease.rs`
exists (verified via ls).

## Spot-check (pyshacl, real validation, 14 cases)

| # | Path | Class | Expected | Observed | Verdict |
|---|---|---|---|---|---|
| 1 | Cargo.toml | CoordinatorOwnedManifest | admit | admit | PASS |
| 2 | deny.toml | CoordinatorOwnedManifest | admit | admit | PASS |
| 3 | registry/capability-registry.json | GeneratedRegistry | admit | admit | PASS |
| 4 | registry/ARTIFACTS.blake3 | GeneratedRegistry | admit | admit | PASS |
| 5 | docs/reference-generated/sql-rdf-conformance.md | GeneratedDocs | admit | admit | PASS |
| 6 | priv/graphlaw.wasm.blake3 | WasmArtifact | admit | admit | PASS |
| 7 | receipts/CHAIN.blake3 | ReceiptAppendOnly | admit | admit | PASS |
| 8 | receipts/sql-rdf-dialect-2026-10-22.md | DatedReceiptImmutable | admit | admit | PASS |
| 9 | src/abi/lease.rs | UnwiredDraft | admit | admit | PASS |
| 10 | docs/refusals.md | RefusalAppendOnly | admit | admit | PASS |
| 11 | Cargo.toml (typed WasmArtifact) | WasmArtifact | refuse | refuse | PASS |
| 12 | receipts/gws-9-wasm-boundary-fuzz.md (typed DatedReceiptImmutable) | DatedReceiptImmutable | refuse | refuse | PASS |
| 13 | registry/hand-edited.json (typed GeneratedRegistry) | GeneratedRegistry | refuse | refuse | PASS |
| 14 | docs/refusals.md.bak (typed RefusalAppendOnly) | RefusalAppendOnly | refuse | refuse | PASS |

Result: 14/14, anti-vacuity included (cases 11-14 are adversarial
misclassifications that must refuse). Non-vacuity witnessed: the audit first
returned AUDIT FAILED on real defects (closed shapes not declaring
`rdf:type`/`policy:rule`/`policy:class` allowances), which was fixed by
declaring permitted properties, then re-run to ALL OK — the gate flips.

## Commands

- `python3` rdflib parse: 253 triples, 10 NodeShapes, 10 closed — exit 0.
- `python3` pyshacl validate over 14 spot cases: "ALL OK" — exit 0.
- Task-gate `cargo test --test sql_rdf_dialect` not applicable: this lane
  touched no `ontologies/`, `ggen/`, or `src/` files (pure addition under
  `.claude/policy/`).

## Falsifier

Any repo path that belongs to a witnessed class but fails its pattern (e.g. a
new generated registry artifact not matching the registry allowlist), or any
misclassified FileNode that still conforms, falsifies the shapes.
