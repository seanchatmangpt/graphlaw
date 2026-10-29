// SPDX-FileCopyrightText: 2026 Sean Chatman <sean@graphlaw.org>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Architectural Review Document (ARD): RDF Dialects, Formal Semantics & Projection Mechanics
//!
//! Document Reference: ARD-26.10.25-DIALECTS
//! Target Release: GraphLaw v26.10.25
//! Subject: Grounding Dialects, Semantic Ingestion, Relational Projections, and the Loss Matrix

# Architectural Review Document (ARD): RDF Dialects, Formal Semantics & Projection Mechanics

**Document Reference:** `ARD-26.10.25-DIALECTS`  
**Target Release:** GraphLaw v26.10.25  
**Audience:** Technical Product Managers, Semantic Architects, Solutions Engineers  
**Subject:** Grounding Dialects, Semantic Ingestion, Relational Projections, and the Loss Matrix  

---

## 1. Architectural Philosophy: The "Sovereign Lattice"

Legacy knowledge graph implementations fail because they attempt to force all computational jobs into a single semantic dialect—either overloading OWL DL with procedural rules, bending SPARQL into an analytical query engine, or flattening rich contextual assertions into fragile string dictionaries.

GraphLaw treats RDF not as a monolithic storage medium, but as a **multi-tiered semantic continuum**. The system maintains an expressive, native representation of facts while compiling specialized projections for execution, validation, and alignment without loss of provenance.

```
                    ┌─────────────────────────────────────────────────────────┐
                    │               CANONICAL SOURCE OF TRUTH                 │
                    │               RDF 1.2 / RDF* + logic:                   │
                    │  (Statement Reification, Standpoints, Consequence Bld)  │
                    └────────────────────────────┬────────────────────────────┘
                                                 │
         ┌───────────────────────────┬───────────┴───────────┬───────────────────────────┐
         ▼                           ▼                       ▼                           ▼
 ┌───────────────┐           ┌───────────────┐       ┌───────────────┐           ┌───────────────┐
 │ Closed-World  │           │ Deductive &   │       │ Relational &  │           │ Cross-Domain  │
 │  Validation   │           │ Decidable DL  │       │ Vectorized    │           │ Alignment     │
 ├───────────────┤           ├───────────────┤       ├───────────────┤           ├───────────────┤
 │ SHACL 1.2     │           │ OWL 2 RL/DL   │       │ PurRDF 5-Table│           │ SSSOM / EDOAL │
 │ ShEx 2.1      │           │ Datalog / N3  │       │ Parquet Push  │           │ FnO Mappings  │
 └───────────────┘           └───────────────┘       └───────────────┘           └───────────────┘
```

---

## 2. Ingested & Evaluated RDF Dialects

GraphLaw provides native, zero-runtime-dependency parsing, compilation, and evaluation across four distinct operational dialects.

### 2.1. Canonical State Layer: Native RDF 1.2 & RDF-Star (RDF*)

* **Role:** The immutable, loss-free recording medium for all knowledge entries, claims, and revisions.
* **Dialect Mechanics:**
  * Uses RDF 1.2 triple terms (`<< :subject :predicate :object >>`) as first-class citizens.
  * Treats metadata (who said it, at what time, from what viewpoint, under what confidence score) as direct reifiers rather than disconnected blank nodes.
  * Preserves directional literals, language tags, and custom data types without flattening.
* **Non-Erasure Guarantee:** State revision operates via **tenure suppression** (`gmeow:displayable false`), retaining historical inferences in the audit trail rather than executing mutable deletions.

### 2.2. Closed-World Structural Validation: SHACL 1.2 & ShEx 2.1

* **Role:** Deterministic boundary enforcement and data-shape integrity gates.
* **Dialect Mechanics:**
  * **SHACL 1.2 Core & Advanced Features (AF):** Enforces data contracts, cardinality limits, node expressions, and custom SPARQL target assertions. Validates incoming customer datasets against pre-set constraints before admitting them into execution.
  * **ShEx 2.1 (Shape Expressions):** Provides compact, deterministic contract checks across API boundaries, guaranteeing that incoming payloads conform strictly to structural definitions.
* **Operational Rule:** SHACL and ShEx are strictly **Validation-Only** dialects. They evaluate state; they never invent or infer new assertions into the graph.

### 2.3. Inference & Entailment: Deterministic Datalog, N3 & OWL 2 RL

* **Role:** Rule evaluation, transitive dependency resolution, and logical deduction.
* **Dialect Mechanics:**
  * **Datalog (Semi-Naive Engine):** Implements recursive graph reachability, role composition, and hierarchical containment (e.g., determining multi-hop ownership or structural risk) with guaranteed mathematical termination.
  * **Notation3 (N3):** Ingests functional implications, mathematical logic rules, and complex constraint expressions directly over RDF syntax (`purrdf::n3`).
  * **OWL 2 RL Downcast:** Evaluates all 78 official OWL 2 RL rule tables without requiring memory-intensive open-world theorem provers. It computes the full logical closure in a single pass directly in system memory.

### 2.4. Cross-Vocabulary Alignment: SSSOM, EDOAL & FnO

* **Role:** Translating foreign enterprise schemas into GraphLaw canonical invariants.
* **Dialect Mechanics:**
  * **SSSOM (Simple Standard for Sharing Ontological Mappings):** Maps foreign entities to GraphLaw terms using unambiguous semantic relations (`skos:exactMatch`, `skos:broadMatch`).
  * **EDOAL (Expressive and Declarative Ontology Alignment Language):** Expresses complex structural cross-mappings (e.g., combining a `givenName` and `familyName` from a legacy database into a single composite identity node).
  * **Function Ontology (FnO):** Encapsulates declarative, reversible transformation rules (such as coordinate system transforms or currency conversions) as execution graphs.

---

## 3. The Execution Projection: The PurRDF Parquet Contract

GraphLaw bridges semantic reasoning and high-speed execution by projecting high-order RDF 1.2 graphs into a flat, columnar relational representation known as the **PurRDF 5-Table Specification** (`purrdf-columnar`).

Instead of storing data as linked objects or pointer-heavy memory structures, the graph is serialized into five immutable Apache Parquet tables:

```
┌────────────────────────────────────────────────────────────────────────┐
│                        PURRDF 5-TABLE CONTRACT                         │
│                                                                        │
│  1. terms.parquet        [ id (i64) | kind | lex | datatype | lang ]   │
│  2. quads.parquet        [ s (i64) | p (i64) | o (i64) | g (i64)? ]    │
│  3. reifiers.parquet     [ reifier (i64) | s | p | o | g? ]            │
│  4. annotations.parquet  [ reifier (i64) | predicate | value | g? ]    │
│  5. blobs.parquet        [ digest (utf8) | bytes (binary) ]            │
└────────────────────────────────────────────────────────────────────────┘
```

### Relational Pushdown Compilation

1. **Integer Dictionary Encoding:** Every unique IRI, literal, or blank node is mapped to a contiguous 64-bit integer (`i64`) in the `terms` table.
2. **Graph Queries as Relational Joins:** Complex SPARQL, Datalog, and SHACL queries compile directly into an algebraic **Abstract Syntax Tree (AST)** of SQL joins over the `quads` table.
3. **Hardware-Level SIMD Execution:** Because graph statements are stored as pure integer tuples, an embedded vectorized engine (`frozen-duckdb`) evaluates multi-hop joins across millions of relationships in milliseconds using CPU vector instructions (AVX-512 / ARM Neon), bypassing pointer dereferencing completely.

---

## 4. The Formal Loss Matrix & Projection Fidelity

When exporting data to third-party consumer formats, GraphLaw enforces a strict **Preservation Law**. Lowering rich RDF 1.2 data into simpler legacy formats inevitably strips nuances; GraphLaw tracks this loss explicitly via an automated, machine-verifiable ledger.

| Consumer Target | Dialect Lowered | Features Preserved | Documented Loss / Mitigations |
| --- | --- | --- | --- |
| **schema.org / vCard** | JSON-LD / Microdata | Core entities, labels, basic types, top-level relations. | **Lossy:** Reified statements, confidence scores, and standpoint contexts are stripped. |
| **Legacy OWL 2 DL** | RDF/XML or Turtle | Formal class hierarchies, property domains, and ranges. | **Downgraded:** RDF 1.2 statement annotations are converted down into `owl:Axiom` annotations. |
| **Analytics (DuckDB)** | Columnar Parquet | Full graph topology, integer terms, statement IDs, blobs. | **Exact:** Complete lossless reproduction of quads, reifiers, and cryptographic proofs. |
| **Graph Visualization** | openCypher / CSV | Node labels, edge types, simple key-value attributes. | **Lossy:** Multi-dimensional reference frames and epistemic qualifications are omitted. |

---

## 5. Architectural Invariants for Implementation Teams

1. **The Single Source Law:** All domain models and institutional rules must be authored once in canonical RDF 1.2 / `logic:`. Down-projections (JSON Schema, TypeScript interfaces, OpenAPI, and SQL tables) must be generated by build tools; they must never be hand-edited.
2. **The Zero-Silent-Approximation Law:** If a legacy format or target consumer cannot represent a semantic nuance (such as an uncertain date or a contested standpoint), the compiler must record the dropped construct in a machine-readable loss ledger (`loss_matrix.json`). It must never silently pick a "default" winner.
3. **No External Network Dependencies:** All dialect validation, schema parsing, and AST transformations must execute strictly in-memory using compiled native Rust/WASM libraries, with zero outbound calls to remote ontology hosts.
