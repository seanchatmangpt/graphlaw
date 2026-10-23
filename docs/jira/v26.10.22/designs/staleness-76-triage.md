# Staleness-gate residual triage (lane L157)

- **Gate**: `scripts/doc_staleness_gate.sh`, run 2026-10-22, exit 1.
- **Live count at triage time: 91 findings, not 76** — the dispatch's 76 is stale
  (gate rules or docs moved since the count was taken). The live list is triaged here.
- **Rule split (measured)**: R2 = 48, R3 = 15, R4 = 28. Raw capture:
  `/tmp/l157-findings.txt` (session temp; re-run the gate to reproduce —
  `bash scripts/doc_staleness_gate.sh` prints all 91).
- **Method**: gate run + per-finding line read (python line extraction over both
  RECEIPTS.md files and the R2/R3 design docs). No finding files edited (lane is
  read-only on them).

## Class counts

| Class | Count | Composition |
|---|---|---|
| (a) PERMANENT-DATED-HISTORY | 46 | 46 of 48 R2 digest findings |
| (b) ACTIONABLE-POINTER | 25 | 25 of 28 R4 (prose lines, pointer appendable) |
| (c) GATE-LIMIT | 20 | 15 R3 + 2 R2 + 3 R4 (fenced embedded output) |
| **Total** | **91** | matches gate's own printed count |

## (a) PERMANENT-DATED-HISTORY — 46 findings, stays forever

46 R2 digest findings live inside dated receipt documents
(`designs/*-receipt.md`, `designs/stale-claims-audit.md`, `designs/wasi-sdk-reproducibility.md`,
`designs/pure-artifact-witness.md` — incl. two placeholder `0000…000` digests —
`mega/tc-mega-fibo-01/README.md`, and the milestone `RECEIPTS.md` §1/§7 narrative).
The digests are witnesses frozen at a point in time; the correct end state is that
they stay, unregistered, exactly as recorded. Not actionable; not a gate bug.

Two R2 findings are (a) in disposition but sit at the (c) boundary and are listed
under (c) because their truth is *verifiable* today — the gate just can't express it:
`RECEIPTS.md:116` (frozen-duckdb crates.io checksum) and `RECEIPTS.md:620`
(receipt chain head).

## (b) ACTIONABLE-POINTER — 25 findings, work list

Fix = append a `receipts/<file>` pointer on the same line (R4 requires the literal
`receipts/` token on the finding line). Proposed edits (exact lines, exact content
to append in parentheses):

### `docs/jira/v26.10.22/RECEIPTS.md` — 22 lines

| Line | Append |
|---|---|
| 24 | `(receipts/BLAKE3.chain — SQLRDF-10 trust_attest run)` |
| 51, 54, 55, 60 | `(receipts/…)` pointing at the per-court receipt file each entry names (SQLRDF-01 → sql_rdf_dialect receipt, SQLRDF-04 → chicago_validation receipt, SQLRDF-05 → duckdb_statement_kinds receipt, SQLRDF-10 → trust_attest receipt) |
| 137 | `(receipts/…)` — receipt file backing the wasm test block |
| 165 | `(receipts/query_ledger_court receipt)` |
| 170 | `(receipts/…)` — backing court run |
| 173 | `(receipts/query_cache_court receipt)` |
| 175 | `(receipts/registry_artifacts receipt)` |
| 182 | `(receipts/…)` — fuzz-court receipt |
| 201 | `(receipts/…)` — wasm test run receipt |
| 227 | `(receipts/…)` — sql_rdf_dialect re-run receipt |
| 322 | `(receipts/…)` — `cargo test -p graphlaw-wasm` run receipt |
| 451 | `(receipts/…)` — registry/registry_dispatch receipt |
| 498 | `(receipts/…)` — gws4 IPC-injection court receipt |
| 620 | `(receipts/CHAIN.blake3)` — the line already names the chain head; pointer satisfies R4 (the R2 half of this line is class (c)) |
| 719 | `(receipts/…)` — sql_rdf_dialect lane re-run receipt |

Exact receipt filenames to be resolved by the implementing lane against `receipts/`
on disk at edit time — the pointer must cite a file that exists (the gate design's
own rule: never a dead pointer).

### `docs/jira/v26.10.22/designs/RECEIPTS.md` — 6 lines

| Line | Append |
|---|---|
| 72 | `(receipts/…)` — 11-passed court run receipt |
| 84 | `(receipts/…)` — the 4/1 court receipt (this line is a real FAILED run — keep the failure visible, do not renumber) |
| 98, 99, 100 | `(receipts/…)` — receipt per cited court run |
| 149 | `(receipts/…)` — sql_rdf_conformance 4/1 receipt |

Note: line 84 and 149 are honest-failure receipts; the pointer addition must not
launder them into passes.

## (c) GATE-LIMIT — 20 findings + proposed rule changes

1. **15 R3 (banned-string) findings.** 13 are in
   `designs/stale-claims-audit.md` and 1 in `designs/registry-regen-receipt.md` —
   documents whose entire purpose is to *quote* the banned strings as an audit
   corpus; quoting them is the document working, not drifting. 1 is
   `SQLRDF-19.md:67` where `--features duckdb` appears in the ticket's
   Required-behavior CI matrix — the feature no longer exists, so this is real
   staleness, but the fix (rewrite the matrix cell to the current feature list) is
   a content edit the gate's historical-marker remedy cannot express.
   **Proposed rule**: extend the R3 allowlist mechanism from per-line
   `historical` markers to a dated per-file allowlist (same pattern R3 already
   uses for `doc-staleness-gate-design.md`), admitting
   `designs/stale-claims-audit.md` and `designs/registry-regen-receipt.md`
   wholesale; separately file a content ticket for SQLRDF-19's matrix cell.
2. **R2 `RECEIPTS.md:116`** — the digest is the frozen-duckdb 2.0.0 crates.io
   checksum, verifiable against `Cargo.lock` on disk today. **Proposed rule**: R2
   should admit any 64-hex digest that appears in `Cargo.lock` (dependency
   checksums are never repo artifacts).
3. **R2 `RECEIPTS.md:620`** — the digest is the live `receipts/CHAIN.blake3` head
   (`bash scripts/receipt_chain_sync.sh --check` exits 0 against it). **Proposed
   rule**: R2 should admit any digest found in `receipts/CHAIN.blake3` and/or
   verifiable via `receipt_chain_sync.sh --check`.
4. **3 R4 fenced-output findings** (`RECEIPTS.md:122`, `:123`, `:594`) — these are
   lines inside fenced blocks of embedded *real command output*; appending a
   pointer on the line would corrupt the receipt. **Proposed rule**: R4 should
   accept a `receipts/` pointer anywhere in the enclosing receipt block (or its
   nearest preceding heading/lead-in line), not mandate it on the same physical
   line as a count token.

## Verification

- `bash scripts/doc_staleness_gate.sh` → exit 1, `doc_staleness_gate: 91 finding(s)`
  (full list captured to `/tmp/l157-findings.txt`).
- Per-finding line reads: both RECEIPTS.md files walked with fence-state tracking;
  every R4 line printed and read; R2/R3 contexts read in
  `RECEIPTS.md:110-125,615-625`, `SQLRDF-19.md`, `designs/registry-regen-receipt.md`,
  `designs/stale-claims-audit.md`.
- No finding file edited (lane read-only on them). One file created: this triage doc.
