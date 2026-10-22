# SQLRDF-20 — BEAM hot-upgrade protocol: artifact swap under the supervisor, with its own court

- **Id:** SQLRDF-20
- **Title:** Hot upgrade = artifact swap under BEAM — the upgrade protocol court
- **Status:** Open (follow-up)
- **Standing:** **BLOCKED-BY: v26.10.22 integration commit + SQLRDF-19** — the
  court below drives the `runner` surface, which today has zero coverage
  (SQLRDF-19 cell 1); nothing to court against until both land.
- **Source:** operator BEAM hot-upgrade protocol, filed 2026-10-22.

## Current behavior (verified on disk, 2026-10-22)

- The compiled-in admission set lives in the wasm artifact
  ([`priv/graphlaw.wasm`](../../../priv/graphlaw.wasm), digest sidecar
  `priv/graphlaw.wasm.blake3`, pinned as a row in
  [`registry/ARTIFACTS.blake3`](../../../registry/ARTIFACTS.blake3)).
  Changing the law means building and swapping that artifact — there is no
  other carrier of the admission logic.
- The runner already has a digest-pinning mechanism:
  `--expect-blake3 <hex>` verifies the artifact against a pinned digest before
  any frame is served ([`src/bin/graphlaw_run.rs:140-170`](../../../src/bin/graphlaw_run.rs)).
  No test in the tree exercises this flag (SQLRDF-19 cell 1).
- Ring/ledger evidence is native-side: `src/evidence_ring.rs` and
  `src/query_ledger.rs` compile under
  `feature = "evidentiary-ring"` on `not(target_arch = "wasm32")`
  (`src/abi.rs:477`), write host files with BLAKE3 chains, and sit outside the
  wasm boundary. An artifact swap does not touch them.
- There is no upgrade court: no test records an artifact digest, drives a swap,
  and verifies admission-table version + receipt continuity across the swap.
  A hot upgrade today is an operational gamble — nothing pins the new
  artifact's admission set before it accepts a graph.

## Required behavior

1. **Hot upgrade is a versioned release event, receipted as one.** Swapping
   the artifact is not a deployment detail; it carries its own receipt:
   re-pinned receipt-surface digest, a new `ARTIFACTS` row, and a CHANGELOG
   entry — all landed **before** the new artifact accepts a single graph.
   The admission set of the new artifact is pinned (via `--expect-blake3`)
   before admission begins.
2. **Evidentiary continuity across the swap.** Ring and ledger are host-side,
   outside the wasm boundary, so an upgrade must behave as a controlled
   instance of the gap-marker discipline: the first post-upgrade ledger batch
   records the admission-table version of the new artifact, and the BLAKE3
   chain is verified unbroken across the boundary.
3. **No-lexer consequence, compounded in wasm.** The 6.3 MiB artifact carries
   no Turtle/parser surface: the parse attack surface is *absent from the
   sandbox*, not defended inside it. Preserve this invariant in the upgrade
   contract:
   - pure-data refusals cross wasm↔BEAM as a small stable struct — the FFI
     contract stops churning when prose changes;
   - `compile_from_bytes` means BEAM callers pass build-stamped canon bytes —
     the wasm binary never parses caller-controlled input. The upgrade court
     asserts both, on the real artifact.
4. **REQUIRED COURT (the deliverable).** One integration court, driving the
   real `graphlaw-run` bin over the real artifact:
   1. artifact digest pinned via the existing `--expect-blake3` mechanism
      (wrong digest must refuse loudly, correct digest must serve);
   2. admission-table version recorded in the first post-upgrade ledger batch;
   3. receipt chain verified across the boundary (pre-swap tail → post-swap
      head, BLAKE3-linked, no gap without a typed gap marker).
   Passing this court turns hot upgrade from an operational gamble into a
   replayable transition.

## Acceptance

- `cargo test --all-features --test <upgrade-court>` — the new court in this
  ticket's deliverable, spawning the real `graphlaw-run` subprocess over
  `priv/graphlaw.wasm` (Chicago: real subprocess, real artifact, assert on
  final state — no mocks), green.
- Mutation check: pin a wrong digest → court refuses; break the ledger chain
  in a temp copy → court fails closed. A court that cannot flip is FAIL.
- Depends on the SQLRDF-19 runner smoke test landing first — that test is the
  harness scaffold this court extends.

## Boundary (already implemented, not re-asked)

- `--expect-blake3` digest verification (`src/bin/graphlaw_run.rs:148-170`)
  exists; the ticket asks only for its first test coverage plus the ledger/
  receipt-continuity assertions.
- Host-side ring/ledger with BLAKE3 chains
  (`src/evidence_ring.rs`, `src/query_ledger.rs:63`) exist; the ticket asks
  only that upgrade continuity across them be courted.
