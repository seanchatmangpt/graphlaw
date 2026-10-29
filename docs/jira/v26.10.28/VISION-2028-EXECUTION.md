# VISION-2028 Execution Plan — Dependency DAG & Court Specifications

Baseline: graphlaw v26.10.28 (wasm blake3 c38b1fdb…, fork 2223f894…).
Companion to GL28-BACKLOG.md. Rule: an edge on the C4 diagram is
unsubstantiated prose until backed by a deterministic wire contract and a
typed falsifier court.

## Dependency DAG

- WAVE 0 (window closure): GL28-9 mutation court solo -> GL28-10 ash_graphlaw
  P6/P7/P9 closure -> GL28-12 npm/Hex publish.
- WAVE 1 (cheap enforcement teeth, in-tree): GL28-6 Art.14 Ed25519 kill-switch
  -> GL28-7 A2A Authority ceiling -> GL28-5 conformance_claim matrix.
  Parallel ungated: GL28-11 graphlaw_ex residuals, GL28-13 wasm slimming.
- WAVE 2 (Article 12 persistence): GL28-8 disk BLAKE3 receipt chain ->
  GL28-1 wasm4pm OCEL 2.0 stream -> GL28-4 EU ledger exporter.
- WAVE 3 (heavy cross-substrate): GL28-2 Smalt admission gate;
  GL28-3 ferroplan C01 lease adapter.

## Court specifications (falsifier pairs, must refuse)

- GL28-6 `art14_killswitch_court`: valid lease admits; revoked signature or
  bumped epoch -> LeaseRevoked, zero state mutation. Seam: Host pre-call
  envelope before gl_alloc (src/attest.rs + graphlaw_ex EngineLoad/Host).
- GL28-7 `a2a_authority_ceiling_court`: unsigned agent law/hooks ->
  Error.Refused(:insufficient_authority); :observe ops (sniff/parse) survive.
- GL28-5 `conformance_claim_court`: parses conformance_claim.md, verifies
  every cited test binary exists in Cargo.toml and every digest matches
  ARTIFACTS.blake3.
- GL28-8 disk chain court: re-verification from disk detects one flipped byte.
- GL28-1 `ocel2_conformance_court`: valid span passes wasm4pm schema +
  Petri-net check; missing pe:sourceDigest attribute -> trace rejected.

## Wave 3 wire contracts

- GL28-2: Smalt ingress envelope {:admitted_mutation, payload_bytes,
  receipt_blake3, engine_digest, ocel_frame}; commit gate refuses any
  PUT/POST/DELETE without a receipt validating against the graphlaw_ex
  :persistent_term digest for that payload's canonical N-Quads projection.
  Wire-parity court: identical deployment through both paths -> byte-identical
  receipt hashes.
- GL28-3: ferroplan candidate plans as glx:CandidatePlan quads submitted
  through Capability.API.law/2 under a monotonic lease ceiling;
  `c01_plan_lease_court`: critical-path sum exceeding the lease or violating
  a SHACL temporal constraint is refused pre-execution.

## GL28-13 slimming approach

twiggy/wasm-objdump section bloat census; feature-gate unused DuckDB
extensions/ICU tables out of the wasm32 build; verify codegen-units=1,
lto=fat, panic=abort; deterministic wasm-opt -Oz --strip-debug
--strip-producers before BLAKE3 minting. Target: restore the 64 MiB gate.
