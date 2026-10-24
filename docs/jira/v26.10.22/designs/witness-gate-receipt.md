# witness-gate-receipt

Lane M02 (GUARD BUILDER), v26.10.22, 2026-10-22.
Lease: `scripts/witness_gate.sh` (new),
`docs/jira/v26.10.22/designs/witness-gate-receipt.md` (this file).

## Law

A factual build/run witness in `docs/**/*.md` must cite an existing
`receipts/` row or an existing path. `scripts/witness_gate.sh` scans
`docs/**/*.md` (excluding generated `docs/reference-generated/`) for blake3
digests (64-hex) and lane citations; each must resolve to a `receipts/` file
containing it, or an existing dir/file on disk. With file args, only those
files are checked. Exit 1 = findings, exit 2 = typed refusal (no receipts/
dir). A gate that cannot exit 1 is the failure.
Skips fenced code blocks (quoted court output is citation, not witness) and
lines carrying `<!-- witness-gate:allow -->` (defect-register prose).

## Defect class (source)

<!-- witness-gate:allow -->
The defect: `docs/WASM.md:42-48` cites a build witness in a lane target dir
(witnessed 2026-10-22, lane lease deleted at integration; no lane dirs on disk <!-- witness-gate:allow -->
→ no matches) and the digest (9d299ed8…51f0) appears in no `receipts/` file. <!-- witness-gate:allow -->
Lane roots are leases deleted at integration; a durable witness must cite a
receipts/ file.

## Non-vacuity — FAIL direction (real output)

Command: `bash scripts/witness_gate.sh docs/WASM.md`

```
WITNESS-GATE FAIL: docs/WASM.md:44 cites nonexistent lane dir target-lane114 (build witness must cite receipts/)
WITNESS-GATE FAIL: docs/WASM.md:47 unregistered blake3 digest 9d299ed81795f3011321b268de1ccc008da74b4fcbbcfa4b3020e61c2d2a51f0 # historical
WITNESS-GATE FAIL: docs/WASM.md:99 unregistered blake3 digest 9d299ed81795f3011321b268de1ccc008da74b4fcbbcfa4b3020e61c2d2a51f0 # historical
exit=1
```

(WASM.md:99 is the same historical witness repeated in the Pin currency
section.)

## Non-vacuity — PASS direction (real output)

Fixture `/tmp/m02_fixture_ok.md`:

```
# Fixture: registered witness
Build witnessed in receipts/wasm-artifact-pin-2026-10-22.md, digest
332ea2188fafb465666e71407e8f940c5b60a15184e6ad6567bb4f723fe2c30c
(6,793,617 B). No lane dir cited.
```

Command: `bash scripts/witness_gate.sh /tmp/m02_fixture_ok.md`

```
WITNESS-GATE PASS
exit=0
```

The cited digest is registered: it appears in
`receipts/wasm-artifact-pin-2026-10-22.md` (verified via `grep -rl`, 1 file).
Cross-check: `docs/WASM.md:91` cites the same `332ea218…` digest and the gate
does NOT flag line 91 — only unregistered digests are flagged.

## Full-sweep census (real output, pre-existing debt)

Command: `bash scripts/witness_gate.sh` (full docs sweep), exit=1 in ~5s.
178 findings across docs/ (excluding reference-generated/), including the
WASM.md findings. This is the live defect-class inventory for the
coordinator; fixing those docs is outside the M02 lease. Until the census is
drained, run the gate with file args (per-doc courts), not as a repo-wide
must-pass gate.

## Reproduce

```
bash scripts/witness_gate.sh docs/WASM.md          # expect exit 1 (live defect)
bash scripts/witness_gate.sh <file.md>             # per-doc court
bash scripts/witness_gate.sh                       # full census
```
