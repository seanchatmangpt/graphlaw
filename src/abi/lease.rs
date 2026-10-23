//! Lane L67 Phase D — lease-verification isolation DRAFT (NEW design, UNWIRED).
//!
//! Law being enacted: **Phi_lease => Valid before dispatch**. A `law` request's
//! lease is validated exactly once, upfront, into an opaque proof object; the
//! step loop then consumes the proof without re-deriving trust. No lazy
//! validation inside op loops.
//!
//! This file is a design draft. `mod lease` includes it so the draft sits
//! with the ops it describes. The types below are inside a doc fence, so
//! `LeaseContext` is not a compiled item and no op calls it. The
//! `unimplemented!` lines are not compiled. It documents the lazy-validation
//! sites in `src/abi.rs` that a later dispatch change would replace.
//!
//! # Current lazy-validation sites (replaced by this design)
//!
//! | Site | abi.rs lines | Lazy behavior today |
//! |---|---|---|
//! | `LeaseAuth::authorize` | 892-902 | Per-step: signature/expiry/scope re-judged inside the step loop (`op_law`, called at 1063-1065 and via `transition` at 1135). An invalid lease can pass N-1 steps before refusing at step N. |
//! | `LeaseAuth::transition` | 904-918 | Per-step re-dispatch to `transition_leased_unverified` / `transition_authorized`; the clock is consulted on every step (`SystemClock` at 900, 916). |
//! | `LeaseAuth::id` | 884-890 | Identity re-derived per receipt instead of fixed at admission. |
//! | `lease_auth_of` | 936-988 | Wire parsing only; attestation signature is verified lazily inside `SignedLease::authorize` (law.rs), not at the boundary. |
//! | expiry/skew check | (law.rs, via 900/916) | `expires_unix` judged per step against the module clock; an unverified lease re-trusts caller `now_unix` on every authorize. |
//!
//! # New design
//!
//! ```
//! /// Validated execution proof, returned upfront or refused. No op loop
//! /// sees an un-verified lease.
//! pub struct LeaseContext {
//!     lease_id: String,
//!     ceiling: Ceiling,
//!     scope: Vec<String>,
//!     expires_unix: u64,
//!     kind: LeaseContextKind,
//! }
//!
//! enum LeaseContextKind {
//!     /// No lease supplied: steps run unleased (authorization is a no-op).
//!     None,
//!     /// Ed25519 attestation VERIFIED at construction; expiry/skew judged
//!     /// once against the module's own clock at construction.
//!     SignedVerified,
//!     /// Explicitly unsigned (`unverified_lease: true`); carries the caller
//!     /// clock it was pinned to. Proves nothing about authority — retained
//!     /// only as a typed, loud downgrade.
//!     UnverifiedPinned { now_unix: u64 },
//! }
//!
//! impl LeaseContext {
//!     /// Total function wire -> proof. Refuses here (UnverifiedLeaseRefused,
//!     /// LeaseRefused) or returns a proof that is Valid for the whole request.
//!     /// No validation happens after this returns Ok.
//!     pub fn admit(v: &serde_json::Value) -> Result<LeaseContext, Fail> { unimplemented!("L67 draft") }
//!
//!     /// Pure predicate over already-validated facts (scope/ceiling lookup);
//!     /// never re-checks signatures, never reads a clock.
//!     pub fn authorize(&self, step: &str, required: Ceiling) -> Result<(), LawError> { unimplemented!("L67 draft") }
//! }
//! ```
//!
//! Dispatch shape after wiring:
//!
//! ```text
//! let ctx = LeaseContext::admit(v)?;      // Phi_lease decided HERE, once
//! for step in steps {                      // loop contains no validation
//!     ctx.authorize(step.name, step.ceiling)?;
//!     let (child, receipt) = state.transition_leased(&ctx, step)?; // proof-consuming
//! }
//! ```
//!
//! Wire surface is unchanged (same JSON fields, same refusal codes), so this
//! is a refactor of the trust boundary, not the ABI.
//!
//! See Also: `ops/governance.rs` (current sliced code), `docs/refusals.md`
//! (`LeaseRefused`, `UnverifiedLeaseRefused`), `src/law.rs` (`SignedLease`).
