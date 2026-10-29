# GL28 — Vision 2028 Backlog (C4 vs. tree, post-v26.10.28)

Part A (landed substrate) carries the receipts in RECEIPTS.md and the live
GitHub release. Parts B-D below are the honest gaps between the Vision 2028
C4 model and the tree, each with its typed falsifier. Sequencing:
GL28-9 -> GL28-10 (close the window), GL28-6 -> GL28-7 -> GL28-5 (enforcement
teeth), GL28-1 -> GL28-8 (Article 12 persistence), GL28-2/GL28-3 (large
integrations), GL28-4/GL28-13/GL28-11/GL28-12 parallelizable.

## Part B — drawn but unbuilt

- **GL28-1** wasm4pm OCEL 2.0 conformance engine (Art. 12) — L, BLOCKED-external.
  Telemetry handler projecting [:graphlaw_ex, :admit, :stop] into OCEL 2.0
  records + a wasm4pm consumer court. Falsifier: an injected non-conformant
  event must fail the checker.
- **GL28-2** Smalt local-cloud integration — XL. Deterministic wire protocol,
  wire-parity court (same receipt bytes both paths), admission-only commit
  gate keyed on R_receipt presence.
- **GL28-3** autofde-lab/ferroplan bounded-plan integration — L. Planner
  adapter emitting law-op payloads with monotonic lease ceilings; court:
  an over-ceiling plan is refused at the turnstile (catalog C01).
- **GL28-4** EU High-Risk AI attestation exporter — M, policy-gated. Signed
  conformance-claim document per release (claims_and_evidence.md pattern).

## Part C — thin enforcement paths

- **GL28-5** Reclassify seed receipts under the EU-Article vocabulary — S.
  documentation/reference/conformance_claim.md naming court -> article clause,
  receipt-gated, falsifier column.
- **GL28-6** Article 14 kill-switch — M. Operator-held Ed25519 revocation
  lease checked per call by EngineLoad/Host; court: revocation stops
  actuation with a typed refusal, flip-twin with an honest survivor.
- **GL28-7** Untrusted-agent ingress (A2A -> turnstile) — M. Port the
  Authority.claim/check_ceiling trio into graphlaw_ex; unsigned leases
  capped at :observe.
- **GL28-8** Receipt-chain serialization — M. Persist src/receipt_chain.rs's
  BLAKE3-chained frames per admission; court: re-verification from disk
  detects a single flipped byte.

## Part D — v26.10.28 debt gating the vision story

- **GL28-9** mutation court solo run (WO-2) — the window stands
  GO-except-WO-2 until it lands.
- **GL28-10** ash_graphlaw P6/P7/P9 parity closure + full suite + coordinator
  commit (in flight).
- **GL28-11** graphlaw_ex residuals: dual refusal vocabulary consolidation,
  top-level verb re-exports, Host/Pool/Telemetry/EngineLoad unit courts,
  capability smoke default-path fix.
- **GL28-12** npm publish (executed 2026-10-29 — see RECEIPTS) + mix
  hex.publish decision.
- **GL28-13** wasm slimming SCAR (80 MiB ceiling is carrying cost).
