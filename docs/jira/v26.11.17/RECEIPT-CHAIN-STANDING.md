# RECEIPT-CHAIN-STANDING (v26.11.17)

Lane 08, read-only. `b3sum` is NOT installed in this sandbox, so digests could not be recomputed; coverage verified by path-set comparison only.

## Components
src/attest.rs (809 L, Ed25519, TrustedKeys), src/receipt.rs (256), src/receipt_chain.rs + frame/record/store (frozen wire shapes), src/receipt_store.rs (497), scripts/receipt_chain_sync.sh.

## CHAIN.blake3 coverage (real output)
    files on disk (excl. CHAIN.blake3): 3698
    paths listed in CHAIN.blake3:       3698
    comm -23 (on disk, not chained):    (empty)
    comm -13 (chained, not on disk):    (empty)
    chain_head 16bb55deb6c95aef910baa0d4f2ff31aff15322ece20c057bba1e4ab20fa3ee0

    $ bash scripts/receipt_chain_sync.sh --check
    scripts/receipt_chain_sync.sh: line 48: b3sum: command not found

Path-set coverage 100%. Digest verification is UNVERIFIED here; re-run `--check` on a host with b3sum. `git status` clean; CHAIN.blake3 last committed dd6a80e; last dedicated receipts commit ccaa171 (2026-11-09).

Transition files by date: 10-23: 1445, 10-24: 864, 10-25: 145, 10-26: 512, 10-28: 114, 10-29: 586, 11-10: 4, 11-12: 1.

## RECEIPTS-INDEX.md
Lives at repo root (15 lines, last commit 6961d73). Two rows only (v26.10.28 RECEIPTS.md, 2026-10-24-log-retention.md); lists none of the transition/spc JSON. Newest row is 2026-10-28; no v26.11.x row.

## Latest mints (schema v2030-step3, append-only-fold)
| receipt | target |
|---|---|
| 2026-11-10T041812Z-Write-c0ec1bcd | ~/.claude/plans/ file |
| 2026-11-10T042107Z-Write-5e58ab6e | /tmp/docs-keep.txt |
| 2026-11-10T043105Z-Write-e83106b4 | docs/index.md |
| 2026-11-10T051909Z-Write-6afce29f | /tmp/docs-triage-commit.txt |
| 2026-11-12T172855Z-Write-29be64be | ~/.claude/plans/ file |

All 5 are chained; all carry `UNSIGNED-NO-KEY`; each points to a `.attest.json` that does not exist (only 3 exist, all 2026-10-23). Nothing minted 11-13..11-17.

## Open items
1. Run `scripts/receipt_chain_sync.sh --check` where b3sum exists; paste VERIFIED line.
2. Index policy: add a v26.11.x row or state transition JSON is covered only by CHAIN.blake3.
3. Signing key absent; src/attest.rs verification exercised only by 3 old attestations.
4. Dangling `.attest.json` pointers on the 5 November mints.
