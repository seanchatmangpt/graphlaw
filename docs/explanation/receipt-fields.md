# Explanation: receipt fields and what they mean

GraphLaw receipts carry the fields a verifier needs to re-derive an admission without
trusting the operator. The mapping below is **informal only** — GraphLaw does not claim
conformance to any external receipt schema.

| portfolio R field | GraphLaw field |
|---|---|
| identity | `parent`, `child` (state ids), `subject_sha256` (external subject digest) |
| authority | `authority`, `revision` (upstream engine), `lease_id`, attestation `key_id` |
| consequence | `step`, `added` |
| replay | `plan_sha256`, and `graphlaw-verify` replay from the start state |
| standing | not stored — derived by the verifier (`ADMITTED`/`REFUSED`) from the receipts |

## Receipt chains (praxis-compatible)

`graphlaw::receipt_chain` is a praxis-compatible receipt-chaining surface:
`ReceiptRecord` (`RECEIPT_RECORD_VERSION = 1`), frozen wire strings
(`praxis-chain/base`, `praxis-chain/v2-fold`), byte-exact BLAKE3 frame hashing —
golden-tested against the committed TCPS fixture (`tests/receipt_chain_golden_test.rs`,
4 tests) — a 4-stage validator, a JSONL `ReceiptStore`, and an epoch module. ggen
migrates onto this to retire its vendored praxis-core.

## History

The two dated session receipts are historical records, kept as written:

- [receipt-2026-10-09-receipt-chain-and-refuse-hooks](../receipt-2026-10-09-receipt-chain-and-refuse-hooks.md)
- [receipt-2026-10-10-publish-dod](../receipt-2026-10-10-publish-dod.md)