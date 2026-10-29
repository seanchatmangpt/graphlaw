# GL28-2 — Smalt Wire Protocol Specification

- **Id**: GL28-2
- **Title**: Smalt local-cloud integration — deterministic wire protocol,
  admission-only commit gate, wire-parity court
- **Status**: DRAWN (spec only; nothing built)
- **Conformance target**: byte-identical receipt hashes through both ingress
  paths (direct graphlaw_ex and Smalt-routed); zero state mutation on any
  refusal; every refusal is typed and stable-coded.
- **Backlog row**: `docs/jira/v26.10.28/GL28-BACKLOG.md` (GL28-2, Part B)

## 1. Scope

graphlaw_ex is the law-admission turnstile. Smalt is a client of it: Smalt's
ingress router may commit a mutation only when graphlaw_ex admitted it, and the
proof of admission is the receipt, not a boolean. This spec fixes the envelope,
the commit gate, the framing, the refusal taxonomy, the parity court, and the
falsifiers — so that graphlaw_ex and Smalt can be built/tested concurrently
without a negotiation.

Non-goals: Smalt's internal storage model; the A2A transport (GL28-7 covers
untrusted-agent ingress); receipt-chain serialization (GL28-8).

## 2. Admission envelope

The single wire unit from graphlaw_ex to Smalt is the 5-tuple:

```
{:admitted_mutation,
  payload_bytes,      # the exact mutation body graphlaw_ex admitted
  receipt_blake3,     # BLAKE3 of the canonical receipt document
  engine_digest,      # digest of the graphlaw engine/wasm artifact used
  ocel_frame}         # OCEL event frame for the admission (GL28-1 vocabulary)
```

Field rules:

1. `payload_bytes` is the byte-exact payload that crossed the turnstile. A
   Smalt-side re-canonicalization is a spec violation (the gate must verify
   against the admitted bytes, not a recomputed projection).
2. `receipt_blake3` is `BLAKE3(Canon(receipt))` — the same fold the Rust kernel
   computes (`purrdf::try_canonicalize` path, CLAUDE.md Receipts convention).
3. `engine_digest` pins the engine identity so a receipt minted by a different
   engine build does not validate cross-build.
4. `ocel_frame` is advisory metadata (telemetry GL28-1); the commit gate MUST
   NOT consult it for the admit decision — the decision input is
   `receipt_blake3` vs `engine_digest` + payload only. This keeps the gate
   decidable without the telemetry subsystem up.

## 3. Smalt ingress router commit gate

Gate law (all branches zero state mutation on refusal):

1. Method filter: `PUT/POST/DELETE` are mutations; `GET/HEAD/OPTIONS` pass
   through ungated.
2. On a mutation, Smalt MUST have an in-hand `{:admitted_mutation, ...}`
   envelope for this request. No envelope → refuse `SMALT-NO-RECEIPT`.
3. Smalt holds graphlaw_ex's current engine digest in a
   `:persistent_term` (key `:gl_engine_digest`), refreshed by a graphlaw_ex
   push (or poll with a bounded backoff — see open questions).
4. Canonical N-Quads projection: Smalt projects the request payload to its
   canonical N-Quads projection and checks `receipt_blake3` validates
   against `(payload projection, engine_digest)` for THAT payload's
   projection — a receipt for a different payload or engine never validates.
5. Validation passes → commit, then (and only then) record the receipt.
6. Any refusal leaves Smalt state byte-identical: no ETS write, no journal
   append, no counter bump attributable to the request (an OCEL refusal frame
   is the one sanctioned write, GL28-1 vocabulary, `:refused` class).

Validation sub-steps (ordered, first failure wins):

1. envelope present → `SMALT-NO-RECEIPT`.
2. `payload_bytes == raw request body` (no re-canonicalization; the gate
   verifies against the admitted bytes) → `SMALT-PAYLOAD-MISMATCH`.
3. receipt validates vs digest for the payload's canonical N-Quads
   projection → `SMALT-RECEIPT-INVALID`.
4. `engine_digest` == `:persistent_term` digest → `SMALT-ENGINE-DRIFT`.
5. receipt replay inside graphlaw_ex (optional deep mode, see §7) →
   `SMALT-RECEIPT-REPLAY-FAIL`.

## 4. Packet framing (packet-4, reused as-is)

Grounded in `graphlaw_ex/lib/graphlaw_ex/abi.ex`:

- A packet-4 frame is `<<length::32-big-endian, payload::binary>>`
  (abi.ex:22-24, `frame/1` at abi.ex:75).
- Framing metadata is the length prefix only; the payload alone lands in
  linear memory (abi.ex:22-24).
- 16 MiB request cap: payloads above `GraphlawEx.ABI.max_request_bytes/0`
  are refused `:resource_limit` BEFORE any engine allocation
  (abi.ex:28-29, 80-90). The Smalt gate inherits the same cap: a Smalt
  mutation body > 16 MiB is refused with its own code `SMALT-OVERSIZE`,
  kept distinct from graphlaw_ex's `:resource_limit` so the two refusal
  vocabularies stay separable (GL28-11 consolidation pending).
- UTF-8 discipline: `frame/1` refuses non-UTF-8 payloads with
  `:invalid_encoding` (abi.ex:78). The Smalt envelope payload must be valid
  UTF-8 for the JSON request classes; `payload_bytes` in the envelope may be
  arbitrary bytes only if the statement class is binary-framed (open
  question Q3).
- Packed result: `gl_call` returns `<<ptr::32, len::32>>` packed into one
  u64; `unpack_result/1` (abi.ex:49-65) normalizes a signed i64 view by
  adding 2^64 before splitting. Smalt never sees this — it is
  graphlaw_ex-internal — but the spec pins it so a Rust-side
  re-implementation of the envelope does not reinvent a second packing.

## 5. Error taxonomy (typed refusals)

Two vocabularies, kept distinct and mapped 1:1 where they meet:

- graphlaw_ex side: `%GraphlawEx.Refusal{code: ...}` — existing codes
  (`:invalid_json`, `:invalid_encoding`, `:resource_limit`,
  `:malformed_response`, ...). Codes are append-only (mirrors
  `docs/refusals.md` discipline).
- Smalt gate codes (new, append-only, stable):
  - `SMALT-NO-RECEIPT` — mutation without an envelope.
  - `SMALT-PAYLOAD-MISMATCH` — envelope bytes ≠ request body.
  - `SMALT-RECEIPT-INVALID` — receipt does not validate against the
    payload projection + engine digest.
  - `SMALT-ENGINE-DRIFT` — envelope engine digest ≠ `:persistent_term`.
  - `SMALT-RECEIPT-REPLAY-FAIL` — deep-mode replay inside graphlaw_ex
    failed.
  - `SMALT-OVERSIZE` — request body above the 16 MiB packet-4 cap.
  - `SMALT-METHOD-NA` — never emitted (GET-class is ungated, not refused);
    listed so testers do not invent it.

Zero state mutation on refusal: the gate is a pure decision function over
(envelope, body, persistent_term) until the commit point. The commit point
is the first and only state write, and it happens only after check #3
(+#4, +#5 in deep mode) passes. A refusal path that writes anything other
than the sanctioned OCEL refusal frame is a court failure.

## 6. Courts (names + falsifiers)

- **`smalt_commit_gate_court`** — proves: a valid envelope commits; every
  §5 code fires on its trigger; state is byte-identical after each
  refusal. Falsifier: strip the receipt → `SMALT-NO-RECEIPT`; flip one
  byte of `receipt_blake3` → `SMALT-RECEIPT-INVALID`; digest B vs
  `:persistent_term` A → `SMALT-ENGINE-DRIFT`; mutate the body
  post-admission → `SMALT-PAYLOAD-MISMATCH`.
- **`smalt_wire_parity_court`** — proves: identical ARM/Cosmos deployment
  through both paths (direct graphlaw_ex vs Smalt-routed) yields
  byte-identical receipt hashes. Falsifier: run the same mutation corpus
  through both paths; any differing hash → FAIL. A direct mutation
  lacking a valid R_receipt → typed refusal, never a silent pass.
- **`smalt_zero_mutation_court`** — proves: refusal paths write nothing
  (except the sanctioned OCEL refusal frame). Falsifier: inventory state
  deltas across all refusal triggers; a non-OCEL delta flips the court.
- **`smalt_oversize_court`** — proves: >16 MiB bodies refuse
  `SMALT-OVERSIZE` with no engine allocation. Falsifier: a 16 MiB + 1
  body must produce the typed refusal with `max_request_bytes/0` honored.
- **`smalt_gate_antivacuity`** — proves the gate can refuse at all (a
  gate that cannot exit 1 is vacuous). Falsifier: invert the gate to
  always-commit → the parity court must FAIL.

The codes in §5 are canonical; the courts reuse them verbatim.

## 7. Open questions

- **Q1 — digest refresh**: push from graphlaw_ex on engine load, or Smalt
  poll? Push is race-free but couples process topology; poll needs a
  bounded backoff that is NOT a sleep-as-coordination (no-sleep doctrine:
  bounded poll on an observable).
- **Q2 — deep mode**: is the optional in-graphlaw_ex receipt replay (§3
  check #5) always-on, off by default, or per-route? Always-on doubles the
  per-request engine cost; off-by-default weakens the parity claim for the
  deep check only.
- **Q3 — binary payloads**: does any Smalt statement class need non-UTF-8
  `payload_bytes`? If yes, the `:invalid_encoding` check in `frame/1`
  (abi.ex:78) needs a binary-framed exception class on the graphlaw_ex side.
- **Q4 — OCEL refusal frame**: is the sanctioned refusal-time write (§3.6)
  in scope for GL28-1's telemetry handler or a separate Smalt-local journal?
  Determines whether GL28-2 depends on GL28-1 landing first.
- **Q5 — ARM/Cosmos deployment topology**: the parity court (§6) says
  "identical deployment through both paths"; pin the exact fixture (same
  graphlaw_ex release, same engine artifact digest, same corpus) so the
  court is replayable in CI, not just locally.
- **Q6 — refusal vocabulary consolidation** (GL28-11): do the Smalt `SMALT-*`
  codes live in graphlaw_ex's `%Refusal{}` or a Smalt-native struct? A
  second vocabulary is the O(N²) drift risk; prefer one struct, one code
  registry, Smalt codes as a disjoint code range.

## Acceptance (for the eventual build ticket)

- `mix test test/smalt_commit_gate_test.exs` (real router + real
  `:persistent_term`, Chicago style — no mocks) — green.
- `mix test test/smalt_wire_parity_test.exs` — identical corpus through
  both paths, byte-identical `receipt_blake3` set, green.
- Mutation court: inverting the gate to always-commit makes the parity
  court FAIL (anti-vacuity). A gate that cannot refuse is the failure.
