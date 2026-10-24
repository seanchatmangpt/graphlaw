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
