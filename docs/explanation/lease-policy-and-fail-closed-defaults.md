# Lease Policy and Fail-Closed Defaults

Why a `law` request without a lease is refused by default in the Elixir wrapper, and why the
Rust kernel still defaults the other way.

## Two layers, two defaults

- Rust kernel: `LeasePolicy::NoneAllowed` is the default (`src/abi/lease.rs:29`, static
  `LEASE_REQUIRED` initialised to `false` at `:37`). It keeps lease-less rdfs/shacl/n3 chains
  working.
- Elixir wrapper: `config :graphlaw_ex, :lease_policy, :required`
  (`graphlaw_ex/config/config.exs:3`); the test env sets `:none_allowed`
  (`graphlaw_ex/config/test.exs:5`).

A production host built on the wrapper therefore fails closed without any extra setup. A
direct Rust embedder must opt in with `abi::set_lease_policy(LeasePolicy::Required)`.

## Tighten-only requests

A request may carry `"lease_policy": "required"`. `effective_policy`
(`src/abi/lease.rs:53`) returns `Required` for it, and returns the host policy for
`"none-allowed"` or an absent field. A request can never loosen a `Required` host. Any other
value is a bad-request refusal.

## Validation happens once

The lease is validated up front into a `LeaseContext` (`src/abi/lease.rs:78`). The step loop
runs a pure scope/ceiling predicate only, so an invalid lease cannot pass step N-1 and refuse
at step N. A missing lease under `Required` refuses with `LeaseRequired` before any step runs
(`src/abi/lease.rs:95`, register in docs/refusals.md).

## See Also

- `docs/refusals.md` (LeaseRequired)
- `docs/adr/ADR-lease-policy-default.md`
- `docs/explanation/hook-refuse-semantics.md`
