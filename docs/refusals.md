# Refusals

Every refusal is typed and leaves no child state. In Rust these are `LawError` variants; over the
JSON ABI (feature `abi`, `src/abi.rs`) the response carries the refusal plus a `details` object
whose `code` field is the stable discriminator. The envelope is described in the
[ABI reference](abi-reference.md).

Refusals raised by the engines themselves (bad syntax, unknown query form) carry no `details`;
only the codes below do. Decoders must treat `details` as optional and unknown codes as open (see
the [capability registry](capability-registry.md)).

## Message ergonomics (lane L85, 2026-10-22)

Every code in this register was triggered FOR REAL and its exact user-visible
message captured (probe: `cargo run --example refusal_probe`, 12/12
exit-class MATCH, exit 0 as of 2026-10-22 (lane R10 re-run; the count tracks
the probe's own case list and must be re-read at use time, not recalled);
 fix-forward note 2026-10-22, lane M6, superseded 2026-10-22 lane N11:
the probe was renamed from `examples/l85_refusal_probe.rs` to
`examples/refusal_probe.rs` (lane L96) and is present on disk again — the
captured-message claim stands as of its original receipt, and the probe source
is re-runnable at the new path). Re-audit 2026-10-22, lane T4 (post-switchover drift check): every file anchor
in this register still resolves — `src/sql_ast/refusals.rs` (constants at
lines 9-11; `RefusalCode = 36/37` at 112/115; `refusal_arm::JOIN_TYPE_IRI = 37`
at 59), `src/sql_ast.rs` (`pub use refusals::*` at 24), `src/tier.rs`
(`REFUSAL_04_CAPABILITY_REFUSAL` at 25), `src/rules/builtins.rs` (literal
`REFUSAL-04` marker at 89), `src/duckdb_eval.rs` (`NoCanonicalGraph` Display
arm at 102-104), `src/attest.rs` (`AttestError`, `#[non_exhaustive]` at 38).
Probe re-run: 12/12 exit-class MATCH, 0 MISMATCH, exit 0 (re-observed
2026-10-22, lane R10; count tracks the probe's case list — re-read at use
time). Zero anchors shifted; no
corrections required. Receipt: `docs/jira/v26.10.22/designs/t4-refusal-reaudit.md`.
The symptom → message → cause → fix mapping built from those captured
messages lives in the [refusal playbook](refusal-playbook.md); it also grades
each message on WHAT / WHERE / WHAT-TO-DO and lists Display strings a src lane
should improve (playbook findings F-1..F-9). Code values in this register are
untouched by that pass.

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
| `DuckDb` | `RefusalKind::DuckDb` | `message` | The DuckDB substrate (ABI op `duckdb`, `src/duckdb_op.rs`) failed or the request misused it — unknown/expired cursor, bad view mount, unsupported param, engine error. Never traps. Chunk limits ride `ResourceLimit` with `limit` = `duckdb_chunk_rows` / `duckdb_chunk_bytes` / `duckdb_views`. Absence from the hashed 14-op list is not this refusal. Promoted 2026-10-23 (in-module execution, wasmtime 48 + new W3C EH archive): the op now dispatches inside the wasm unit on every target, and `RefusalKind::DuckDb` fires in-module too — a wasm caller no longer gets `Unsupported` / `unknown op`. Receipt: pending DFLSS-GL integration receipt (2026-10-23 wave). |
| `AstError` | `sql_ast::AstError` | `reason` (Display of the variant) | Refusal during SQL AST validation before query dispatch to `frozen-duckdb`. Post SQLRDF-18
(anchor correction 2026-10-22, lane V-L11) `AstError` is a pure-data struct `{code:
RefusalCode, ctx: RefusalContext}` (`src/sql_ast/refusals.rs`), not an enum; the names below
are constructor functions, and Display renders from a const code→template table. Codes in src:
`InvalidIdentifier` (identifiers must match regex `^[a-zA-Z_][a-zA-Z0-9_]*$`), `MissingProperty`, `RecursionLimitExceeded`, `MalformedList`, `UnsupportedTerm`, `MalformedNode`, `ShaclViolation`. The stable `REFUSAL-01..04` codes for this surface appear as `[REFUSAL-nn]` markers inside the `reason` string (see below); no WHERE clause is required — bounded-WHERE enforcement does not exist on this path. |

## SQL AST refusals (`REFUSAL-01..05`)

Stable-coded refusal rows for the sql-rdf surface. The codes are stable as string
markers: each refusal embeds a `[REFUSAL-nn]` marker in its message/reason and the
trigger-to-code mapping is fixed by the tables below. Scope note (honest limits): the
marker prefixes are typed constants in `src` — `sql_ast::REFUSAL_01_MALFORMED_AST`,
`sql_ast::REFUSAL_02_UNIMPLEMENTED`, `sql_ast::REFUSAL_03_SCOPE_INTEGRITY`
(anchor correction 2026-10-22, lane V-L11: the SQLRDF-18 switchover moved these
constants from `src/sql_ast.rs` to `src/sql_ast/refusals.rs`; the
`sql_ast::REFUSAL_*` paths still resolve via `pub use refusals::*`), and
`tier::REFUSAL_04_CAPABILITY_REFUSAL` (`src/tier.rs`) — but the
codes still have no capability-registry rows, and the register flip itself has no
`receipts/` entry this wave. Those are tracked integration items, not mechanical
guarantees yet.

| code | name | trigger | carrier | recovery |
|---|---|---|---|---|
| `REFUSAL-01` | `MalformedAstRefusal` | The graph AST fails strict closed-world SHACL shape validation before admission: missing or non-integer `sql:statementIndex`, empty/missing `sql:from`, missing projection list, malformed/unclosed RDF list, unexpected terminal node, recursion-limit breach, more than one expression discriminator per node, a token outside the §4 operator enum (`Operator::from_token`), or a non-`sql:Scope` individual. | `sql_ast::AstError::ShaclViolation` (operator tokens, scope individuals, and shape failures — message carries `[REFUSAL-01]`), plus `MissingProperty`, `MalformedNode`, `MalformedList`, `RecursionLimitExceeded`, `InvalidIdentifier` for the non-shape malformed cases. Unrecognized *function* IRIs use `UnsupportedTerm` with no `[REFUSAL-nn]` marker today (unclassified — pinned as a known gap). | Fix the AST to satisfy the shapes (`ontologies/sql-ast.ttl`); re-admit. |
| `REFUSAL-02` | `UnimplementedStatementRefusal` | A dialect statement kind parsed from the grammar (`ggen/sql-dialect/sql_statement_kinds.rs`) is recognized but has no evaluation path in this DuckDB build — currently `ConnectStatement`, `DisconnectStatement`. | `sql_ast::AstError::UnsupportedTerm` carrying the decorated string `compile refusal: http://graphlaw.org/sql#<Kind> [REFUSAL-02] UnimplementedStatementRefusal: not implemented by this DuckDB build` | Rewrite the query using admitted statement kinds, or implement the evaluation path behind a receipted ticket. |
| `REFUSAL-03` | `ScopeIntegrityRefusal` | Scope discrimination boundary violated (rec §1.3): an aggregate in a `WHERE`/`JOIN`/`GROUP BY` position, a `sql:ProcessScope` column in the projection list, or a `sql:ViewScope` column inside a predicate — including `HAVING` (`ViewScope` at `src/sql_ast.rs:284-295`, arm `VIEW_SCOPE_IN_PREDICATE = 5`), an explicit scope annotation on an aggregate argument beneath a predicate-rooted aggregate (arm `SCOPE_IN_PRED_AGG_ARG = 6`, `src/sql_ast.rs:304-311` — laundering marker: the argument position is process-only, so the annotation can only re-scope the aggregate's output past the gate; recovery: drop the scope annotation on the aggregate argument), or a MERGE `ON` condition referencing no target-qualified column (`MERGE_ON_NO_TARGET_COLUMN = 7`, `src/sql_ast.rs:987-993` — a constant-only ON like `$1 = $2` matches every target row and makes WHEN MATCHED fan-out unbounded; recovery: qualify the ON condition to at least one target column, e.g. `t.id = s.id`). | `sql_ast::AstError::ShaclViolation` (message carries `[REFUSAL-03]`); the appended arms carry `code: RefusalCode::ScopeIntegrity` with `shape_id` = the `refusal_arm` constant. | Move the aggregate into `SELECT`/`HAVING` as shapes permit; keep `sql:ProcessScope` columns out of the projection and `sql:ViewScope` columns out of predicate/`GROUP BY`/`HAVING` positions; drop scope annotations on aggregate arguments under predicates; qualify MERGE `ON` to a target column. |
| `REFUSAL-04` | `CapabilityRefusal` | Builtin SQL eval was given a reader that is not document-backed. The wasm module still admits and compiles SQL-in-RDF through `compile_turtle`, the same path as the other dialects. | `rules::builtins`: `BuiltinError::AstError` whose text starts with the marker `REFUSAL-04` (value of the typed constant `tier::REFUSAL_04_CAPABILITY_REFUSAL`; the current call site `src/rules/builtins.rs` uses the literal string — same text, verified 2026-10-22 lane V-L11) | Supply the admitted Turtle document (`query_document_turtle`) and compile with `compile_turtle`. |
| `REFUSAL-05` | `UnboundedMutationRefusal` | UPDATE/DELETE statement node with neither `sql:whereColumn` nor `sql:whereValue` — refused at compile, zero engine contact. (Appended 2026-10-23, append-only per convention — never renumbered.) Receipt: pending DFLSS-GL integration receipt (2026-10-23 wave). | `sql_ast::AstError` (`src/sql_ast/refusals.rs`), raised before any engine call | Add a WHERE clause (`sql:whereColumn`/`sql:whereValue`) bounding the mutation. |

### Appended internal discriminants (v26.10.22, lane L4) — appended, never renumbered

Typed refusal codes appended to the MalformedAst (REFUSAL-01) family: the
discriminants are `sql_ast::RefusalCode` arms appended at `= 36` / `= 37`
(the arm-namespace constants append at `refusal_arm::JOIN_TYPE_IRI = 37`).
Messages carry the `[REFUSAL-01] MalformedAstRefusal:` marker, the offending
lexical form, and the AST node.

| code | name | trigger | carrier | recovery |
|---|---|---|---|---|
| `LITERAL_NOT_INT` (`RefusalCode = 36`) | MalformedAst family (REFUSAL-01) | A `sql:literalValue` typed `xsd:integer` whose lexical form does not parse as an i64 (e.g. `"not-a-number"^^xsd:integer`). The value is never silently stringified or coerced to 0. | `sql_ast::AstError` with `code: RefusalCode::LiteralNotInt` (`= 36` — the literal-typed-code discriminator; correction 2026-10-22, lane V-L11: the rendered message carries the `[REFUSAL-01]` marker, the offending lexical, and the node, not the `LITERAL_NOT_INT` token). | Fix the lexical form to a valid integer, or retype the literal as `xsd:string`. |
| `LITERAL_NOT_FLOAT` (`RefusalCode = 37`) | MalformedAst family (REFUSAL-01) | A `sql:literalValue` typed `xsd:double` whose lexical form does not parse as an f64 (e.g. `"1.2.3 garbage"^^xsd:double`). Never coerced to 0.0. | `sql_ast::AstError` with `code: RefusalCode::LiteralNotFloat` (same correction as above: the message carries the `[REFUSAL-01]` marker + lexical, not a `LITERAL_NOT_FLOAT` token). | Fix the lexical form to a valid double, or retype the literal. |
| `JOIN_TYPE_IRI` (`refusal_arm = 37`, REFUSAL-01 arm) | MalformedAst family (REFUSAL-01) | A `sql:joinType` IRI that is neither `sql:InnerJoin` nor `sql:LeftJoin` (e.g. `sql#FullJoin`). An unknown join-type IRI refuses typed; it never silently downgrades to Inner. A *missing* `sql:joinType` predicate defaults to `Inner` (not a refusal — pinned by court). | `sql_ast::AstError` with `code: RefusalCode::MalformedAst`, `shape_id: refusal_arm::JOIN_TYPE_IRI`; message names the offending IRI. | Use `sql:InnerJoin` or `sql:LeftJoin`. |

### Appended document-sequence discriminants (v26.10.23, G5) — appended, never renumbered

Typed refusal codes appended to the `sql_ast::RefusalCode` arm namespace at
`= 39` / `= 40` / `= 41` (`src/sql_ast/refusals.rs`, `refusal_arm` constants).
Receipt: pending DFLSS-GL integration receipt (2026-10-23 wave).

| code | name | trigger | carrier | recovery |
|---|---|---|---|---|
| `MIXED_DOCUMENT_ROOTS` (`RefusalCode = 39`) | Document-root discipline (document:true surface) | Document carries >1 bare statement root or a bare+sequence mix — one root or one sequence, never both. | `sql_ast::AstError` with `code: RefusalCode::MalformedAst`, `shape_id: refusal_arm::MIXED_DOCUMENT_ROOTS` | Wrap multiple statements in one `rdf:List` sequence, or emit a single bare root. |
| `INDEX_SEQUENCE_DISORDER` (`RefusalCode = 40`) | Document-root discipline (document:true surface) | Sequence member `sql:statementIndex` values not strictly ascending along the `rdf:List` spine. | `sql_ast::AstError` with `code: RefusalCode::MalformedAst`, `shape_id: refusal_arm::INDEX_SEQUENCE_DISORDER` | Renumber `sql:statementIndex` so members ascend strictly along the spine. |
| `SEQUENCE_EMPTY` (`RefusalCode = 41`) | Document-root discipline (document:true surface) | Sequence with zero members (defense-in-depth below the `OrderedListShape` shape refusal). | `sql_ast::AstError` with `code: RefusalCode::MalformedAst`, `shape_id: refusal_arm::SEQUENCE_EMPTY` | Give the sequence at least one statement member. |

### Appended evaluator/attestation refusals (v26.10.22, lane L72) — appended, never renumbered

Kernel-level refusal surfaces not carried by the `RefusalCode`/`[REFUSAL-nn]`
markers above. Stable string codes, appended in place.

| code | name | trigger | carrier | recovery |
|---|---|---|---|---|
| `NO-CANONICAL-GRAPH` | `duckdb_eval::EvaluatorError::NoCanonicalGraph` | A hand-built `CompiledQuery` reaches eval with no canonical AST graph bytes or receipt anchor. The legacy SQL-string hash arm and the re-canonicalize fallback are deleted, not gated: the only admitted hashes are `BLAKE3(Canon(G_AST) \|\| P_params)` (rec §1.4, first formula). | `Err(EvaluatorError::NoCanonicalGraph(query_iri))`; Display: `no canonical graph bytes — hand-built CompiledQuery must use compile_turtle (query <iri>)` (the offending query IRI is interpolated; `src/duckdb_eval.rs` Display arm). | Obtain the query via `compile_turtle`, which pins `receipt_state`/`canon_graph_bytes` at compile time. |
| `ATTEST-REFUSED` | `attest::AttestError` | An Ed25519 attestation fails verification: `BadSignature` (signature does not verify or payload no longer matches the digest), `UnknownKey` (key_id not in the trusted set), `Malformed` (key/signature/document not well-formed). Court: `tests/trust_attest.rs`. | `Err(AttestError::...)`; `#[non_exhaustive]` — downstream matches need a wildcard. | Re-sign with a trusted key over the exact payload bytes; enrol the key before use. |

## Harness refusal surfaces

Typed refusals emitted by repo gates and harnesses. These are NOT kernel
`RefusalCode` discriminants and carry no `details.code` over the ABI — they
are deliberately kept out of the numeric register above.

| code | surface | trigger | exit | recovery |
|---|---|---|---|---|
| `WASI_PATCH_BUILD_REFUSED` | `build.rs` (wasi-patched-deps feature) | The `wasi-patched-deps` feature is enabled but the required WASI SDK patch input is missing. | cargo build script failure | Set up the WASI SDK patch input per `docs/WASM.md`, or drop the feature. |
| `MUTATION-COURT-DIRTY-TREE` | `scripts/mutation_court.sh` | Uncommitted work under `src/` while the court runs `--in-place` (false-red channel); bypassed only with `--force`. | 6 | Commit the work, or pass `--force` deliberately. |
| `GRAPHLAW-WASM-BUILD-FAILED` / `BLOCKED-LINK` | `tests/common/mod.rs` wasm harness | `cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm` fails with the typed frozen-duckdb-sys `build.rs` refusal (`WASI_SDK_PATH required`): without a wasi-sdk root on `WASI_SDK_PATH` (`bin/clang`, `share/cmake/wasi-sdk-p1.cmake`) the build script refuses before any compilation. (Historical, 2026-10: the pre-typed-refusal failure mode was the linker failing to resolve `-lduckdb`; that class is superseded by the typed build.rs refusal.) The harness also panics when cargo reports success and the artifact is missing. Update 2026-10-23 (EH wall closed by uniform new-EH rebuild, wasmtime 48): the link now succeeds with `WASI_SDK_PATH=<staging> CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS="-C target-feature=+exception-handling"`; wasmi remains unable to host — wasmtime 48 is the host. Receipt: pending DFLSS-GL integration receipt (2026-10-23 wave). | test panic (loud) | Recovery: `docs/WASM.md` (WASI_SDK_PATH setup; the engine cache is not a wasi-sdk). |
| `DOC-STALENESS-FINDINGS` | `scripts/doc_staleness_gate.sh` | Rules R1-R4 fire: unresolvable fenced commands, unregistered 64-hex digests, banned stale strings, or count/digest claims without a `receipts/` pointer. | 1 (findings), 0 (clean), 2 (startup failure, e.g. cd) | Fix each finding line; re-run the gate to exit 0. |
| `MANIFEST-REFUSED` / `DISK-REFUSED` | `docs/jira/v26.10.22/designs/rules-pruner-design.md` only | DESIGN-STAGE — not implemented on disk as of v26.10.22; listed here so the codes stay claimed and stable. | n/a | Implement behind a receipted ticket; flip this row with a receipt. |
| `GRAPHLAW-RUN-EH-REQUIRED` | `src/bin/graphlaw_run.rs` (`has_tag_section` pre-flight, `RunError::EhArtifact`) | Legacy-host scope: `graphlaw-run` (wasmi host) is given a wasm artifact carrying a wasm Tag(13) section (EH-class artifact). The pre-flight refuses before any parse attempt — wasmi cannot host these — with the message `EH-class artifact requires graphlaw-run-eh`. Witnessed exit 96 against the 46 MB EH artifact (2026-10-23 wave). | 96 | Run the artifact under `graphlaw-run-eh` (wasmtime 48 host). |
| `GRAPHLAW-RUN-EH` (exit trio 95/96/97) | `src/bin/graphlaw_run_eh.rs` (`RunError` mapping) | The `runner-eh` host surface (wasmtime 48): 95 — bytes are not a valid wasm module (`WasmInvalid`); 96 — module failed to instantiate/link/export the graphlaw ABI, including refusal of a non-wasi import (`WasmInstantiation`); 97 — guest ABI call or linear-memory access faulted (`AbiFault`). Each failure prints one typed one-line stderr diagnostic. Cross-ref: the EH-class pre-flight row above (`GRAPHLAW-RUN-EH-REQUIRED`). | 95 / 96 / 97 | Fix the artifact (95), the imports/exports vs the graphlaw ABI (96), or the guest fault site (97). |
| document-sequence wire surface (`BatchReceipt` / `BatchMemberFailed`) | `src/duckdb_op.rs` document execution | mode `exec` with `document:true` answers the `BatchReceipt` response shape (per-member `rows_emitted`); a failing member mid-sequence returns the typed `BatchMemberFailed` details `{code, member, partial_receipt}` naming the failing member and the receipt for the prefix that did run. Receipt: pending DFLSS-GL integration receipt (2026-10-23 wave). | ABI envelope | Fix the failing member's statement; replay resumes from the named member. |
| session-surface refusals (`DuckDb` / `ResourceLimit`) | `src/duckdb_op.rs` session registry | DuckDB session surface refusals, all typed: `unknown or expired session \`<name>\`` (`DuckDb`, session get after expiry/drop), duplicate session name on open (`DuckDb`), and the 16-session cap via `check_limit("duckdb_sessions", …, MAX_SESSIONS)` (`ResourceLimit`, `limit` = `duckdb_sessions`). Session state is process-local and crash-only, like cursor state. | ABI envelope | Use a live session name; open under a fresh name; close a session to free a slot. |

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
`TierMismatch` (the document belongs to the other semantic tier, R04: e.g. N3 logic given to an
RDF parser — see `src/tier.rs`),
`DuckDb` (the DuckDB substrate failed; op `duckdb` — dispatched inside the wasm unit on every target as of the 2026-10-23 promotion; receipt: pending DFLSS-GL integration receipt (2026-10-23 wave)).
Source of truth: `src/dialect/refusal.rs` `RefusalKind` (7 arms).

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
