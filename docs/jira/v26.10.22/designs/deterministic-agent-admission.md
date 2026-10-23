# Deterministic admission for agent labor (operator brief, 2026-10-22)

This file records the operator brief that follows the file-split plan in
[core-team-decomposition.md](core-team-decomposition.md). Nothing in this
file is an encoded rule, an ontology, or a court. The one measured instance
of the three-way partition is [code-partition-census.md](code-partition-census.md):
gate FAIL, 44 unadmitted projection leaks, handwritten/derived ratio 1.1425
at the N21 run. That run still names the deleted `*_split` folders.

## What the quality bar is

The brief treats graphlaw as a closed admission machine rather than an
application crate.

- A refusal is a stable code with a trigger and a recovery
  ([`docs/refusals.md`](../../../refusals.md)). A missing lease or a broken
  seal stops before the engine call.
- Receipt bytes and the registry document are content-addressed. A module
  move is not accepted merely because it compiles. The split that landed
  preserved field order and left the receipt formula alone. It did not
  re-run the registry artifact court.
- Pure folding and POSIX writes are different files on the query ledger
  (`src/query_ledger/batch.rs`, `src/query_ledger/storage.rs`). The brief's
  stronger claim — `batch.rs` imports no `std::fs` — was not audited in this
  filing.
- Compile units were cut so a builtin edit does not rebuild the whole
  reasoner parent. `reasoner.rs` is still 4248 lines. The
  `clippy::too_many_arguments` allow is still on that file.
- Courts are Chicago readbacks and mutation flips
  (`tests/chicago_validation.rs`, `scripts/mutation_court.sh`), plus the
  partition court. The brief's named test `registry_hash_invariance` is not
  in `tests/`.

The admission identity the brief writes as

```text
ΔS_admitted + Φ_lease ≡ R_receipt  (mod Σ_PQC)
```

is the same family as the sql-rdf receipt law already in
[`docs/reference/sql-rdf-rec.md`](../../../reference/sql-rdf-rec.md) §1.4
and the P28-SIL epoch note in [P28-SIL-specs.md](P28-SIL-specs.md). Σ_PQC
remains the SQLRDF-10 conformance note. It is not a post-quantum signature
on agent patches.

## A code edit as an admitted transition

The brief's reading, filed as design:

| Term | Meaning in the brief | What exists today |
|---|---|---|
| `ΔS_admitted` | The AST or file delta | Ordinary git diff. No RDF patch graph. |
| `Φ_lease` | File scope, byte ceiling, forbidden imports, time budget | `src/abi/lease.rs` is a comment. No agent lease type is admitted. |
| `R_receipt` | Receipt that the invariants held | Ed25519 receipts and the Blake3 receipt chain for admitted SQL and law transitions. Not issued for a Rust edit. |
| `Σ_PQC` | Seal on the release surface | Registry surface digest and the wasm artifact pin. Not a seal over agent commits. |

A refused edit, in the brief, returns the violating node, the rule, and a
counter-example. The kernel already does that for SQL and dialect input
(`RefusalCode`, `RefusalKind`). It does not do that for Rust source.

## Five rule families (not encoded)

The brief lists five checks an admission gate would run on a Rust change.
None of them are N3 rules in this tree. `ontologies/` has no Rust-review
vocabulary and no `jira:` prefix.

| Family | Brief's reject condition | Nearest thing on disk |
|---|---|---|
| Wire layout | `repr(C)` / packed field order or size changes across revisions | Receipt frame move kept the body. No `rustc` layout diff court. |
| Public surface | Registry Blake3 changes without a signed lease upgrade | `graphlaw-registry --check` and `tests/registry_artifacts.rs`. No lease upgrade object. |
| Purity | A module classed pure imports `std::fs`, a runtime, or a socket | Partition court G4 on delta slabs. `src/evidence_ring.rs` carries a recorded io justification. |
| Monomorphization | Generic depth or cycle past a fixed bound | Not measured. The split reduced compile units. It did not add a bound. |
| Exhaustive refusal | Every error arm is a registered code, not `unwrap` or an open `_` | Refusal register and the SQL compiler's `refusal_arm`s. Not an AST lint over the whole workspace. |

The illustrative refusal the brief gives —

```text
Refusal: VIOLATION_IMPORT_BOUNDARY
Subject: crates/graphlaw-eyeron/src/reasoner/unify.rs
Forbidden edge: std::fs::read_to_string
Rule: urn:rules:pure-kernel
```

— names a file that does not exist. Unification is still in `reasoner.rs`.
`unify.rs` is remainder item 1 in the decomposition note.

## Semantic work graph (not in the tree)

The brief replaces a prose ticket with an RDF task: assignee, target module,
allowed directories, max file count, disallowed imports, a public-symbol
digest that must hold, a complexity cap, a lease duration, and the rule
sets that apply.

Composable packs named in the brief, none of which are files here:

- `pure-kernel` — no effectful imports, no unbounded generics
- `abi-frozen` — public signatures and enum discriminants hold
- `test-coverage-court` — a new AST variant requires a Chicago court node

A task closes only when the gate emits a receipt. A refusal names the node
and the rule and spends the same lease. That loop is the brief. The window
index and these design notes are still markdown.

The nearest executable cousin is the code-partition court
(`scripts/partition_court.sh`, `courts/code-partition/manifest.json`):
every owned `.rs` file is classified once (G1), axiomatic digests are
pinned (G2), a projection file without a generator or an admitted-leak
entry fails the gate (G3), a delta slab that touches io without a
justification fails (G4), and the handwritten/derived ratio is printed and
not turned into a pass (G5).

## Three partitions

The brief's rule for what a person or an agent may write:

```text
S_handwritten  ≪  S_derived
```

| Partition | Who writes it | Brief's test | This repo |
|---|---|---|---|
| Axiomatic kernel | Handwritten once, then pinned | Encoding ground truth, an algebraic step, or a property a court can exhaust | 14 files pinned in `courts/code-partition/axioms.blake3`. G2 failed on the N21 run because four pins drifted (`duckdb_eval`, `law`, `plan`, and the old `src/sql_split/ast.rs` path). |
| Compiled projection | A generator, never an edit by hand | Dispatch, refusal tables, serializers, sniff tables, visitors, when the ontology already has the rows | G3: 44 files are handwritten projections with no generator header and no admitted-leak entry. Generator-backed count on that run was 3. |
| Constrained delta slab | An agent, inside one signature and one court | Pure leaf: inputs, a refusal code, a complexity cap, a named test | 20 files classed delta on that run. G4 passed, with one io justification. |

The pre-determination flow the brief asks for, applied to a file before
anyone edits it:

1. If the file is an axiom, it is already pinned. A change is a G2
   re-pin receipt, not a silent edit.
2. If the rows already live in an ontology or a static table, the Rust is
   a projection. Hand-editing it adds an unadmitted leak. The leak list in
   the census is the codegen backlog (ABI dispatch, CLI bridges, emitters,
   refusal tables).
3. Otherwise it is a leaf. The brief's example shape is one function,
   `fn eval(args: &[Term], ctx: &mut SolveContext) -> Result<bool, RefusalCode>`,
   with a named court. `SolveContext` is not on that signature today.

Measured ratio at N21: S_handwritten 19999, S_derived 17505, ratio 1.1425.
The inversion the brief wants has not happened. The ratio is informational.
The court exits 1 while any unadmitted leak remains.

## What this would change about agent work

Architects would write axioms and rule packs. Generators would emit
routers, refusal enums, and serializers. An agent would receive one leaf
signature and one court, and a refusal would name the triple that failed.

That is the brief's claim for why the rules can be composed quickly: the
acceptance criteria move out of the prompt and into the gate. None of the
rule packs, the task ontology, or the agent receipt are implemented. The
implemented piece is the partition court, and it is red.
