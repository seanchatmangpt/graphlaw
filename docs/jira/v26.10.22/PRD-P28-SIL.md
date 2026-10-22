# Product Requirements Document (PRD): GraphLaw Substrate & P28-SIL Zero-Overhead Execution Architecture

- **Document Version:** 1.0.0
- **Filed:** 2026-10-22 (verbatim operator text; supersedes the earlier skeleton)
- **Target Release:** v26.10.22 Wave Successors (Moonshot Implementation)
- **Status:** Approved for Engineering & Integration
- **Scope:** Core Runtime Engine, `sql-rdf` AST Pipeline, P28-SIL Evidentiary Engine
- **Program ticket:** [SQLRDF-18.md](SQLRDF-18.md)

---

## 1. Executive Summary & Core Objective

The GraphLaw Substrate integrates semantic relational processing (`sql-rdf`), compile-time artifact generation (`ggen`), and self-attesting legal compliance under the **P28 Sovereign Invariant License (P28-SIL)**.

The primary architectural mandate is **zero hot-path latency impact**:

1. Evidentiary cryptographic receipts ($\mathbf{R}_{\text{receipt}}$) and audit invariants must add **$\le 0.25\,\mu\text{s}$** of overhead per step.
2. The core hot path must sustain **$>12,000,000\text{ rows/sec}$** analytical ingestion and **$p95 < 12\text{ ms}$** analytical query latency on bare-metal/Apple Silicon memory-mapped buffers.
3. The P28-SIL governance mechanism operates via compile-time constraints, incremental hash digests, and asynchronous write-ring buffers. It establishes a **$50,000,000 annual revenue safe harbor**, routing large enterprises directly to fair commercial regularization by making technical circumvention cognitively and legally intractable.

---

## 2. Invariant Performance Budgets (Hot-Path vs. Cold-Path)

Execution is divided by an asynchronous hardware ring buffer into two distinct operational paths:

```
[HOT PATH: Direct Vectorized Execution]
Incoming Admitted Graph ──► In-Memory Pre-Computed H_query ──► Native Vectorized Engine (DuckDB) ──► Return Results
                                         │
                                         ▼ (Zero-Copy Pointer Emit)
                       [Ring Buffer / SPSC Lock-Free Queue]
                                         │
                                         ▼ (Cold Path: Thread Offload)
                          [COLD PATH: Evidentiary Engine]
                           Async Audit Ledger Batched Disk I/O
                           ASC 450 Receipt Aggregation Ledger
```

| Pipeline Segment | Execution Path | Target Budget | Enforcement Mechanism |
| --- | --- | --- | --- |
| **AST Parse & Grammar Check** | Build/Compile Time | $0.00\,\mu\text{s}$ (Runtime) | Pre-compiled via `ggen`; no runtime Turtle/text lexer |
| **SHACL Shape Gate** | Hot Path Admission | $\le 0.25\,\mu\text{s}$ | Pre-compiled static match trees; closed-world lookups |
| **$\mathbf{H}_{\text{query}}$ Receipt Calculation** | Hot Path | $\le 0.15\,\mu\text{s}$ | Incremental BLAKE3 fold over pre-canonicalized AST bytes |
| **Engine Execution** | Hot Path | $<12.0\text{ ms}$ ($p95$) | Native vectorized columnar processing (`frozen-duckdb 2.0.0`) |
| **Ingestion Pipeline** | Hot Path | $>12\text{M}$ rows/sec | Memory-mapped zero-copy arrow/column buffers |
| **Audit Log Persistence** | Cold Path (Async) | Bounded background | SPSC lock-free ring buffer offloaded to dedicated disk actor |

---

## 3. Detailed Technical Requirements

### 3.1. Zero-Copy Incremental Receipting ($\mathbf{R}_{\text{receipt}}$)

* **Requirement 3.1.1 (Incremental Canonical Fold):** The runtime shall NOT re-canonicalize the entire RDF AST graph at execution time. The RDFC-1.0 canonical bytes of the AST template (`Canon(G_AST)`) must be computed at compile time or initial shape admission and stored as a static byte slice.
* **Requirement 3.1.2 (Hash Generation):** At runtime, computation of $\mathbf{H}_{\text{query}} = \operatorname{BLAKE3}\left(\operatorname{Canon}(G_{\text{AST}}) \mathbin{\Vert} \mathbf{P}_{\text{params}}\right)$ must execute strictly as a single incremental update over the extracted literal buffer $\mathbf{P}_{\text{params}}$.

> **Harmonization note (2026-10-22, SQLRDF-18):** LICENSE-P28-SIL.md §1.5 files the
> full three-term form, $\mathbf{H}_{\text{query}} = \operatorname{BLAKE3}(
> \operatorname{Canon}(G_{\text{AST}}) \mathbin{\Vert} \mathbf{P}_{\text{params}}
> \mathbin{\Vert} \oint_{\partial \Omega} \omega_{\text{context}})$. The third term
> is the **deployment epoch** — an immutable, deployment-scoped `u64` assigned once
> at substrate initialization, not ambient hardware state. Requirement 3.1.2's
> two-term formula is the $\omega_{\text{context}} = 0$ (single-deployment) special
> case; the term is **reserved** and zero-valued in single-deployment mode, pinned
> by the epoch court in `tests/p28_determinism_court.rs`. No hot-path change.
* **Requirement 3.1.3 (State Conservation Invariant):** Every admitted transition must satisfy:

$$\Delta S_{\text{admitted}} + \mathbf{\Phi}_{\text{lease}} \equiv \mathbf{R}_{\text{receipt}} \pmod{\mathbf{\Sigma}_{\text{PQC}}}$$

If verification fails or parameters exceed $\mathbf{\Phi}_{\text{lease}}$, the engine emits an in-memory typed refusal (`REFUSAL-01` through `REFUSAL-04`) in $\le 0.10\,\mu\text{s}$ without initiating engine allocation.

### 3.2. Asynchronous Evidentiary Audit Offload

* **Requirement 3.2.1 (Lock-Free Write Ring):** After computing $\mathbf{H}_{\text{query}}$, the hot path pushes a 64-byte receipt descriptor pointer to a Single-Producer Single-Consumer (SPSC) lock-free ring buffer. The transaction completes and returns execution results immediately.
* **Requirement 3.2.2 (Decoupled Persistence):** A dedicated background BEAM actor/OS thread drains the ring buffer, batching receipts into an append-only, content-addressed ledger. Failure or backpressure on this ring buffer must never block native vectorized query evaluation.

### 3.3. Scope Discrimination Without Reflection

* **Requirement 3.3.1 (Static Scope Position):** The compiler must enforce `sql:ViewScope` vs. `sql:ProcessScope` via compile-time enum tagging (`ScopePos`).
* **Requirement 3.3.2 (No Runtime Introspection):** Relational columns must never be queried or checked against scope policies via dynamic string reflection on the hot path. All projections are verified during the SHACL admission phase.

---

## 4. Legal & Governance Specification (P28-SIL Integration)

### 4.1. The Commercial Revenue Threshold ($50M Safe Harbor)

* **Under $50,000,000 Annual Consolidated Revenue:** Free, unencumbered, royalty-free execution grant. All ASC 450 accounting triggers, commercial balance claims, and audit disclosures remain inactive.
* **At or Exceeding $50,000,000 Annual Consolidated Revenue:** The deployment transitions to Commercial Exploitation Status. The runtime's local cryptographic ledger begins recording valid operational receipts for commercial title verification.

### 4.2. Anti-Circumvention Ratchet & Spoliation Attachment

* **Trigger Event (Circumvention Dialogue):** The drafting, transmission, or oral holding of any meeting, RFC, ticket, or communication designed to suppress, decouple, or alter the execution receipt generator ($\mathbf{R}_{\text{receipt}}$) dissolves the Covenant Not to Sue *ab initio*.
* **Automatic Evidentiary Hold:** The occurrence of any circumvention dialogue automatically constitutes formal anticipation of litigation under FRCP 37(e), triggering an immediate common-law and contractual duty to preserve all associated communications, commit histories, and technical notes.
* **Adverse Inference Stipulation:** In any proceeding where the deployer fails to produce pristine chat, code, or email records following circumvention dialogue, the tribunal shall presume bad faith and assess commercial usage at maximum theoretical hardware capacity.

### 4.3. Harmonic Regularization (The Fair-Payment Off-Ramp)

* **Bilateral Settlement Channel:** A commercial entity exceeding $50M in revenue may clear all accrued accounting liabilities at any time prior to formal dispute notice by executing an Enterprise Commercial Execution Lease.
* **Cryptographic Title Certificate:** Upon remitting the agreed commercial settlement, the licensor issues a signed state certificate ($\mathbf{\Sigma}_{\text{regularized}}$), extinguishing all balance sheet contingencies under FASB ASC Topic 450 and Sarbanes-Oxley § 404 without penalties, audits, or operational disruption.

---

## 5. Verification & Acceptance Criteria (Chicago School Doctrine)

| Test ID | Test Scope | Verification Rule | Pass/Fail Criteria |
| --- | --- | --- | --- |
| **TC-HOT-01** | Hot-Path Invariant Overhead | Microbenchmarks across $10,000,000$ consecutive SELECT and MERGE queries. | Invariant check + $\mathbf{H}_{\text{query}}$ generation must measure $\le 0.25\,\mu\text{s}$ per operation ($p99$). |
| **TC-HOT-02** | Columnar Throughput | Bulk ingestion of parquet-grounded analytical records into local memory. | Sustained throughput $>12\text{M}$ rows/sec on Apple Silicon/native hardware. |
| **TC-MUT-01** | Anti-Vacuity Mutation | Invert operator enum within SHACL shape or inject aggregate into `sql:ProcessScope`. | Must fail closed with typed refusal (`REFUSAL-01` or `REFUSAL-03`); zero execution passes. |
| **TC-MOCK-00** | Pure Native Execution | Audit codebase for mocks, test doubles, or synthetic driver shims. | Build failure if any test utilizes mocks. All runs execute against live in-memory `frozen-duckdb 2.0.0`. |
| **TC-LEGAL-01** | Receipt Lineage Determinism | Compile identical AST with identical parameters across 10,000 iterations. | $\mathbf{H}_{\text{query}}$ must produce 100% byte-identical BLAKE3 hashes across all runs. |
| **TC-LEGAL-02** | Scope Leak Refusal | Submit query attempting to expose protected attribute in `sql:ViewScope`. | Blocked at admission; zero CPU execution cycles delivered to relational engine. |

> Implementation mapping: TC-HOT-01 = latency court (benches/tc_hot_01), TC-HOT-02 = throughput court (benches/tc_hot_02); see designs/TC-HOT-benchmarks.md.

---

## 6. Execution Phasing & Milestones

1. **Phase 1 (Compile & Memory Binding):** Generate native Rust/C structs and serialized byte constants directly from `ontologies/sql-ast.ttl` using `ggen`. Strip all runtime text parsers from release builds.
2. **Phase 2 (Lock-Free Write Ring Implementation):** Implement the SPSC ring buffer for $\mathbf{R}_{\text{receipt}}$ descriptors, decoupling the hot path from filesystem/database ledger sync.
3. **Phase 3 (Chicago Mutation & Anti-Vacuity Gate):** Deploy the mutation test harness across `tests/chicago_validation.rs`, confirming that every deliberate syntax, scope, or license evasion mutant is killed cleanly by typed refusals.
4. **Phase 4 (P28 License Packaging):** Embed the normative legal instrument (`urn:graphlaw:license:p28-sil:v1.0.0`) with the $50M safe-harbor and harmonic regularization terms into the release documentation and repository root.
