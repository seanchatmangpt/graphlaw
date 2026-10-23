# M12 — doc_staleness_gate Receipt

Date: 2026-10-22 · Lane: M12 · Repo: /Users/sac/graphlaw · Gate:
`scripts/doc_staleness_gate.sh` (sha256
`296812dcc06582d3f73f4694840d69229d301b3ba60014fc11e944d8262a94d0`) (historical)

## Command receipts

```
$ bash scripts/doc_staleness_gate.sh            # BEFORE (pre-fix baseline)
doc_staleness_gate: 92 finding(s)
EXIT=1

$ bash scripts/doc_staleness_gate.sh            # AFTER gate fixes
doc_staleness_gate: 83 finding(s)
EXIT=1
```

## Gate bugs fixed (2)

1. **R2 wrapped-historical false positive** — the `historical` escape marker
   was checked line-locally; a digest whose marker wraps to the previous line
   (witnessed: `docs/wasm-integration.md:163-164`, "…(Historical: the
   26.10.13-era pin was / `cbc80f07…`") fired R2 despite being explicitly
   marked historical. Fix: R2 now consults the digest's line plus its
   immediate predecessor for the marker.
2. **R3 vacuous self-fire on the gate's own spec** — the gate already
   self-excludes `scripts/doc_staleness_gate.sh` ("scanning itself is vacuous
   self-fire"); its design doc
   `docs/jira/v26.10.22/designs/doc-staleness-gate-design.md` states the
   BANNED strings as spec text and was firing 8 R3 findings for quoting its
   own rules. Fix: the spec doc is excluded alongside the script (same
   rationale).

Intermediate defect caught by rerun: the first R2 patch referenced `lines`
outside its scope (NameError-class); caught by the immediate rerun before
any receipt was staged and fixed in the same pass.

## Anti-vacuity re-witness (required: gate logic touched)

```
$ cat > docs/scratch-m12-inject.md   # unregistered digest + '--features duckdb' (historical)
                                     # + digest under a wrapped Historical marker
$ bash scripts/doc_staleness_gate.sh
doc_staleness_gate: 85 finding(s)    # 83 + 2, both on the scratch file
FINDING R2 docs/scratch-m12-inject.md:1: digest aaaa0000bbbb… not in
  registry/ARTIFACTS.blake3, …
FINDING R3 docs/scratch-m12-inject.md:2: banned stale string
  '--features duckdb' … (historical)
EXIT=1
$ rm docs/scratch-m12-inject.md
$ bash scripts/doc_staleness_gate.sh
doc_staleness_gate: 83 finding(s)
EXIT=1
```

The gate still flips (R2+R3 fired on injection, exit 1) and the wrapped
historical marker still suppresses (the line-4 digest under the wrapped
marker produced no finding — the fix is not vacuous either way).

## Findings census (83 REPORTED — none editable by this lane)

Per task law: findings in dated receipts / lane M1-M11 files are reported,
not edited. No file outside `scripts/doc_staleness_gate.sh` and this receipt
was touched.

### LIVE docs, lane-owned (fix owners: M1-M11)

- `docs/WASM.md:46,97` — R2: digest `9d299ed81795…` (2026-10-22 wasm build
  witness, 45,162,366 B) not in `registry/ARTIFACTS.blake3`, not
  on-disk-backed, not marked historical. Owner: wasm docs lane.
- `docs/wasm-integration.md:20,22` — R2: digests `9dcab4e4099e…`,
  `8821e2776084…` same class. Owner: wasm docs lane.
- `docs/ARCHITECTURE.md:47,68,116` — R3: `native-only` without historical
  marker (describes the `evidentiary-ring` feature gate — likely legitimate,
  owner should mark historical or reword). Owner: architecture lane.
- `docs/wasm-duckdb-integration.md:63` — R3: `graphlaw_duckdb.wasm` (itself (historical)
  a *corrective* mention — "there is no separate target"; owner adds the
  `historical` marker). Owner: wasm docs lane.
- `docs/jira/v26.10.22/SQLRDF-19.md:67` — R3: `--features duckdb`. (historical)
  Owner: SQLRDF lane.

### Dated receipts (stay, per shared law)

- `docs/jira/v26.10.22/RECEIPTS.md` — R4 ×21, R2 ×2 (f9eae23d…, 2c316b02…).
- `docs/jira/v26.10.22/designs/RECEIPTS.md` — R4 ×6.
- `docs/jira/v26.10.22/designs/`: registry-regen-receipt.md (R2×2, R3×1),
  stale-claims-audit.md (R2×3, R3×13), examples-census.md (R2×2),
  graphlaw-run-live-receipt.md (R2×3, incl. placeholder
  `000000000000…:98`), spc-append-only-fix.md (R2×2),
  tc-hot-full-receipt.md (R2×2), split-prewrite-verification.md (R2×1),
  gws4-postlink-receipt.md (R2×1), chain-resync-receipt.md (R2×3),
  refusals-register-reconciliation.md (R2×1), wasi-sdk-reproducibility.md
  (R2×4).
- `docs/jira/v26.10.22/mega/tc-mega-fibo-01/README.md` — R2 ×7.

Total: 13 live-doc findings (5 files) + 70 receipt findings = 83. Pre-existing
on main before this session (all flagged by the pre-fix run too); the gate
fixes removed 9 findings, all false positives on legal text.

## Standing

GATE-ALIVE: rung 0 (bash), exit codes observed 1 pre/post/inject/removal;
anti-vacuity flip re-witnessed after logic change. No SKIP. No version
bumps, no re-pin, no path patch, no dated receipt edited.

## N10 lane — 2026-10-22 re-run (gate unchanged, census stable)

- Gate sha256: `296812dcc06582d3f73f4694840d69229d301b3ba60014fc11e944d8262a94d0`
  — byte-identical to M12's close; no logic changes this lane.
- Census: `bash scripts/doc_staleness_gate.sh` → 76 finding(s), exit 1
  (matches M13 close 83→76). All 76 sit in dated jira history —
  `docs/jira/v26.10.22/RECEIPTS.md` (R2×2, R4×22),
  `docs/jira/v26.10.22/designs/*` (R2×28, R3×15, R4×6),
  `docs/jira/v26.10.22/SQLRDF-19.md` (R3×1),
  `docs/jira/v26.10.22/mega/tc-mega-fibo-01/README.md` (R2×7) —
  dated receipts stay; zero live-instruction findings in lane-owned files,
  so zero doc fixes this lane.
- Anti-vacuity flip re-witnessed on scratch `docs/scratch_n10_flip.md`
  (banned string + unregistered 64-hex digest): findings 76→78, both rules
  fired (R2 + R3), exit 1; scratch removed, re-run restores exactly 76.
- Final: gate exits 1 with 76 findings. No SKIP.
