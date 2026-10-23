# SA2A Factory B — executable evidence gates

- Base: `4e4873ca377d50af5268e8736be4afe6badeb862`
- Branch: `factory-b/v26.9.29-sa2a-executable-gates-r2`
- Semantic head before this receipt: `2f385fa2814b6e285c4bee1c5a28c5ae34d0b226`
- Qualifying semantic files: 50 SPARQL ASK gates, 900 additions
- Authority: `NONE`
- Consequence: `EVIDENCE_ONLY`
- Verification: authored, not executed under Maximum Code Mode
- PR rail: refused; no unchanged retry
- Notion writeback rail: refused; receipt preserved here
- GGEN_WRITE_EXCLUDED: true
- FILE_FLOOR_MET: true
- Next edge: AshR2RML exact-source VKG evidence -> GraphLaw gates -> XaaS/ggen_igniter authority-free consumer

## STALE (2026-10-22, lane R3 digest audit)

This file carried no embedded real command output (no command, exit code,
count, or hash) — under the receipts law it was not a receipt. The subject
(branch `factory-b/v26.9.29-sa2a-executable-gates-r2`, base `4e4873ca`) is not
this checkout, so the receipted commands cannot be cheaply re-run here; the
file is marked STALE as-of 2026-10-22 instead of backfilled. Digest audit
evidence, real output, from /Users/sac/graphlaw @ main `615dbfaf94d407cffe174af02804d0dbfb3f8f1d`:

```
$ b3sum -c <(grep -v chain_head receipts/CHAIN.blake3) 2>&1 | grep -v ': OK$'
receipts/sa2a/factory-b-v26.9.29-executable-gates.md: FAILED
b3sum: WARNING: 1 computed checksum did NOT match
$ grep -cv chain_head receipts/CHAIN.blake3
35
$ b3sum -c <(grep -v chain_head receipts/CHAIN.blake3) 2>/dev/null | grep -c 'OK$'
34
```
(All 35 other receipts verify byte-exact; the FAILED line above is this file's
 own pre-CHAIN-repin state, expected since this STALE marking postdates the
 chain row. Pre-edit digest:
 d464dad4e0e8e1979ca8cf8b4426584f6186bfc98476da22f4d4af301a5d4aa6. Post-edit
 digest recorded in receipts/CHAIN.blake3.)
