# Refusals

Every refusal is typed and leaves no child state. In Rust these are `LawError` variants; over the
JSON ABI (feature `abi`, `src/abi.rs`) the response carries the refusal plus a `details` object
whose `code` field is the stable discriminator. The envelope is described in the
[ABI reference](abi-reference.md).

Refusals raised by the engines themselves (bad syntax, unknown query form) carry no `details`;
only the codes below do. Decoders must treat `details` as optional and unknown codes as open (see
the [capability registry](capability-registry.md)).

## Codes (`details.code`)

| code | Rust | `details` fields | What to do |
|---|---|---|---|
| `PlanRefused` | `LawError::PlanRefused` | `index`, `action`, `missing`, `violated_absent` | Action `index` (or `<goal>` at one past the last) lacks the N-Quads lines in `missing`, or has the forbidden lines in `violated_absent` (`pre_not` / `goal_not`, PDDL `(not p)`) present. Fix the plan or the start state. |
| `ReceiptRequired` | `LawError::ReceiptRequired` | `step` | Run the step, then `receipt::record` its receipt into the state before the gate. |
| `ReceiptRefused` | `LawError::ReceiptRefused` | `step`, `reason` (`unattested`, `bad_signature`, `untrusted_key`) | Raised by `require-signed-receipt`. `unattested`: the receipt has no attestation; `bad_signature`: the signature does not verify; `untrusted_key`: the signing key is not in the step's `trusted_keys`. |
| `LeaseRefused` | `LawError::LeaseRefused` | `reason`, `lease_id`, `step` | See lease reasons below. |
| `UnverifiedLeaseRefused` | ABI request check | none | A `law` request carried an unsigned `lease` without `"unverified_lease": true`. Supply `signed_lease` + `trusted_keys`, or opt in explicitly (with `now_unix`). The envelope `kind` is `Unsupported`. |
| `NotAdmitted` | `LawError::NotAdmitted` | `violations[]`: `focus`, `path`, `component`, `message`, `severity` | Fix the data or the shapes; each entry is one SHACL result. The envelope `kind` is `EngineRejected`, dialect `Turtle`, engine `PurRdf`. |
| `ResourceLimit` | `RefusalKind::ResourceLimit` | `limit`, `observed`, `max` (N3 limits always carry `max`) | Shrink the request. Limits: `request_bytes`, `json_depth`, `atoms_per_field`, `plan_actions`, `policy_entries` (constants `abi::MAX_*`), plus the N3 reasoner limits `n3_iterations`, `n3_match_steps`, `n3_term_bytes`, `n3_total_bytes`, `n3_derived_facts` (constants `law::N3_*`). |
| `Refused` | `LawError::Refused` | `kind` | Upstream engine or routing refusal raised inside a `law` step; `kind` is a `RefusalKind` name (below). |
| `PolicyRefused` | `policy::PolicyRefused` | `policy_kind`, `state`, `action` | See policy kinds below. |
| `DuckDb` | `RefusalKind::DuckDb` | `message` | The embedded DuckDB substrate (ABI op `duckdb`, feature `duckdb`, native-only; `src/duckdb_op.rs`) failed or the request misused it — unknown/expired cursor, bad view mount, unsupported param, engine error. Never traps. Chunk limits ride `ResourceLimit` with `limit` = `duckdb_chunk_rows` / `duckdb_chunk_bytes` / `duckdb_views`. |
| `AstError` | `sql_ast::AstError` | `reason` (Display of the variant) | Refusal during SQL AST validation before query dispatch to `frozen-duckdb`. Variants in src: `InvalidIdentifier` (identifiers must match regex `^[a-zA-Z_][a-zA-Z0-9_]*$`), `MissingProperty`, `RecursionLimitExceeded`, `MalformedList`, `UnsupportedTerm`, `MalformedNode`, `ShaclViolation`. The stable `REFUSAL-01..04` codes for this surface appear as `[REFUSAL-nn]` markers inside the `reason` string (see below); no WHERE clause is required — bounded-WHERE enforcement does not exist on this path. |

## SQL AST refusals (`REFUSAL-01..04`)

Stable-coded refusal rows for the sql-rdf surface. The codes are stable as string
markers: each refusal embeds a `[REFUSAL-nn]` marker in its message/reason and the
trigger-to-code mapping is fixed by the tables below. Scope note (honest limits): the
marker prefixes are typed constants in `src` — `sql_ast::REFUSAL_01_MALFORMED_AST`,
`sql_ast::REFUSAL_02_UNIMPLEMENTED`, `sql_ast::REFUSAL_03_SCOPE_INTEGRITY`
(`src/sql_ast.rs`), and `tier::REFUSAL_04_CAPABILITY_REFUSAL` (`src/tier.rs`) — but the
codes still have no capability-registry rows, and the register flip itself has no
`receipts/` entry this wave. Those are tracked integration items, not mechanical
guarantees yet.

| code | name | trigger | carrier | recovery |
|---|---|---|---|---|
| `REFUSAL-01` | `MalformedAstRefusal` | The graph AST fails strict closed-world SHACL shape validation before admission: missing or non-integer `sql:statementIndex`, empty/missing `sql:from`, missing projection list, malformed/unclosed RDF list, unexpected terminal node, recursion-limit breach, more than one expression discriminator per node, a token outside the §4 operator enum (`Operator::from_token`), or a non-`sql:Scope` individual. | `sql_ast::AstError::ShaclViolation` (operator tokens, scope individuals, and shape failures — message carries `[REFUSAL-01]`), plus `MissingProperty`, `MalformedNode`, `MalformedList`, `RecursionLimitExceeded`, `InvalidIdentifier` for the non-shape malformed cases. Unrecognized *function* IRIs use `UnsupportedTerm` with no `[REFUSAL-nn]` marker today (unclassified — pinned as a known gap). | Fix the AST to satisfy the shapes (`ontologies/sql-ast.ttl`); re-admit. |
| `REFUSAL-02` | `UnimplementedStatementRefusal` | A dialect statement kind parsed from the grammar (`ggen/sql-dialect/sql_statement_kinds.rs`) is recognized but has no evaluation path in this DuckDB build — currently `ConnectStatement`, `DisconnectStatement`. | `sql_ast::AstError::UnsupportedTerm` carrying the decorated string `compile refusal: http://graphlaw.org/sql#<Kind> [REFUSAL-02] UnimplementedStatementRefusal: not implemented by this DuckDB build` | Rewrite the query using admitted statement kinds, or implement the evaluation path behind a receipted ticket. |
| `REFUSAL-03` | `ScopeIntegrityRefusal` | Scope discrimination boundary violated (rec §1.3): an aggregate in a `WHERE`/`JOIN`/`GROUP BY` position, a `sql:ProcessScope` column in the projection list, or a `sql:ViewScope` column inside a predicate. | `sql_ast::AstError::ShaclViolation` (message carries `[REFUSAL-03]`) | Move the aggregate into `SELECT`/`HAVING` as shapes permit; keep `sql:ProcessScope` columns out of the projection and `sql:ViewScope` columns out of predicates. |
| `REFUSAL-04` | `CapabilityRefusal` | Builtin SQL eval was given a reader that is not document-backed. The wasm module still admits and compiles SQL-in-RDF through `compile_turtle`, the same path as the other dialects. | `rules::builtins`: `BuiltinError::AstError` whose text starts with the typed constant `tier::REFUSAL_04_CAPABILITY_REFUSAL` (`REFUSAL-04`) | Supply the admitted Turtle document (`query_document_turtle`) and compile with `compile_turtle`. |

## Conformance note: `Σ_PQC` is UNMET (SQLRDF-10)

The second formula of `docs/reference/sql-rdf-rec.md` §1.4,

`ΔS_admitted + Φ_lease ≡ R_receipt (mod Σ_PQC)`,

names a post-quantum signature scheme `Σ_PQC` that this crate does not implement. The
conformance clause is **UNMET** as of v26.10.22. The only signature scheme in the tree is
Ed25519 via `src/attest.rs` (`sign_bytes`, `sign_receipt`), courted by
`tests/trust_attest.rs`. A signature over a SQL query digest (e.g. the §1.4
`H_query = BLAKE3(Canon(G_AST) ‖ P_params)`) *can* ride that existing Ed25519 path;
nothing signs `H_query` today. No post-quantum scheme, parameter set, or modulus is
defined by this register, and no new signature scheme is admitted without a receipted
ticket.

## `Refused {kind}`

`kind` names the upstream `RefusalKind` that a `law` step surfaced. Refusals from top-level ops
(not inside `law`) put the same kind in `error.kind` and have no `details`.

## `RefusalKind`

`NotSemanticContent` (empty, HTML or HTTP error body), `Ambiguous` (add a dialect hint),
`EngineRejected` (the owning engine rejected the document), `Unsupported` (valid but unusable
here, for example blank nodes in plan atoms, or a malformed ABI request), `ResourceLimit`,
`DuckDb` (the native-only embedded DuckDB substrate failed; op `duckdb`, feature `duckdb`).

## Lease reasons (`LeaseRefused.reason`)

| reason | Meaning |
|---|---|
| `expired` | Strict expiry: `now >= expires_unix`. |
| `out_of_scope` | The step name is not in the lease `scope`. |
| `ceiling` | The lease ceiling is below the step's required ceiling. |
| `bad_signature` | The lease attestation does not verify. Signed leases only. |
| `untrusted_key` | The attestation key is not in `trusted_keys`. Signed leases only. |
| `clock_skew` | `issued_unix` leads the verifier's clock by more than `max_skew_secs` (default 60). Signed leases only. |

Required ceilings per step are in the [ABI reference](abi-reference.md#13-law).

## Receipt reasons (`ReceiptRefused.reason`)

`unattested`, `bad_signature`, `untrusted_key`.

## Policy kinds (`policy_kind`)

`MissingEntry` (reachable non-goal state has no policy entry), `InventedOutcome` (outcome not in
the problem's transitions), `BadMass` (outcome probabilities do not sum correctly), `DeadEnd`,
`Malformed`. `state` and `action` name where the policy failed.

## Aggregate boundary

Law-kernel materialization (N3/Datalog closure) refuses SPARQL 1.1 aggregates by design: a law
kernel is one executable law and must be total and deterministic over the closure, so
`GROUP_CONCAT`/`SAMPLE` in a query handed to the law path is a refusal, not a result.
GraphLaw's `Step` enum (`src/law.rs`) offers no aggregate-capable step — the kernel surface is
`AdmitShacl`, `DeriveN3`, `Hooks`, `Plan`, receipt gates, and entailment closures only — so an
aggregate simply has no admitted path through a `law` step. (Historically the graphlaw
law-materialization engine, vendored in ggen as `crates/graphlaw` until its retirement
in ggen commit `8e92df31f`, 2026-10-10, surfaced this as a typed
`SPARQL query planning refused (unsupported construct)` error from its `build_for_aggregate`
planner; that crate is gone and the boundary now lives in the step surface itself.)

Aggregate projections (`GROUP_CONCAT`, `SAMPLE`, non-`COUNT` aggregation over groups) belong to
the generation/projection layer instead, where the full SPARQL 1.1 engine (oxigraph in the
ggen sync path) has standing and determinism is enforced by projection replay, not by the kernel.

Rule: if a gate or query needs an aggregate, run it in the projection layer and admit the
projected artifact; do not push it through a `law` step expecting a kernel answer.

## See Also

- [ABI reference](abi-reference.md)
- [Capability registry](capability-registry.md)
