# GL28 status re-read, 2026-11-17

Authority: docs/jira/v26.10.28/GL28-BACKLOG.md. Checked against tree at dd6a80e (migration snapshot, 50 commits). No cargo/mix run; no verdict is a witnessed run. Sibling checkouts (smalt, beam4pm, ferroplan, ash_graphlaw, ggen-marketplace) are absent, so claims living only there are UNVERIFIED.

| Ticket | Register status | Evidence on disk | Verdict |
|---|---|---|---|
| GL28-1 | ALIVE @ cd69757 | commits cd69757, 199cca1, fc1576f real; receipts/graphlaw_bridge/*.json absent; bridge lives in beam4pm | UNVERIFIED |
| GL28-2 | PARTIAL; kernel receipt_blake3 mint open | zero `receipt_blake3` in src/; smalt SHAs do not resolve | CONFIRMED (kernel half); smalt rows UNVERIFIED |
| GL28-3 | unbuilt | no CandidatePlan / c01_plan_lease_court outside the register | CONFIRMED |
| GL28-4 | spec only | src/attestation_export.rs and tests/attestation_export.rs absent | CONFIRMED |
| GL28-5 | LANDED | docs/reference/conformance_claim.md tracked; no conformance_claim_court in tests/; register cites ARTIFACTS.blake3 but only ASSETS.blake3 at repo root | DRIFTED |
| GL28-6 | LANDED 4d2ffd0 | kill_switch.ex (:lease_revoked) + kill_switch_test.exs | CONFIRMED (run not witnessed) |
| GL28-7 | LANDED 4d2ffd0 | authority.ex, :ceiling_unmet, a2a_authority_ceiling_court_test.exs | CONFIRMED (run not witnessed) |
| GL28-8 | thin: persist receipt_chain frames | src/receipt_chain/store.rs has open/append/load_all; tests/receipt_store.rs, tamper tests exist; per-admission wiring not shown | DRIFTED |
| GL28-9 | CLOSED (28/19/9/0) | commits 66fc0e2, c070392, f1321ca resolve; cited receipt absent (archived in 6961d73); mutation-court-L3.log has no tally | UNVERIFIED |
| GL28-10 | PARTIAL | no ash_graphlaw here; 55c1bc3 does not resolve | UNVERIFIED |
| GL28-11 | open residuals | defdelegate re-exports exist; host/pool/telemetry/engine_load/smoke tests exist; 43ea53e, 49b6316 postdate register; dual refusal vocab not checked | DRIFTED |
| GL28-12 | NOT executed | no publish receipt on disk; npm ledger archived | CONFIRMED (no receipt); 26.10.14 claim UNVERIFIED |
| GL28-13 | CLOSED 95cd3df | priv/graphlaw.wasm = 62,909,537 B; wasm_size_court MAX_BYTES=134217728 | CONFIRMED (digest not recomputed) |
| GL28-14 | LANDED 14df7b9 | mcp-manifest.json + tests/mcp_manifest.rs exist | CONFIRMED (run not witnessed) |

## Tally
CONFIRMED: GL28-3, 4, 6, 7, 13, 14 (+ partial confirmations GL28-2, GL28-12). DRIFTED: GL28-5, 8, 11. UNVERIFIED: GL28-1, 9, 10.

## Recommended register edits
1. GL28-5: add the court or correct the court row; fix ARTIFACTS vs ASSETS name.
2. GL28-8: re-scope to what is actually unbuilt.
3. GL28-11: tick re-exports and unit courts; name the remainder.
4. GL28-9/-1/-12: repoint receipt paths archived by 6961d73.
5. Re-verify smalt / ash_graphlaw / ggen-marketplace SHAs in their own repos.
