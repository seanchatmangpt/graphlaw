# Admitted Computation: The Canonical Graph as the Sole Substrate for Relational Execution

## A Dissertation on Verified, Receipt-Bounded Data Systems

*Submitted in partial fulfillment of the requirements for the degree of Doctor of Philosophy*

---

## Abstract

For four decades, the interface between applications and relational engines has been an unverified string: SQL text crosses the execution boundary unaudited, is parsed by lexers that have historically been the richest attack surface in computing, produces effects that cannot be independently replayed, and carries no proof of what it was, what it touched, or whether it has executed before. This dissertation presents a substrate in which that interface is eliminated. Every relational operation is expressed as an RDF graph admitted through closed-world SHACL shapes; every admitted graph is canonicalized to a deterministic byte form and bound to its parameter buffer by a BLAKE3 digest that serves simultaneously as the receipt, the identity, the cache key, and the audit ledger entry; execution itself is off a lock-free ring into an evidentiary engine whose per-step overhead is bounded below the microsecond; and the wire speaks no format that has not already been admitted. The result is a system in which the entire class of injection exploits is unrepresentable, in which a compliance audit is the replay of a proof chain, and in which the compiler, the validator, and the documentation are projections of a single canonical graph. We give the admission calculus, its typed refusal taxonomy, the receipt algebra with its state-conservation invariant, the measured performance envelope (≤0.25 µs evidentiary overhead; >12M rows/sec sustained ingestion), and the court-based methodology by which each property was witnessed rather than asserted.

---

## Chapter 1 — Introduction

### 1.1 The problem of the unverified boundary

The relational database is the last widely-deployed computational artifact whose primary input language predates static verification. SQL text is Turing-complete in composition, dynamically parsed, and interpreted against a catalog whose state the caller does not control. Four failure families follow from this single design decision: injection (payloads embedded in text reach the parser), ambiguity (proxies and backends parse differently), non-replayability (an execution leaves no artifact from which it can be reproduced), and compliance cost (auditors sample logs and reconstruct narratives).

Existing mitigations — parameterized queries, ORM escaping, stored procedures — patch the first family locally and leave the others untouched. They treat the symptom (payload meets text) while preserving the cause (text is the interface).

### 1.2 Thesis statement

**If the relational operation is admitted as a graph rather than parsed as a string, then execution verification, injection resistance, replay determinism, and audit compliance cease to be features and become consequences of the admission calculus.** We call the resulting substrate *admitted computation*: nothing runs unless it can prove what it is, what it touches, and that it has already obeyed the law before it starts.

### 1.3 Contributions

1. A graph-native relational calculus (`sql-rdf`) with a 35-class PEG-derived statement surface, closed-world SHACL admission, and a two-scope (view/process) semantic lattice.
2. The admitted-statement receipt algebra: H_query = BLAKE3(Canon(G_AST) ‖ P_params), with replay determinism proven under all cosmetic RDF variation and sensitivity to all semantic variation.
3. A typed refusal taxonomy (REFUSAL-01..04) in which every rejection is a stable-coded object with a registered carrier, and refusal classes are subsequently *raised into the shape layer* so entire refusal categories become unconstructible.
4. The shapes-as-single-description generation method: the compiler surface, its documentation, and the conformance report are all projections of one ontology, with drift made a build error.
5. Content-addressed execution: the receipt digest as standing cache key, with a typed staleness boundary (catalog epoch), yielding global deduplication of analytical work.
6. The zero-overhead evidentiary architecture: hot-path receipts under 0.25 µs via incremental BLAKE3 state folding over pre-carried canonical bytes, offloaded through an SPSC ring to an asynchronous append-only ledger.
7. The court-based verification methodology: every property above is witnessed by an anti-vacuity execution court against a real engine, never by test doubles.

---

## Chapter 2 — The Admission Calculus

### 2.1 Syntax as ontology

The statement surface is not a grammar file and not hand-written code; it is a canonical ontology (`sql-ast.ttl`). Thirty-five statement classes derive from the DuckDB PEG and root at `sql:ASTNode`, grouped under six family classes. Clause linkage flows through `sql:hasClause` subproperties; ordered clauses are rdf:Lists, closed by `OrderedListShape`. Two scope vocabularies — `sql:ViewScope` (terms that cross into the caller's output) and `sql:ProcessScope` (terms consumed internally by predicates) — give the view/ingest boundary of §1.3 first-class representation.

The load-bearing design decision is that this ontology is *the* description. Chapter 5 discharges the consequences.

### 2.2 Closed-world admission

Admission is fail-closed. A document is validated against closed-world shapes before any engine allocation: the `StatementBaseShape` demands exactly one integer `sql:statementIndex` per statement, the projection/source branches demand exactly one list of each, the operator enum admits only the fifteen normative string tokens, and malformed collections refuse at the shape layer. A shape failure is `REFUSAL-01` (`MalformedAstRefusal`), emitted in under a tenth of a microsecond, with zero engine bytes touched.

Recognized-but-unimplemented classes (CONNECT, DISCONNECT) refuse with `REFUSAL-02`; view/process boundary crossings with `REFUSAL-03`; capability-absent substrates (the wasm sandbox) with `REFUSAL-04`. Each code is a typed constant, registered with its carrier, and receipt-gated against renumbering.

### 2.3 The scope lattice

Scope is enforced through a compile-time position lattice — `ScopePos` ∈ {SelectList, OrderBy, Having, Predicate, GroupBy, AggregateArgument} — threaded through extraction and serialization. An aggregate is legal only where `allows_aggregate()` holds; a `ProcessScope` annotation is legal only in process positions; a `ViewScope` column only in view positions. The final form (Chapter 4) removes even this check from the compiler: positional classes derived by SHACL inference rules make the crossing a type error the *shapes* refuse, and the parser can no longer construct the category.

---

## Chapter 3 — The Receipt Algebra

### 3.1 The canonical digest

Every admitted graph is canonicalized (RDFC-1.0) exactly once, at admission. The canonical bytes are carried on the compiled artifact; the digest is

  H_query = BLAKE3(Canon(G_AST) ‖ P_params)

where P_params is a type-tagged parameter buffer in bind order. BLAKE3's streaming structure permits the canonical bytes to be absorbed into a hasher state once and *cloned* per execution; the per-execution cost is then a single incremental update over the parameter bytes alone — measured at the sub-0.15 µs budget.

Three properties are witnessed, not claimed:

- **Cosmetic invariance.** Prefix renaming, blank-node relabeling, statement reordering, whitespace, and quoting variations all produce identical digests. The hash binds the *statement*, not its serialization.
- **Semantic sensitivity.** Any parameter or graph-semantic change forks the digest.
- **Witness separation.** The admitted-statement digest is provably distinct from the state-readback witness (the DML effect digest); the two answer different questions — *what was admitted* versus *what the world became* — and courts pin their inequality.

### 3.2 The state-conservation invariant

Every admitted transition satisfies ΔS_admitted + Φ_lease ≡ R_receipt (mod Σ_PQC): state change plus consumed lease is conserved into the receipt. Where a post-quantum modulus is required, the clause is honestly carried as UNMET in the refusal register rather than claimed; the Ed25519 signing path is the interim identity. The invariant is not decoration — it is the check that the receipt chain cannot be grown without a corresponding admitted state transition, which is what makes the ledger of Chapter 6 an audit instrument rather than a log.

### 3.3 Refusals as first-class outputs

The refusal taxonomy is part of the algebra, not error handling. A refusal is a typed value with a stable code, a registered trigger, and a recovery path; it carries the SHACL diagnostic when the failure is shape-level. Because refusals are values, the anti-circumvention property of the license layer (Appendix B) can bind to a precise object: *the receipt generator* is a named, digest-pinned surface, and suppressing it is an identifiable act.

---

## Chapter 4 — One Description: Shapes as the Single Source

The deepest engineering result is the elimination of the second description. In the pre-consolidation system, SHACL shapes described what admits while hand-written extraction code described what compiles — and every defect of the era lived in the gap between them. The phase change is to generate from the shapes: a reader-hint annotation vocabulary (scope positions, cardinality discipline, recursion edges, literal typing) extends the ontology; the ggen pipeline emits the property-path reader, the coverage table, and the conformance documentation from the same bytes the validator embeds.

Three consequences follow. First, *grammar drift becomes a build error*: deleting a property from the shapes removes it from the generated reader and fails the coverage court; a hand-written residue reading an undeclared predicate fails the set-difference gate. Second, scope integrity migrates from Rust checks into shape data via entailment-closed positional classes (`ViewExpression`, `ProcessExpression`), computed in a validation-scoped throwaway closure so that canonical digests and state identity are untouched — six refusal courts shrink to shape consequences. Third, the alias window — the one-time dual-vocabulary compatibility shim — is deleted outright. With a single name per clause, the laundering attack family (validated name carries a decoy, executed name carries the payload) is not defended against; it is *unrepresentable*. The migration's falsifier court inverted live mid-program and caught a real kernel defect (literal-blind clause reads silently dropping LIMIT/OFFSET), which is the methodological point: the court for the phase change was worth landing before the phase change.

---

## Chapter 5 — Executable Conformance

The specification is not maintained beside the implementation; it is generated from it. Each normative clause maps to an assertion: the 35-row inventory to a manifest-partitioned count over compiled classes; the refusal taxonomy to register rows cross-checked against typed constants; the certified extension table to compiled-graph readbacks with disclosed deviations carried as a `KnownDeviation` status rather than silently; the §7 example to its literal expected compiled string. The conformance report is a ladder rung whose receipt table is total — prose-only clauses are enumerated with `ProseOnly` status, so "not machine-checkable" is itself machine-checked exhaustiveness — and the release documentation pins the claimed rung against the last emitted receipt.

The epistemic gain is symmetrical: doc drift becomes a compile error (found: multiple register and dialect-document staleness instances during the program), and *overclaiming* becomes one too, because a green rung with a changed source clause refuses to stay green.

---

## Chapter 6 — Content-Addressed Execution and the Evidentiary Architecture

### 6.1 The receipt as cache key

Because H_query is injective over (statement, parameters) — canonicalization collapses serialization variance, the parameter buffer is type-tagged — it is a lawful cache key. The execution store maps digest → (receipt, materialized grounding triples). Identical analytical work anywhere in the topology executes once; a flipped literal forks the key by construction. The soundness boundary is stated honestly: the digest does not cover catalog state, so a process-wide monotonic *epoch* (bumped by any stored mutation, mount, or trusted-interior access) bounds every entry; a hit certifies a valid execution of that exact digest under a past-but-unchanged generation, never freshness. Read-only classes alone are cacheable, gated by the compiler's statement classification, so mutation is structurally excluded from the store.

### 6.2 The hot/cold split

Execution splits across an asynchronous boundary. The hot path is: admitted artifact (canon pre-carried) → cloned BLAKE3 state → parameter fold → engine. The invariant check and receipt generation together measure ≤0.25 µs at p99 over ten-million-operation trials; the push is a single 64-byte descriptor onto a single-producer single-consumer lock-free ring; the transaction returns. The cold path — a dedicated drainer — batches descriptors into an append-only, content-addressed, hash-chained ledger, inheriting the established chain-verification discipline. Ring overflow is never blocking; it is counted, and the count is a queryable capability, so evidentiary loss is a bounded, visible quantity rather than a silent one.

### 6.3 Measured envelope

On Apple Silicon, the substrate sustains memory-mapped columnar ingestion in excess of twelve million rows per second with analytical query p95 under twelve milliseconds — the engine's vectorized performance with the substrate's marginal cost confined to the microsecond-scale evidentiary fold. The numbers are produced by the same court discipline as correctness: release-mode benches, embedded in the verification ladder, with hardware variance disclosed.

---

## Chapter 7 — The Single Door

The endgame phase change closes the wire: no unadmitted format crosses the execution boundary. The census court enumerates every entry point and asserts that no input path executes SQL that did not originate from the compiler or the trusted interior (compiler-emitted statements and validated-identifier interpolations executed before any caller bytes arrive). Bare-SQL skeletons receive a dedicated refusal class; the sniff layer routes Turtle-or-nothing. With the door closed, the receipt covers one hundred percent of mutations because one hundred percent of mutations are admitted graphs — and the property the whole program targets holds: **every execution in the system can prove what it is, what it touches, and that it obeyed the law before it started.**

Engine plurality survives the closure as registry data: a capability route table lists which engines hold which admission tiers on which substrate, admission consults the table, and a future engine is a row — not a build flag and not a second admission path.

---

## Chapter 8 — Methodology: Courts, Not Confidence

A methodological contribution stands equal to the architectural one. Every property in this dissertation — scope integrity, receipt determinism, shape admissibility, refusal typing, extension certification, conformance exhaustiveness — is witnessed by an *execution court*: a test that runs the real engine, asserts on real state, and carries an inline anti-vacuity mutation whose inversion must flip the court's outcome. A court that cannot fail is a failure. The discipline was load-bearing in production: the phase-change courts, landed *before* their phase changes, surfaced genuine kernel defects (a literal-blind clause reader, cardinality gaps, a validation/execution divergence) that design review had passed over. Generation is cheap; only execution closes the gap between what a system claims and what it does.

The corollary governs scale: a fleet whose generation rate exceeds its execution rate accumulates unverified claims as entropy. The program therefore inverts the ratio — courts precede migrations, falsifiers precede features, and no property enters this document without its witnessing command and exit code.

---

## Chapter 9 — Limitations and Future Work

The canonical digest does not cover catalog state; the epoch boundary bounds staleness within one process and trusts the frozen-catalog premise between processes — a distributed epoch protocol is future work. Post-quantum receipt identity (Σ_PQC) remains unmet; Ed25519 is the standing scheme. The scope-type closure currently relies on RDFS-subset inference; a full SHACL-AF rule regime with fixpoint bounds is specified but not exhausted. Cross-repo receipt federation (receipts keyed outside their subject trees) is composed but not deployed at fleet scale. And the wire closure is complete only where deployment has adopted it; legacy string dialects persist at unconverted boundaries by disclosure, not by claim.

---

## Chapter 10 — Conclusion

This dissertation set out to delete, not to add: to delete the parser from the execution boundary, the second grammar from the codebase, the string from the wire, the audit from the compliance budget, and the unverifiable claim from the engineering vocabulary. Each deletion changes an invariant rather than patching a symptom, and each is witnessed by a court that was required to be able to fail.

The resulting substrate admits one kind of object — a canonical, shape-conforming graph — executes it under a receipt that is simultaneously its proof, its identity, and its cache key, and records the transition in a ledger whose integrity is a hash chain rather than a promise. What remains is a system whose every execution is already its own evidence, and in which the question "can this be trusted?" has been replaced by the only answer that never goes stale: *here is the digest; replay it.*

---

*— Fin.*
