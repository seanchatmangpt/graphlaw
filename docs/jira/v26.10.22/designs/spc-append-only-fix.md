# L111: spc_court.sh append-only receipts (L108 structural finding, fixed)

## Change

`/Users/sac/graphlaw/scripts/spc_court.sh` receipt-writing block (lines ~150-162)
now stamps every receipt with a per-run timestamp:

- before: `receipts/spc-$(date +%Y-%m-%d).json` written with `>` — a second
  same-day run silently overwrote the standing receipt in place. That file
  feeds `receipts/CHAIN.blake3`, so the chain could never re-sync afterward
  (TAMPER-DETECTED forever, by construction).
- after: `receipts/spc-<date>T<HHMMSS>.json` (inject runs:
  `spc-<date>T<HHMMSS>-inject.json`), append-only — every run creates a NEW
  file; none is ever overwritten. PASS-line message updated to the same stamp.

Measurement logic untouched; only filename + stamp variable lines changed.

## Run receipts (real output)

1. Run 1: `bash scripts/spc_court.sh` → exit 1, honest FAIL verdict
   (`cpk 0.0045 < 1.33`, `p99 291ns > 250.0ns`, Shewhart rule-1 flagged count 18).
   New file created: `receipts/spc-2026-10-22T173758.json`
2. Run 2: exit 1, distinct file `receipts/spc-2026-10-22T173809.json`
   (772 bytes vs 773 bytes, different p-values) — two runs, two distinct
   files, no overwrite. APPEND-ONLY PROVEN.
3. `bash scripts/receipt_chain_sync.sh --check` → exit 1:

   TAMPER-DETECTED: receipts/spc-2026-10-22.json
   (chain=1da77e6b743577d627aa588f1e1264fea4ff9bcf86b8dd3f30ecca0ba4cbbc3a
    actual=50896fa3253cf162946b5c7eb6eb325fe909cda9305cdb3359619fa52ba11708)

## Permanent history: the old drift stands (do NOT re-baseline by default)

`receipts/spc-2026-10-22.json` was overwritten in place by a pre-fix run
(its digest 50896fa3... differs from the chained digest 1da77e6b...). That
drift is permanent history: the overwrite already happened and cannot be
un-happened. The chain refusing to verify is the chain working correctly.

## Re-baseline DECISION (coordinator-owned, not lane-executed)

The chain goes green only via an explicit coordinator decision to re-baseline:

    bash scripts/receipt_chain_sync.sh sync        # fails: TAMPER-DETECTED
    # → the only path to green is regenerating the chain head, i.e. ACCEPTING
    #   the current on-disk bytes of receipts/spc-2026-10-22.json as the new
    #   baseline: delete/move CHAIN.blake3 and re-run `sync`, or hand-author
    #   a CHAIN.blake3 line replacing 1da77e6b… with 50896fa3…

Concretely a coordinator would run:

    rm receipts/CHAIN.blake3 && bash scripts/receipt_chain_sync.sh sync

Implication: this discards the old chain head (1da77e6b743577d6…) and
certifies the post-overwrite bytes of spc-2026-10-22.json — i.e. a human
asserts "the overwrite was a defect of the harness, the new content is the
true record." Until that decision is made, `--check` stays exit 1
(TAMPER-DETECTED) and that is correct behavior. This lane did NOT re-baseline.

## Standing

- spc_court.sh append-only fix: ALIVE (two-run witness above).
- chain state: TAMPER-DETECTED / exit 1 on the old file — historical drift
  stands; re-baseline is a coordinator DECISION, pending.
