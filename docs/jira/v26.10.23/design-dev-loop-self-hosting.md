# Design: Dev-Loop Self-Hosting — the Process-Mining Endgame

Milestone: v26.10.23 · Lane: no-code/docs · Status: DRAFT
Owner file: this document only. No code execution in this lane.

## Report Sections

- [Context: the repo emits but never admits](#context-the-repo-emits-but-never-admits)
- [Ontology sketch: dev-loop.ttl](#ontology-sketch-dev-loopttl)
- [The gate: wave-close as admitted transition](#the-gate-wave-close-as-admitted-transition)
- [Machinery inventory: what already exists](#machinery-inventory-what-already-exists)
- [Phased rollout](#phased-rollout)
- [Risks](#risks)
- [Acceptance](#acceptance)
- [See Also](#see-also)

## Context: the repo emits but never admits

graphlaw already receipts its own development. Every file-tool transition mints a
transition receipt into `receipts/` (e.g.
`receipts/transition-2026-10-24T013941Z-Edit-4ae88f78.json`), a `CHAIN.blake3` fold
tracks the receipt stream, and `src/receipt_chain.rs` implements the fold. But the
loop is open at the admission step. Three measured facts (2026-10-24, on disk):

1. **1,510 / 1,515 transition receipts are `UNSIGNED-NO-KEY`.** The receipts
   exist; the identity field is empty. Under `R=receipt(A)` a receipt without
   identity is not a receipt — it is telemetry. The 5 signed artifacts are
   session outputs, not dev-loop receipts.
2. **A tamper-detected file was left in-tree.** Detection fired (digest mismatch
   against the recorded chain) and no state change followed: the court found the
   defect and the world was unchanged. Detection without admission is a vacuous
   gate.
3. **The chain is not serialized into `receipts/`.** `src/receipt_chain.rs` folds
   OCEL frames in memory only; root `CLAUDE.md` marks inter-receipt chaining
   "pending decision — in-memory only, not yet serialized into `receipts/`".
   `CHAIN.blake3` is a flat head file; no per-receipt prev-hash linkage exists on
   disk, so truncation and reordering are undetectable by construction.

Together: the repo is a receipt *emitter*, not a receipt *admitter*. The endgame
is to close the loop — make the coordinator's integration commit an ADMITTED
transition under an executable law the repo itself hosts.

## Ontology sketch: dev-loop.ttl

New canonical graph at `ontologies/dev-loop.ttl` (hand-authored source, not a
projection). The Turtle below is a design sketch; the authored file must be valid
Turtle and canonicalize via `purrdf::try_canonicalize` (canon form is the
admission form — the sql-rdf receipt hash is `BLAKE3(Canon(G_AST) || P_params)`;
dev-loop admission uses the same fold).

### Classes

```turtle
devloop:Lane a owl:Class ;
    rdfs:comment "One dispatch lane; subject of its TransitionEvents and WaveCloses." .

devloop:TransitionEvent a owl:Class ;
    rdfs:comment "One file-tool actuation (Edit/Write/Bash) on the canonical checkout." .

devloop:WaveClose a owl:Class ;
    rdfs:comment "Coordinator integration transition; SHACL-gated state change." .
```

### TransitionEvent properties

```turtle
devloop:tool            xsd:string ;   # Edit | Write | Bash
devloop:target          xsd:string ;   # absolute path acted on
devloop:artifact_blake3 xsd:string ;   # BLAKE3 of post-state (or of the receipt payload)
devloop:signed_by       devloop:Lane ; # lane identity (lane-<8 hex>)
devloop:prev_hash       xsd:string ;   # fold of receipt N-1 (Phase 2)
devloop:status          xsd:string .   # ADMITTED | REFUSED(<CLASS>) — typed, append-only
```

### WaveClose shape

WaveClose is a SHACL-gated state change over the wave's TransitionEvent closure.
The node shape requires, conjunctively:

- `(a)` every TransitionEvent in the window carries `devloop:signed_by` matching
  `^lane-[0-9a-f]{8}$` and a verifying Ed25519 signature against the pinned lane
  pubkey map;
- `(b)` the serialized chain over the window folds to the committed
  `receipts/CHAIN.blake3` head with zero mismatch;
- `(c)` every destructive transition in the window (deletes/wipes: staging
  `_split/` paths, lane build roots, receipt files) carries a receipt naming the
  target and a typed reason — an unreceipted wipe is
  `REFUSED(UNRECEIPTED_DESTRUCTIVE)`, not a warning.

## The gate: wave-close as admitted transition

The coordinator's integration commit is lawful only as an ADMITTED transition:

- `receipts/` is the event log (OCEL-shaped transition receipts).
- `src/receipt_chain.rs` is the fold (serialized in Phase 2).
- `src/law.rs` is the gate: `graphlaw-run` evaluates `ontologies/dev-loop.ttl`
  plus the wave receipt window as params → `ADMIT` or `REFUSED(class)`.

Enforcement discipline: the gate must be able to exit 1. A wave-close that fails
(a), (b), or (c) flips the wave to BLOCKED — never log-and-continue. That is the
direct lesson from fact 2 above: the tamper incident is replayed as a court, not
a log line.

## Machinery inventory: what already exists

~90% of the admission machinery is built. Do not rebuild. This table is a claim
until re-read against the code; the Phase 1 first action is exactly that pointer
audit (Current-behavior pointers verified on disk before any work is filed).

| Piece | Locus | Role in dev-loop gate |
|---|---|---|
| Law kernel | `src/law.rs` | evaluates G_ast + params → ADMIT / REFUSED(class) |
| Chain fold | `src/receipt_chain.rs` | folds the receipt stream (in-memory today; on-disk in Phase 2) |
| Runner | `graphlaw-run` (host bin; wasm/ module) | runs the admission law over dev-loop.ttl + wave params |
| Verifier | `graphlaw-verify` (`./target/debug/graphlaw-verify`) | batch-verify Ed25519 signatures on transition receipts |
| Signing primitives | `src/attest.rs` | Ed25519 sign/verify already exist |
| SHACL machinery | `ontologies/` shapes (e.g. `sql-ast.ttl`) | node-shape pattern for WaveClose |
| Anti-vacuity harness | `scripts/mutation_court.sh` | gains dev-loop mutation arms (Phase 3) |

## Phased rollout

### Phase 1: sign the receipts (the hook gains a key)

- The transition-receipt minting hook gains a lane key: one Ed25519 keypair per
  lane at dispatch. Private half lives outside the repo tree
  (`.devloop/private/$LANE`, gitignored); public halves are pinned in a new
  lane→pubkey pin file (a `courts/code-partition/`-adjacent pin, lane-map class —
  not the axioms pin).
- Receipt schema gains `signed_by` + `sig` fields. New receipts can no longer be
  `UNSIGNED-NO-KEY`. The 1,510 historical unsigned rows are NOT backfilled —
  they are pre-admission telemetry, marked as such in the milestone INDEX.
- Exit criterion: 100% of new transition receipts verify under
  `graphlaw-verify` against the pinned pubkey map, real output embedded in the
  phase receipt.
- Falsifier (anti-vacuity): flip one byte of one new receipt's payload → the
  signature check MUST fail with a typed error and the wave-close court MUST
  exit 1. Detection must be followed by a state change (BLOCKED), or it is the
  tamper incident again.

### Phase 2: serialize the chain

- Per-receipt position-level linkage: receipt N carries `prev_hash` = fold(N-1)
  and `head` = BLAKE3(payload || prev_hash). This makes the stream
  position-sensitive, not set-sensitive — deletion, reordering, or truncation of
  any receipt breaks every subsequent link.
- `receipts/CHAIN.blake3` becomes the fold head over the serialized stream,
  recomputed at each wave close and committed at integration.
- Exit criterion: recompute over the real serialized stream matches the
  committed head (exit 0); deleting one middle receipt from a temp copy breaks
  the fold (falsifier flips, exit 1). The court prints measured fold time in ms
  — measured, not claimed.

### Phase 3: wave-close admission court

- New gate `scripts/dev_loop_gate.sh` (code lane): `graphlaw-run` over
  `ontologies/dev-loop.ttl` + the wave receipt window → `ADMIT` /
  `REFUSED(class)`. The coordinator's integration commit is permitted only on
  ADMIT.
- Courts: (a) batch signature verify over the window; (b) chain fold vs
  committed head; (c) destructive-transition coverage vs the diff.
- Anti-vacuity: `scripts/mutation_court.sh` gains three dev-loop arms — forged
  signature, truncated chain, unreceipted wipe. All three must REFUSE. A gate
  that accepts all three is `VACUOUS GATE`, exit 1.
- Bootstrap: the first gate run admits against the pre-gate chain head at T0
  (2026-10-24) as the one dated trust anchor; every later replay cites it.

## Risks

### Key management for lane keys

- Private keys never enter the repo tree or any receipt payload; lease root
  (`.devloop/private/`) is gitignored and swept at integration alongside lane
  build roots.
- Compromise model: a stolen lane key lets an attacker sign fake transitions.
  Mitigation is the committed chain head — a forged segment will not fold to the
  head committed at the prior integration, so forgery is detectable at the next
  wave close.
- Key loss = typed refusal `REFUSED(KEY_LOST)` for that lane's pending receipts.
  Rotation is a receipted transition: old pub revoked in the map, new pub
  pinned, revocation itself receipted.

### Hook latency

- Ed25519 signing is microseconds; per-transition cost is O(1) (sign + link to
  prev fold). Wave-close recompute is O(n) over the window (n in the hundreds →
  sub-second expected).
- The Phase 2 court must print the measured milliseconds, not claim them. If
  measurement contradicts, the typed fallback is batch-signing the fold head at
  wave close instead of per-receipt signatures — documented here, decided by
  measurement, never improvised; gates (a)/(c) reword accordingly, gate (c)
  semantics unchanged.

### Bootstrap circularity

- The first gate run has no prior admitted head. Resolution: the pre-gate chain
  head at T0 (2026-10-24) is the one explicit, dated trust anchor — stated once,
  cited by every later replay.
- Post-T0, the law protects itself: tampering with the serialized chain breaks
  the fold; tampering with `src/law.rs` is covered by the axiom pin files;
  `ontologies/dev-loop.ttl` is a hand-authored source (not a generated
  artifact), so it is court-covered, not registry-sync-covered.

## Acceptance

- Phase 1: `graphlaw-verify` over a real wave window with real output embedded in
  the phase receipt; byte-flip falsifier receipted with real output.
- Phase 2: fold-recompute court with real output; middle-receipt-deletion
  falsifier receipted.
- Phase 3: `scripts/dev_loop_gate.sh` prints `ADMIT` with measured ms, real
  output embedded; all three mutation arms REFUSE with real output. A gate that
  cannot exit 1 is the failure.
- Overarching: every phase receipt embeds REAL command output — a receipt with
  no embedded output is not a receipt.

## See Also

- `docs/jira/v26.10.23/ROADMAP.md`
- `docs/jira/v26.10.23/ADR-wasmi-retirement.md` (ADR pattern)
- `docs/refusals.md` (typed refusal register, append-only)
- `src/receipt_chain.rs` (fold — in-memory today, serialized in Phase 2)
- `src/law.rs` (admission kernel)
- `src/attest.rs` (Ed25519 primitives)
