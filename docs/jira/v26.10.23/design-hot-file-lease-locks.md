# Design: Mechanical Hot-File Lease Locks (ERRC-Reduce Fix 2)

- **Status**: DRAFT (design spec; no code in this lane)
- **Window**: `docs/jira/v26.10.23/`
- **Related**: `src/abi/lease.rs` (L67 lease-verification draft, UNWIRED);
  `docs/jira/v26.10.23/_LANES.md`; `docs/refusals.md` (append-only refusal
  register)
- **Ticket class**: ERRC-Reduce — replace a prose invariant with a mechanism
  ("disjoint leases" was stated, never enforced)

## Context

The lane fan-out invariant — every lane's file lease is disjoint from every
other lane's — has been prose only. Multiple lanes write the same files
seconds apart. Measured write collision surface (write counts per file, by
ten-minute bucket, one recent wave):

| File | Writes | Buckets (10-min) | Meaning |
|---|---|---|---|
| `src/sql_ast.rs` | 124 | 29 | 124 writes spread over 29 buckets — sustained multi-lane contention |
| `duckdb_op` (receipts/transition log names) | 50 | — | transition receipts for duckdb_op edits seconds apart |
| `src/duckdb_eval.rs` | 10 | — | see note |
| `RECEIPTS.md` | 38 | — | append traffic from many lanes, seconds apart |

Corrected reading of the raw counts: `src/duckdb_eval.rs` shows 47 file
events in the same window (mostly `Read`/`Edit` events, 10 of them writes);
`duckdb_op`'s 50 events are transition receipts (evidence of edits landing),
not hook events. The contention signal stands regardless of the exact
denominator: `sql_ast.rs` alone had 124 writes across 29 ten-minute buckets
from multiple lanes, and `RECEIPTS.md` saw 38 appends seconds apart.

The result was recurring edit collisions: a lane's Edit lands on a file
another lane was mid-edit on, producing diffs attributable to neither lane's
declared scope. Root cause is structural, not behavioral: **the invariant
was never a mechanism**. `src/abi/lease.rs` exists as a doc-fenced draft and
is UNWIRED — no hook, no dispatch path, no refusal path consults it. So the
system had one enforcement point (the human/agent reading the dispatch
prompt) and zero mechanical ones.

## Mechanism

A dispatch-time, per-lane JSON lease declaration, checked by a PreToolUse
hook on every Edit/Write.

### Lease declaration schema

Each dispatch prompt carries a lease block; the lane persists it to
`docs/jira/v26.10.23/leases/<lane-id>.json`:

```json
{
  "lane_id": "L07-design",
  "repo": "/Users/sac/coordinated/clawdbot",
  "issued_unix": 1791748800,
  "expires_unix": 1791756000,
  "released": false,
  "leases": [
    { "path": "src/abi.rs", "modes": ["edit", "write"] },
    { "path": "docs/jira/v26.10.23/*", "md_only": true }
  ]
}
```

Entries are literal paths or globs.

### Dispatch flow

1. Coordinator mints the lease JSON at dispatch (lane id, globs, expiry =
   dispatch time + default TTL 2h).
2. Lane's first tool call writes the lease file (this write is always
   allowed — a lane may always write its own lease file).
3. Every Edit/Write hook invocation loads **all** lease files, drops
   non-LIVE lanes, and refuses `LITHO-REFUSED(lease)` when the target path
   matches another LIVE lane's declared lease and does not match the
   current lane's lease.

### Liveness

A lane is LIVE iff:
- its lease file exists, is unreleased (`released: false` final report, the coordinator sets `released: true` at integration, and
- its lane process is alive (checked via the dispatch PID record; a dead PID → lease is STALE, not blocking.

Both conditions checked; `released: false` alone is not liveness (a crashed
lane must not hold the tree hostage).

## The initial hot set (4 files)

The first deployment scopes enforcement to a fixed 4-file set to keep the
refusal surface small and auditable:

1. `src/sql_ast.rs`
2. `src/duckdb_eval.rs`
3. `src/abi.rs`
4. `docs/jira/v26.10.23/RECEIPTS.md` — see append-only special case below

## RECEIPTS.md append-only special case

The subject is `docs/jira/v26.10.23/RECEIPTS.md`. Concurrent appends were mostly legal — the lock must not
forbid multi-lane appends, only rewrites. Classification rule:

- **Allowed (append)**: the pre-image and post-image satisfy
  `post.startswith(pre)` — the edit only added text at the end (Edit with
  `old_string` being the file's final line(s), Write with append-semantic
  content extending the file, or `Bash` `>>` is out of scope (hook sees
  Bash tool input, not the redirection target — explicitly not covered by
  the Edit/Write hook).

Bash redirections are declared out of scope: the hook sees the Bash tool's
`command` string, not the resolved redirection target, so a lane could
`echo >> RECEIPTS.md` past the lock. That is accepted residual risk for the
initial deployment, declared here rather than hidden.

Bash is not idle either: transition receipts
(`receipts/transition-*.json`) are produced by Bash-side Edits; those are
already receipted per transition, and the lock's guarantee on RECEIPTS.md
is scoped to the Edit/Write hook surface.

## Expiration / steal semantics

- **Expiry**: `expires_unix` in the lease file; a hook consulting a lease
  past expiry treats the lane as not-LIVE (its lease grants nothing and
  blocks nothing).
- **Stale takeover ("steal")**: taking over another lane's hot-file lease
  requires a **receipt** — the taking lane writes
  `receipts/transition-<ts>-lease-steal-<lane-id>.json` naming the stolen
  lane, the file, and the evidence of staleness (dead PID or past expiry)
  before its next write to the contested file. The hook enforces this:
  a write to a file in a STALE lane's lease is refused unless a steal
  receipt naming that (lane, file) predates the write attempt.
- **Re-issue**: on steal, the stolen lane's lease is marked
  `released: true` by the stealing lane, and the file's lease moves to the
  writer's lease file. No human admission needed mid-wave; the receipt is
  the admission.

## Anti-vacuity self-test

The gate must be able to fail. Two hooks run in the self-test; at least one
must refuse:

1. **Refuse case**: a synthetic PreToolUse payload with lane A live-holding
   `src/sql_ast.rs`, lane B attempting Write to `src hot-file` in lane A's
   cross-lane Write to a hot file → hook must exit 2 (refuse), typed
   `LITHO-REFUSED(lease)`.
2. **Allow case**: lane B appending to `RECEIPTS.md` while lane A holds it
   (append-only case) → hook must allow (exit 0).

If case 1 ever allows (gate accepts the violation), the self-test exits 1
as `VACUOUS GATE`. If case 2 refuses a legal append, the self-test exits 1
as `OVER-REFUSAL`. Both failure modes are typed.

## Conformance target

- Hook: PreToolUse on Edit|Write, loads `docs/jira/v26.10.23/leases/*.json`,
  path-match + liveness + append-only classification, typed refusal codes
  (`LITHO-REFUSED(lease)`, `LEASE-STEAL-RECEIPT-REQUIRED`).
- Court: `bash scripts/mutation_court.sh` rung plus a dedicated
  self-test hook pair as described above; a gate that cannot exit 1 is the
  failure.
- Court rung: cheapest first — self-test hook pair directly (no cargo).
- This is a docs lane; implementation is a separate lane's work order.

## Acceptance

1. Lease schema + hook design approved (this doc) → implementation lane.
2. Self-test: refuse case refuses (exit 2), allow case allows (exit 0),
   vacuity check fails the gate if either arm is inverted.
3. A wave runs with the lock live and records: zero cross-lane write
   collisions on the 4-file hot set, and zero over-refusals of legal
   RECEIPTS.md appends.
4. `docs/refusals.md` gains the two typed codes (append-only; no renumber).

## Report Sections

- **Measured contention** — the write-count table above; the "disjoint
  leases" invariant was prose, `src/abi/lease.rs` UNWIRED.
- **Mechanism** — dispatch-time JSON lease, PreToolUse hook, liveness =
  alive-PID AND unreleased, scope = 4-file hot set first.
- **Append-only case** — `post.startswith(pre)` classification; Bash
  redirection out of scope, declared residual risk.
- **Steal semantics** — stale takeover requires a receipt naming
  (lane, file, staleness evidence); receipt is the admission.
- **Anti-vacuity** — two-hook self-test; refuse arm must refuse (exit 2),
  allow arm must allow; inverted gate = `VACUOUS GATE` exit 1;
  over-refusal = `OVER-REFUSAL` exit 1.

## See Also

- `docs/refusals.md` (typed refusal register, append-only)
- `docs/jira/v26.10.23/_LANES.md`
- `src/abi/lease.rs` (L67 signed-lease draft, separate mechanism —
  request-scoped, not file-scoped)
