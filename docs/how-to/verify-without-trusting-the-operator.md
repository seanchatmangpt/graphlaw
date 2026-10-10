# How to verify without trusting the operator

Replay an admitted plan offline — no sockets, no operator cooperation. A third party
with the start state, the plan, the receipt store and your trusted keys can re-derive
the whole chain and get `ADMITTED` or a typed refusal.

## Usage

```
graphlaw-verify --start state.nt --plan plan.json --receipts ./receipts \
    [--subject <key>] --trusted-key <hex-public-key> [--trusted-key ...]
```

Build with `--features abi`. Usage/I-O errors (including no `--trusted-key`) exit **2**.

## What it does

1. Replays `Plan::admit` from the start state.
2. Checks the receipt store holds exactly the replayed chain (parent/child ids, step,
   authority, added, plan digest).
3. Verifies each receipt's Ed25519 attestation against the trusted keys.

## Verdicts

| exit | output | meaning |
|---|---|---|
| 0 | `ADMITTED` | replay + chain + signatures all hold |
| 1 | `REFUSED: <code>: <detail>` | typed refusal: `PlanRefused`, `ReceiptTampered`, `ChainBroken`, `Unattested`, `BadSignature`, `UntrustedKey`, `MalformedAttestation`, `ReceiptMismatch` |
| 2 | usage / I/O error | malformed invocation or unreadable inputs |

## Worked example

`tests/verify_cli.rs` (6 tests, verified 2026-10-10: `6 passed; 0 failed`) drives the
real binary end to end: a robot-move plan admitted over N-Triples, every receipt signed
by a seeded Ed25519 key, written via `ReceiptStore::put_signed`, then verified:

```rust,ignore
let store = ReceiptStore::open(dir.join("receipts")).unwrap();
for r in &admitted.receipts {
    store.put_signed(r, SUBJECT, &attest::sign_receipt(signer, r)).unwrap();
}
// graphlaw-verify --start start.nt --plan plan.json --receipts receipts \
//   --trusted-key <hex>  ->  ADMITTED (exit 0)
```

Tamper cases (swap a receipt's `added` triple, drop a receipt from the middle of the
chain, verify with a wrong key) each land on a distinct refusal code — the same codes
the tests assert on.
