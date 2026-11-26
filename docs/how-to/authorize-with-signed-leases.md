# How to authorize steps with signed leases

Grant a holder authority to run named steps until an expiry, with offline
signature verification. The unsigned counterpart (`transition_leased_unverified`) exists
for tests: anyone can build a lease and pick the time — the signed path is the real one.

## Prerequisites

- A Rust host using `graphlaw::law` / `graphlaw::attest`, or an ABI/Elixir caller that can
  build a `signed_lease` + `trusted_keys` request.
- Decide the lease policy (section 3) before issuing leases.

## 1. Generate a key (deterministic, no RNG)

`graphlaw::attest` uses `ed25519-dalek =2.2.0` (pinned in `Cargo.toml`) with caller-supplied seeds:

```rust,ignore
use graphlaw::attest::{SigningKey, TrustedKeys};

let issuer = SigningKey::from_seed([7u8; 32]);
let holder = SigningKey::from_seed([9u8; 32]);
let trusted = TrustedKeys::new([issuer.verifying_key()]);
```

## 2. Issue and authorize

```rust,ignore
use graphlaw::law::{Ceiling, Clock, Lease, Step, SystemClock};
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

## 3. Choose the lease policy (Required vs NoneAllowed)

A `law` request with no lease is admitted or refused by the lease policy
(`src/abi/lease.rs:9-14,96-110`):

| policy | lease-less `law` request | default where |
|---|---|---|
| `NoneAllowed` | runs unleased | Rust ABI (`abi::set_lease_policy` unset) |
| `Required` | refused, `details.code = "LeaseRequired"` | Elixir (`config/config.exs:3`) |

Opt in to `Required`:

```rust,ignore
graphlaw::abi::set_lease_policy(graphlaw::abi::LeasePolicy::Required); // host-wide
```

```json
{"op": "law", "lease_policy": "required", "signed_lease": {}, "trusted_keys": []}
```

A request may only tighten the policy: `"lease_policy": "required"` tightens a
`NoneAllowed` host, `"none-allowed"` never loosens a `Required` host, and any other value
is refused (`src/abi/lease.rs:52-60`).

Elixir: `config :graphlaw_ex, :lease_policy, :required` is the default; `:none_allowed`
is accepted and `graphlaw_ex/config/test.exs:5` sets it for the test env
(`graphlaw_ex/lib/graphlaw_ex/config.ex:10-11,108-111`). Request opts can only tighten it.

## 4. What refuses, and why

| refusal | reason | meaning |
|---|---|---|
| `LeaseRefused` | `Expired` | now >= expires_unix (strict; judged by the verifier's `law::Clock` — `SystemClock`, or `FixedClock(t)` for audits) |
| | `OutOfScope` | step not in `scope` |
| | `Ceiling` | step requires a higher ceiling than granted (`Observe` gates, `Select` admit:plan, `Construct` derive:*) |
| `LeaseRequired` | — | policy is `Required` and the `law` request carried no lease (`docs/refusals.md:49`) |
| `UnverifiedLeaseRefused` | — | unsigned `"lease"` in the ABI without `"unverified_lease": true` + caller `now_unix` |
| `BadSignature`/`UntrustedKey` | — | signature fails, or key not in the verifier's `TrustedKeys` |
| `ClockSkew` | — | issuer's `issued_unix` leads the verifier's clock by more than `max_skew_secs` (ABI default 60) |

Signed-lease requests carry **no time field** in the ABI: `now_unix` is ignored there —
the verifier's own clock judges expiry. Skew bounds how far the issuer may lead; it never
extends an expiry.

Key custody, distribution and revocation are outside the library: a verifier is only as
good as the `TrustedKeys` it supplies. See [SECURITY.md](../../SECURITY.md).

## Verify

- Send a `law` request with `"lease_policy": "required"` and no lease: expect a refusal whose
  `details.code` is `LeaseRequired`.
- Send the same request with a valid `signed_lease` and `trusted_keys`: expect admission and
  a receipt carrying `lease_id`.
- Court: `cargo test --test trust_attest`. Not run by this edit.

## See Also

- [../refusals.md](../refusals.md)
- [use-knowledge-hooks.md](use-knowledge-hooks.md)
- [run-the-wasm-module.md](run-the-wasm-module.md)
