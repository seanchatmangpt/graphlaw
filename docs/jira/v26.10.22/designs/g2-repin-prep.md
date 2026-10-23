# G2 AXIOM-IMMUTABILITY re-pin prep (N20 runbook step 3 prep)

Lane L158, 2026-10-22. Read-only over src/; owns only
`courts/code-partition/axioms.blake3.next` (sidecar) and this doc.

## Subject

- Repo: `/Users/sac/graphlaw`
- HEAD at measurement: `615dbfaf94d407cffe174af02804d0dbfb3f8f1d`
- Court: `scripts/partition_court.sh` (G2 section), pins at
  `courts/code-partition/axioms.blake3`, manifest at
  `courts/code-partition/manifest.json` (14 AXIOMATIC-KERNEL entries).

## Drift measured (b3sum vs pins, all 14 files checked)

| file | pinned | current |
|---|---|---|
| `src/duckdb_eval.rs` | `cc199cf615186a59..` | `e04f67764cd789da..` |
| `src/law.rs` | `3143efeda6c2f6f8..` | `1392d5499533b58e..` |
| `src/plan.rs` | `7287b986a2561a81..` | `0cf1ac8931592ea3..` |
| `src/sql_split/ast.rs` | `748a957b9b358348..` | `4541822fba81b652..` |

The other 10 files MATCH byte-identically. N20's 4-file list is confirmed
complete; no additional drift. Full digests are in the sidecar. Baseline court
run (live tree): `GATE-FAIL: G2: AXIOM DRIFT` x4, `VERDICT: FAIL`, exit 1.

## Sidecar

`courts/code-partition/axioms.blake3.next` — current b3sum of all 14 files.
Pure digest lines only (the G2 parser `h, p = line.split(None, 1)` accepts no
comment lines — do not add a header comment inside the file). The commit the
repin must be applied WITH: the commit that lands the four drifted src files
(current HEAD at prep time `615dbfa` + the in-flight src diff; coordinator
names the exact SHA at application). Record it here at application time.

## Application procedure (coordinator, not this lane)

`src/` is NOT quiescent — do not apply now. When the src diff lands:

1. `mv courts/code-partition/axioms.blake3.next courts/code-partition/axioms.blake3`
   in the SAME commit as the src changes.
2. **manifest re-sync is required, pins alone are insufficient**: G2 compares
   pins AND manifest digests (`scripts/partition_court.sh:71-81`); the manifest's
   4 AXIOMATIC-KERNEL digests carry the same stale values. In the same commit,
   update those 4 entries in `courts/code-partition/manifest.json` to the
   sidecar digests (see the python re-sync in the non-vacuity receipt below).
3. Re-run `bash scripts/partition_court.sh`; expect
   `G2 PASS: 14 AXIOMATIC-KERNEL digests byte-identical to manifest and pins (b3sum)`.
   G3's 44 architecture-leak findings are pre-existing and out of scope here.

## Non-vacuity receipt (executed 2026-10-22, fixture /tmp/g2fixture.NbmWuM)

Fixture = court script + court dir + the manifest's 81 .rs files copied loose
into /tmp (not a repo copy). Two legs:

- **Leg A (sidecar works)**: applied `axioms.blake3.next` as pins + re-synced
  the 4 manifest digests → `G2 PASS: 14 AXIOMATIC-KERNEL digests byte-identical
  to manifest and pins (b3sum)`.
- **Leg B (gate can exit 1)**: corrupted pin 0 (`crates/graphlaw-eyeron/src/ast.rs`
  → all-zeros) in a /tmp copy of the pins, applied to the fixture →
  `GATE-FAIL: G2: PIN DRIFT crates/graphlaw-eyeron/src/ast.rs: pinned=0000000000000000.. actual=8a23531e25177987..`,
  court exits 1. Restored copy → `G2 PASS` again (digests byte-identical).

A gate that cannot exit 1 is the failure; G2 flips on both sides. Live-tree
baseline confirmed the flip on the drift side too (4x AXIOM DRIFT, exit 1).

## Standing

Sidecar PREPARED, not applied. Live pins and manifest still carry the stale
digests; live G2 remains FAIL-by-design until the coordinator applies the
sidecar + manifest re-sync in the src-landing commit.
