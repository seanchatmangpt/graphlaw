# E3 — executable conformance rung (design record)

Condensed from the E3 spec lane. Maps to SQLRDF-16 row E3; depends on E2.
Existing infrastructure to build on: SQL_STATEMENTS/SQL_EXTENSIONS tables,
`w3c_conformance.rs::w3c_conformance_receipt_summary` (per-dialect
ConformanceReceipt = the exact rung-report template), mutation_court --check,
docs_release_claims.rs (doc-claim pinning precedent).

## Rung: tests/sql_rdf_conformance.rs (aggregator, not new assertions)
`RungReceipt { clause_id, section, kind, status: Pass|Refused|KnownDeviation|ProseOnly, witness }`
- §3.2: iterate SQL_STATEMENTS → one receipt row per class (33 run-or-typed-
  REFUSAL-02 + explicit 33/2 partition assertion, currently implicit).
- §3.3/§4: compile-time refusal courts (existing) + needed negatives (non-list
  ordered clause, double sql:select, malformed cons cell, all 14 operator
  tokens positively, table-driven from sql-ast.ttl so shape and test cannot
  drift).
- §5: per-code receipt rows carrying the triggering graph's BLAKE3; REFUSAL-04
  sql-rdf court needed (currently only wasm-side cousins); REFUSAL-01 must bind
  SHACL report details.
- §6.1: no-test-doubles grep court + mutation-court --check folded into the
  receipt + the §3.2 partition IS rule 3.
- §6.2: per-extension rows with `KnownDeviation` for the disclosed raw-SQL
  materialization; catalog status + certified result in one row.
- §7.1: exists (SQLRDF-04/09 courts).

## Key invariants (from the w3c_conformance precedent)
- Clause manifest as one `const` table (~60 rows §3–§7); emitted receipt covers
  every ID exactly once.
- **ProseOnly is a status, not an omission** — §1/§2/§8 rows appear with
  status ProseOnly; "not courtable" is machine-checked exhaustiveness.
- Rungs: conformance level = highest rung where all rows are Pass;
  KnownDeviation disclosed (mirrors known_diverged).

## Doc generation (one-way: tests → doc)
Receipt table emits Markdown via env gate (CONFORMANCE_REPORT=path) →
`docs/reference-generated/sql-rdf-conformance.md` (GENERATED header) → a
docs_release_claims court pins the claimed rung in hand-written docs against
the last emitted receipt.

## Legal posture (skeptic-hardened)
Courts assert REPO behavior; the rec's W3C status claims stay disclaimed
document text and are never converted into asserted facts. The vacuous-subset
risk ("green = the document" by construction of extraction) is countered by
the manifest-exhaustiveness check + ProseOnly enumeration.

## Gap list (all [N])
1. Explicit 33/2 partition assertion. 2. REFUSAL-01 SHACL-details payload
binding. 3. REFUSAL-04 sql-rdf court. 4. Negative-shape courts (non-list
clause, double select, malformed cons cell). 5. Positive 14-token operator
compile, table-driven. 6. No-doubles grep court + mutation-court in-rung.
7. Per-extension receipt rows + KnownDeviation. 8. The aggregator + manifest +
emitter + doc-claim pin.
