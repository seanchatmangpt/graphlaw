# Coordinator chain re-baseline note (2026-10-23)

- receipts/2026-10-23-eh-closure-wave-draft.md was chained at creation, then
  edited post-chaining by two doc lanes (RESOLUTION append + witnessed-value
  fill). Chain --check flagged TAMPER-DETECTED (chain 296870d5... vs actual
  598b8e34...).
- The file was a DRAFT-PENDING-WITNESS, never a witnessed receipt; it is moved
  verbatim to docs/jira/v26.10.22/designs/eh-closure-draft.md (outside the
  receipt chain scope). Its FINAL replacement will be a witnessed integration
  receipt after the artifact trap fix (SQLRDF-24) and re-pin.
- CHAIN.blake3 is regenerated below by the coordinator as an explicit,
  documented transition (same algorithm: sorted per-file b3sum digests +
  chain_head). No other receipt changed. Precedent: the factory-b
  adjudication (coordinator re-baseline vs revert).

## Re-baseline 2 (2026-10-23)

- receipts/2026-10-23-eh-closure-wave.md drifted post-chaining (chain
  b2fa405f... vs actual 19e42378...) — a transition-receipt hook and a doc
  pass touched the tree between the filing lane's write and the chain state.
- Coordinator re-baseline #2 applied with the same documented transition;
  CHAIN.blake3 regenerated over the current on-disk receipts. All further
  receipt edits route through the integration sync, never post-chain edits.

## Re-baseline 4 (2026-10-23, coordinator)

- Trigger: `receipt_chain_sync.sh --check` exit 1 — exactly one drift:
  `receipts/2026-10-23-eh-closure-wave.md` (chained `19e42378…` vs on-disk
  `933a6e46…` per b3sum). Cause: post-baseline finalization append (§8 final
  pin values + residual-OPEN rows), same class as re-baselines 1–3. Content
  verified as the coordinator's own finalize append; 903/904 other entries
  clean; 373 files legal chain-extension.
- Action: coordinator re-baseline via `receipt_chain_sync.sh` (fold + extend).
- Verification: `--check` exit 0 required before the fix-forward commit.

## Re-baseline 5 (2026-10-23, coordinator)

- Trigger: sync-mode REFUSED on one drift — `receipts/2026-10-23-banking-4of4-witness.md`
  (chained `6f91accc…` vs on-disk `f3570f15…`): the filing lane appended its
  compile-freeze disclosure transcript after the transition hook chained the
  file at creation. Same class as re-baselines 1–4; content is the lane's own
  witness append, digest matches the RECEIPTS.md pointer row.
- Action: coordinator re-baseline (fold + extend) via the same documented
  direct-write transition.
- Verification: `--check` exit 0 before the fix-forward commit.

## Re-baseline 6 (2026-10-23, coordinator)

- Trigger: sync-mode `--check` exit 1 on two drifts, same class as
  re-baselines 1–5 (post-chaining content append by the filing lanes):
  - `receipts/2026-10-23-sqlrdf27-verdict.md`
    (chain `b7483dbdcad3a8fc15ad6d4ce3c5ccfa253a5b52ee154d20ca2d497d58959ef9`
    vs on-disk `c89d9a81d20ddebeb9a04b1f35a13e1787710943a4300ec18d5d55ea191886a0`)
  - `receipts/2026-10-23-workload-amortization.md`
    (chain `363042738342bfba942cc715f42a139782567542e97cc2d0ed4a21068de37159`
    vs on-disk `505ffaa4576a9c3ba85f4d00fe376695cb753ccb114906453c856027ebf33efa`)
- All other entries clean; no receipt content changed by this transition.
  Previous chain: 1482 receipts, head
  `ea3229020f69bbbba4db44fd350f02177344f0f827e628962495ddff8488ffaa`;
  backup at /tmp/CHAIN.blake3.rb6.bak.
- Action: coordinator re-baseline via the same documented direct-write
  transition (sorted per-file b3sum + chain_head), with this note appended
  first so the regenerated chain covers it.
- Verification: `--check` exit 0 required. Observed (2026-10-24T00:04Z):
  sync NO-OP/VERIFIED, exit 0, head
  `12cd76e086d4e3db668e64c1019fc226ab5dfbd97778f3832eba7482d122c821`
  (1493 receipt files at check time; count grows with transition receipts
  from concurrent lanes — the exit-0 verify is the gate, not the count).
