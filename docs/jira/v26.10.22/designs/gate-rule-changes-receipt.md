# Receipt: doc_staleness_gate rule changes (L168, 2026-10-22)

Subject: `/Users/sac/graphlaw/scripts/doc_staleness_gate.sh` (untracked in
git — new file this milestone; coordinator owns git).

Scope: L157 §(c) rule-change proposals ONLY. No doc content edits; the
`docs/jira/v26.10.22/SQLRDF-19.md:67` `--features duckdb` line is a content
ticket owned by another lane (see Note at bottom).

## Baseline (pre-change, first run of the session)

`bash scripts/doc_staleness_gate.sh` → exit 1, **100 findings** (R2=54, R3=16,
R4=30, R1=0). The tree is shared and other lanes landed edits mid-session, so
later raw totals drift; per-rule deltas below are from controlled paired runs.

## Rule deltas (before → after, controlled paired runs)

| Rule | Change | Before | After | Resolved findings |
|---|---|---|---|---|
| R2 §c-2 (Cargo.lock) | Cargo.lock checksums admitted | 108 | 101 | proven by injection (below) — no pre-existing finding matches a Cargo.lock digest |
| R2 §c-3 (CHAIN heads) | receipts/CHAIN.blake3 digests admitted | 108 | 101 | 7: `RECEIPTS.md:620`, `spc-append-only-fix.md:30`, `chain-resync-receipt.md:64/75/78/109`, `post-commit-runbook.md:126` (all digest 2c316b02… ×5, 50896fa3… ×2) |
| R3 §c-1 (dated allowlist) | per-file ISO-expiry allowlist for quoting-corpus docs | 16 | 0 | all 16 pre-existing + this receipt file itself (added to `R3_ALLOW`, expiry 2026-12-31, disclosed here): `SQLRDF-19.md:67`; `stale-claims-audit.md:55,56,59,61,62,70,197,252,256,258,259,275,287`; `staleness-76-triage.md:84`; `registry-regen-receipt.md:29` |
| R4 §c-4 (block pointer) | receipts/ pointer anywhere in the enclosing non-blank block admits its claim lines | 28 | 23 | 5 (all in `designs/RECEIPTS.md` bullet blocks whose block carries the receipts/ pointer) |

Final steady state: **76 findings (R2=53, R4=23), exit 1** — remaining
findings are content defects owned by other lanes, correctly still firing.

## Flip witnesses (each rule proven both directions)

All scratch files removed after demo; final tree has zero scratch findings.

1. **R2 Cargo.lock admit** — scratch doc quoting Cargo.lock checksum
   `320119579fca…40abefa` (first checksum in Cargo.lock): BASELINE variant
   (rule reverted) → `FINDING R2 … digest 320119579fca…`; AFTER → 0 findings,
   passes. Random `deadbeef…` digest in the same doc → `FINDING R2 … digest
   deadbeefdead…` (detector not vacuous); scratch removed → gone.
2. **R2 CHAIN admit** — scratch quoting CHAIN head
   `604df95ddf5d…3bb3c4` passes (0 findings); removal of the rule (paired
   baseline diff above) re-fires the 7 real findings. Deadbeef counter-witness
   as in (1).
3. **R3 allowlist** — scratch doc with `--features duckdb` (NOT allowlisted) →
   `FINDING R3 …scratch-flip-l168.md:1`; expiry flip: setting SQLRDF-19.md's
   entry to `2026-01-01` re-fires `SQLRDF-19.md:67`; restoring `2026-12-31`
   clears it. Gate rots loudly on expiry, not silently.
4. **R4 block pointer** — scratch `designs/scratchflip/RECEIPTS.md` with
   `12 passed / 0 failed` + pointer on the next non-blank line → PASS; pointer
   separated by a blank line (different block) → `FINDING R4 …:1`; removed →
   clean.

## Commands / exits

- `bash scripts/doc_staleness_gate.sh` before: exit 1, 100 findings.
- `bash scripts/doc_staleness_gate.sh` after: exit 1, 76 findings (R2=53, R4=23).
- All flip demos exit 1 when findings present, 0-findings passes embedded in
  exit-1 runs of the shared tree (remaining findings are other lanes').

## Note (content ticket, not this lane)

`docs/jira/v26.10.22/SQLRDF-19.md:67` quotes `--features duckdb` in the plan
body. Now covered by the §c-1 dated allowlist (expires 2026-12-31); the
underlying content disposition (quote vs. fix vs. historical-marker) belongs to
the SQLRDF-19 owner.

## Fix note (2026-10-22, lane T3 — content disposition executed)

The SQLRDF-19 owner resolved the live instruction (operator tick, lanes 5-6):
`docs/jira/v26.10.22/SQLRDF-19.md:67` restated — the duckdb feature-matrix
cell is withdrawn; frozen-duckdb 2.0.2 is an unconditional crates.io
dependency, not a feature (`Cargo.toml:59-78`, `default = ["abi"]`), so its
compile/test behavior is covered by every default-features build. The
`R3_ALLOW` entry for `docs/jira/v26.10.22/SQLRDF-19.md` is **removed** from
`scripts/doc_staleness_gate.sh` (the allowlist existed only for that line;
remaining entries are unchanged). Gate receipt:

- Baseline (allowlist still active): `bash scripts/doc_staleness_gate.sh` →
  exit 1, **73 finding(s)**, all R2.
- After (allowlist removed, line fixed): exit 1, **73 finding(s)**, all R2 —
  **zero R3 findings**, and `grep SQLRDF-19` over the gate output returns
  nothing. The `--features duckdb` finding class is eliminated, not deferred;
  total did not rise. (Remaining R2 findings are other lanes' content, still
  firing correctly.)
