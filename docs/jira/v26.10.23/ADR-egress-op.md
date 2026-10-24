# ADR-egress-op: Promote Egress to a First-Class ABI Op

## Status

Proposed, 2026-10-23. Consumes the ERRC-raise findings in
`docs/jira/v26.10.23/ROADMAP.md` ("RAISE egress to a first-class ABI op").
Consumes the CanonArrow Phase 2 wire format
(`docs/jira/v26.10.22/ADR-canonarrow-dual-witness-egress.md`).

## Context

Egress carries the strongest adoption signal in the kernel and the weakest
conformance surface — an inversion the repo's own typed-refusal law
(`docs/refusals.md`) applies to its own registry.

- **Adoption**: 6.83× serialization-tax baseline (Lane 4, v26.10.22;
  text rendering 9.246 ms / 10k rows vs raw-bytes digest proxy ~992 µs) is
  the measured mandate for columnar egress. The FIBO egress leg runs 4/0
  (`tests/wasm_fibo_fixtures_eh.rs`); egress is adopted across a dozen
  courts; the tc08 COPY egress EINVAL remains open as EH-19
  (`docs/jira/v26.10.22/RECEIPTS.md` W59: "tc08 egress EINVAL → EH-19,
  FIBO 3/4 → EH-19").
- **Surface**: zero registry presence — no row in `OPS`
  (`src/registry/definitions.rs`), no `dispatch` arm (`src/abi.rs`), no
  typed refusal codes (`docs/refusals.md` REFUSAL-01..52 have no egress
  arms), no op-examples in `capabilities.ops`. The `duckdb` precedent shows
  the current mechanism: `NATIVE_ONLY_OPS` (`src/registry/definitions.rs:876`)
  exists precisely to keep ops *off* the hashed table — i.e. the registry
  structurally cannot advertise egress today, and absence is documented as
  "not a capability refusal and is not REFUSAL-04" — a typed no-op.
- **Antipattern**: the silent-skip antipattern. A consumer probing
  `capabilities.ops` for `egress` gets a silent absence indistinguishable
  from "this kernel has no egress," while a dozen courts exercise it in the
  same tree.

## Decision

Promote egress to a first-class ABI op `{"op":"egress", ...}`:

1. **Registry row** in `src/registry/definitions.rs` `OPS`, wire name
   `"egress"`, with fields
   for target (text vs CanonArrow Arrow IPC stream), receipt version, and
   columnar digest leg; op-examples embedded in the op row so
   `capabilities.ops` carries runnable examples, not a name-only advertisement.
2. **Dispatch arm** in `src/abi.rs` `dispatch` (the op-15 pattern: the arm
   dispatches inside the module on every target; the `duckdb` precedent shows
   the unadvertised state is a digest-stability choice, not a host-only
   constraint — egress is module-owned the same way).
3. **Typed refusal arms**: EH-19's raw `EINVAL` becomes a typed code. New
   REFUSAL-53 (`EgressOpenatMismatch` — the wasi-libc openat word-mismatch
   channel behind tc08) and REFUSAL-54 (`EgressColumnarMismatch` — a flipped
   Arrow byte fails closed at the egress boundary, the consumer-side law
   proven by the FIBO leg and demanded by the Arrow consumer demo). Codes are
   append-only per `docs/refusals.md`; never renumber.
4. **Perf court**: the Divan egress baseline is promoted from a bench to a
   TC-HOT-style perf court — release-mode, budget-gated (the serialization
   tax stays under the 6.83×-mandated ceiling), hardware variance disclosed,
   a gate that can exit 1.
5. **Digest-bump ceremony**: `registry_blake3` moves. Per
   `src/registry/definitions.rs:867-876`, the digest moves only on an
   explicit bump: add the op to `OPS` (removing it from any
   `NATIVE_ONLY_OPS`-style suppression), update `tests/registry_dispatch.rs`
   composition (`op_names() + NATIVE_ONLY_OPS` == parsed `dispatch` arms),
   regenerate `registry/capability-registry.json` / `.ttl` /
   `ARTIFACTS.blake3` via the registry sync flow, then quiesced-regen
   `CHAIN.blake3` per the release runbook. Hand-patching the registry
   artifacts is refused (`LITHO-REFUSED(registry-artifact)`).
6. **Phase 2 ordering**: this ADR consumes CanonArrow's wire format
   (`ArrowEgressHeader`, `receipt_v2` sealing `triples_digest` +
   `columnar_digest`) and is sequenced after Phase 2 CanonArrow lands —
   `src/egress/arrow.rs` is currently DCE'd with no courts
   (`docs/jira/v26.10.23/ROADMAP.md` G2); promoting an op whose payload type
   has no courts would advertise an unverifiable surface.

## Alternatives Considered

- **Status quo (rejected)**: silent absence from the registry. This is the
  silent-skip antipattern the repo's own law forbids elsewhere: a gate that
  cannot fail, an advertisement that cannot be probed. The `duckdb`
  NATIVE_ONLY_OPS carve-out documents absence as "not REFUSAL-04" — a typed
  no-op that leaves consumers with an unprovable UNKNOWN.
- **Advertise without typed refusals (rejected)**: registry row + dispatch
  arm but EH-19 stays a raw `EINVAL` surfacing through wasi-libc. Half the
  promotion: the surface would be advertised but un-conformed — a failure
  mode with no stable code violates the typed-refusal law
  (`docs/refusals.md`: every refusal has a stable code + trigger + recovery).
- **Advertise before Phase 2 (rejected)**: promote the op while
  `src/egress/arrow.rs` is DCE'd. Advertises an unverifiable payload type;
  the digest-bump ceremony would move `registry_blake3` twice.

## Phased Rollout

- **Phase 2 (precondition)**: CanonArrow lands (constructor + courts;
  `tests/canonarrow_egress_court.rs` Twin A/B per ROADMAP step 14). This ADR
  consumes its wire format.
- **Phase 3a**: registry row + dispatch arm + REFUSAL-53/54 + op-examples;
  digest-bump ceremony in one atomic transition (registry artifacts
  regenerated, not patched; CHAIN quiesced-regen appended).
- **Phase 3b**: Divan baseline promoted to a TC-HOT-style perf court
  (release-mode, budget-gated, exit-1-capable), folded into the test ladder
  between `cargo test` and the mutation court.
  Falsifier: the perf court cannot exit 1 under an injected serialization-tax
  regression, or `capabilities.ops` still omits `egress` after the digest
  bump.

## Consequences

- `registry_blake3`, `capability-registry.json`, `ARTIFACTS.blake3` move once,
  by ceremony — never hand-patched.
- EH-19's failure channel gets a stable code; wave-close rows flip
  REPAIR → CLOSED only on a witnessed court.
- Egress becomes probe-able: consumers distinguish "kernel has egress" from
  "kernel has no egress" mechanically.

## Acceptance

- `cargo test --test registry_dispatch` — op/composition equality holds
  post-bump.
- `cargo test --test trust_attest` and the sql-rdf acceptance set stay green
  across the digest-bump transition.
- `cargo test --test canonarrow_egress_court` (Phase 2) green before 3a.
- Perf court exits 1 under an injected serialization-tax regression
  (anti-vacuity: a court that cannot flip is FAIL).

## See Also

- `docs/jira/v26.10.23/ROADMAP.md` (ERRC-raise findings)
- `docs/jira/v26.10.22/ADR-canonarrow-dual-witness-egress.md`
- `docs/refusals.md` (typed refusal register, append-only)
- `src/registry/definitions.rs` (digest-stability comments, NATIVE_ONLY_OPS)
