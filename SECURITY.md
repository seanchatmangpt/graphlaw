# Security Policy

## Supported versions

| version | supported |
|---|---|
| 26.10.10 (latest released) | yes |
| 26.10.5 | no |
| 26.9.28 | no |
| earlier / `praxis-graphlaw` | no |

## Reporting a vulnerability

Use GitHub private vulnerability reporting on `seanchatmangpt/graphlaw` (Security tab, "Report a
vulnerability"). Do not open a public issue for an undisclosed vulnerability.

## Scope

In scope: untrusted plan, policy, and JSON ABI input; receipt forgery; resource exhaustion
(memory, CPU, unbounded loops). Vulnerabilities in PurRDF or Eyeron themselves belong upstream.

## Known limits (as of the current `git log`)

- Plain receipts (`receipt::record` / `receipt::require`) prove presence only: a receipt is a
  content-addressed record, and anyone who can write a state can record one. Authenticated use needs
  the signed API: Ed25519 attestations (`attest`), `receipt::require_signed` /
  `Step::RequireSignedReceipt`, signed leases judged against a verifier-supplied `Clock`, and the
  `graphlaw-verify` binary. Key custody, distribution and revocation are outside the library.
- The wasm ABI has no signing op; hosts issue and sign leases and receipts natively. A request that
  sets `unverified_lease: true` opts out of lease signature checks and is not authenticated.
- Resource caps exist on the JSON ABI (`abi::MAX_*`); the native Rust API does not enforce them.

## Disclosure timeline (target, not a promise)

- Acknowledgement: target 7 days.
- Triage and severity: target 14 days.
- Fix and coordinated disclosure: target 90 days.
