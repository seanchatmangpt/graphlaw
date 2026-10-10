# How to authorize steps with signed leases

Grant a holder authority to run named steps until an expiry, with offline
signature verification. The unsigned counterpart (`transition_leased_unverified`) exists
for tests: anyone can build a lease and pick the time — the signed path is the real one.

## 1. Generate a key (deterministic, no RNG)

`graphlaw::attest` uses `ed25519-dalek =2.2.0` with caller-supplied seeds:

```rust,ignore
use graphlaw::attest::{SigningKey, TrustedKeys};

let issuer = SigningKey::from_seed([7u8; 32]);
let holder = SigningKey::from_seed([9u8; 32]);
let trusted = TrustedKeys::new([issuer.verifying_key()]);
```

## 2. Issue and authorize

```rust,ignore
use graphlaw::law::{Clock, Lease, Step, SystemClock};
use graphlaw::attest::sign_lease;

let lease = Lease {
    id: "lease-1".into(),
    holder: holder.verifying_key().to_hex(),
    ceiling: Ceiling::Select,          // Observe < Select < Construct
    scope: vec!["admit:plan".into()],
    expires_unix: now + 3600,
    issued_unix: now,
};
let signed = sign_lease(&issuer, lease);
let (child, receipt) = state.transition_authorized(&signed, &trusted, &SystemClock, 60, &step)?;
// receipt now carries lease_id
```

## 3. What refuses, and why

| refusal | reason | meaning |
|---|---|---|
| `LeaseRefused` | `Expired` | now >= expires_unix (strict; judged by the verifier's `law::Clock` — `SystemClock`, or `FixedClock(t)` for audits) |
| | `OutOfScope` | step not in `scope` |
| | `Ceiling` | step requires a higher ceiling than granted (`Observe` gates, `Select` admit:plan, `Construct` derive:*) |
| `UnverifiedLeaseRefused` | — | unsigned `"lease"` in the ABI without `"unverified_lease": true` + caller `now_unix` |
| `BadSignature`/`UntrustedKey` | — | signature fails, or key not in the verifier's `TrustedKeys` |
| `ClockSkew` | — | issuer's `issued_unix` leads the verifier's clock by more than `max_skew_secs` (ABI default 60) |

Signed-lease requests carry **no time field** in the ABI: `now_unix` is ignored there —
the verifier's own clock judges expiry. Skew bounds how far the issuer may lead; it never
extends an expiry.

Key custody, distribution and revocation are outside the library: a verifier is only as
good as the `TrustedKeys` it supplies. See [SECURITY.md](../../SECURITY.md).
