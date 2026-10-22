# Receipt-surface digest pin (design record — P28-SIL §4 circumvention witness)

Condensed from the pin-design lane. Upgrades circumvention detection from
NOT PRESENT (legal-only) to a non-silent technical witness.

## Surface (minimal, deliberate)
src/duckdb_eval.rs (hash_param_buffer :93, admitted_statement_hash :125,
canonical_ast_graph_bytes :87, receipt path :172-186/:384, verify mirror
:560) + src/sql_ast.rs (canonical_graph_bytes :382, REFUSAL_01/02/03 consts
:15-17 — laundered refusal paths are a circumvention vector). OUT: law
receipt chain/store (already chain-covered), registry.rs (self-referential),
abi.rs.

## Pin form (recommendation: registry-style canonical document, not raw
concatenation)
`receipt_surface_document()` = canonical JSON {schema, files: [{path, blake3,
bytes} ×2]}; `receipt_surface_blake3()` = blake3:<hex> over it — reuses the
audited canonical/blake3_of law, self-describing (per-file digests localize
mutations), no self-reference hazard, 71-char form matches existing
surface_blake3 invariants (registry.rs:1938-1939). File digests via
include_bytes! at compile time (simplest, target-invariant; registry module
already rebuilds on any tree change).

## Where it lives
1. ARTIFACTS.blake3 new row `receipt-surface blake3:<hex> - capability-registry.json`.
2. Registry document key `receipt_surface_blake3` in base_document + TTL
   emission + schema $defs — NOTE: enters registry_blake3's preimage →
   registry files regenerate in the same commit.
3. ABI capabilities response (compare-against-published-pin, same status as
   surface_blake3 — drift reported, never masked).

## Re-pin workflow (gws-16 pattern)
Trigger: any commit touching the two files. Regenerate registry + re-pin the
ARTIFACTS row in the same commit; header carries pin-source version.

## Enforcement teeth
Extend tests/registry_artifacts.rs: pinned row == live function; drift court:
mutate a per-file digest in the document → computed digest differs (negative
control). Falsifier scenario: altering admitted_statement_hash changes
duckdb_eval.rs bytes → three independent mismatch signals fire (registry
court, pinned key, ARTIFACTS CI). Evasion requires an explicit diffable
re-pin — silent laundering becomes a visible reviewable diff. The witness
does not PREVENT circumvention; it makes it non-silent.
