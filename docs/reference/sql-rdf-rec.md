# W3C Recommendation: SQL in RDF (`sql-rdf`)

> **Status line (SQLRDF-12):** This file is the **conformance text for graphlaw
> v26.10.22** — it is the pasted Recommendation text under which tickets
> SQLRDF-01..SQLRDF-11 were written. It is **not** a fetched W3C publication,
> and its W3C status claims below are document text, not facts asserted by
> this repository.

**World Wide Web Consortium (W3C Recommendation 22 October 2026)**

* **This version:** `https://www.w3.org/TR/2026/REC-sql-rdf-20261022/`
* **Latest published version:** `https://www.w3.org/TR/sql-rdf/`
* **Editor:** Sir Tim Berners-Lee, *World Wide Web Consortium / GraphLaw Architecture Working Group*
* **Authors & Contributors:** GraphLaw Semantic Systems Architecture Working Group

---

## Abstract

When we first conceived the World Wide Web, the foundational premise was universality: that any piece of information, whether a scientific article, a raw tabular observation, or an abstract computational model, could be named, addressed, and interconnected through dereferenceable Uniform Resource Identifiers (URIs). The Semantic Web extended this vision by introducing the Resource Description Framework (RDF)—asserting that the structure of human knowledge is inherently a graph of relations.

Yet for four decades, the vast majority of the world's operational data has remained locked within relational databases, expressed through Structured Query Language (SQL) statements that are treated by web agents as opaque strings of characters. When queries exist only as unstructured strings, they cannot be reasoned over, statically verified, checked against decentralized policy graphs, or safely compiled without introducing security vulnerabilities such as injection attacks.

This specification, **SQL in RDF (`sql-rdf`)**, establishes an open, normative standard that represents relational syntax, relational algebra, and execution trees as first-class, verifiable RDF graphs. Grounded in the Parsing Expression Grammar (PEG) of DuckDB 2.0.0-dev and validated through closed-world Shapes Constraint Language (SHACL) shapes, `sql-rdf` unifies relational queries with graph semantics, providing a deterministic, verifiable, and fail-closed bridge between the relational universe and the Semantic Web.

---

## Status of This Document

This section describes the status of this document at the time of its publication. Other documents may supersede this document. A list of current W3C publications and the latest revision of this technical report can be found in the W3C technical reports index at `https://www.w3.org/TR/`.

This document has been reviewed by W3C Members, software architects, and relational engine implementers, and is endorsed by the Director as a W3C Recommendation. It is a stable document and may be used as reference material or cited from another document.

---

## 1. Philosophical Foundations & Architectural Principles

### 1.1 The Web of Data and the Relational Silo

The World Wide Web succeeded because it did not dictate a uniform storage architecture for servers; instead, it specified a uniform protocol (HTTP) and a uniform identifier space (URIs). When the Semantic Web was framed, our goal was not to eliminate specialized computing systems, but to enable interoperability at the semantic layer.

Relational databases excel at bulk vectorized arithmetic, row-and-column storage layouts, and algebraic joins. However, the mechanism through which applications communicate with these engines—SQL text—has remained an architectural silo. A SQL query passed over an operational connection is typically a black box to intermediate proxies, web agents, and decentralized verification nodes. By elevating the SQL Abstract Syntax Tree (AST) into an RDF graph, we bring relational operations directly into the Web of Data.

### 1.2 The Principle of Least Power

The *Principle of Least Power* suggests that on the Web, one should choose the least powerful language suitable for a given task. When query operations are expressed as imperative scripts or string-interpolated procedural code, understanding their intent requires full Turing-complete execution.

Expressing queries as declarative RDF graphs constrained by SHACL ensures that:

* The query structure can be statically analyzed without running it.
* Security policies, access controls, and operational scopes can be mathematically proven prior to evaluation.
* The semantics of the operation remain invariant across execution environments, from native database engines to embedded sandboxes.

### 1.3 Scope Discrimination: Viewing vs. Processing

Drawing upon the ARGOS access-control model, this specification recognizes that relational evaluation entails two distinct actions: *viewing* data and *processing* data. A column referenced in a `SELECT` clause crosses the execution boundary into the consumer's view. Conversely, a column referenced inside a `WHERE`, `JOIN`, or `HAVING` predicate is processed internally by the engine to filter records, but may never be exposed to the user. `sql-rdf` models this boundary normatively via explicit scoping properties (`sql:ViewScope` and `sql:ProcessScope`), allowing semantic graphs to guard against side-channel exfiltration while supporting query optimization.

### 1.4 Fail-Closed Execution and Cryptographic Determinism

In the open Semantic Web, we operate under the Open-World Assumption (OWA): the absence of a triple merely implies that an assertion is unknown. However, database operations must be safe. A query missing a mandatory predicate cannot be assumed benign; an unrecognized token cannot be ignored.

`sql-rdf` mandates a **Fail-Closed** execution posture. Conforming systems MUST validate graph ASTs against strict, closed-world SHACL shapes prior to admission. An admitted AST produces an immutable, content-addressable BLAKE3 query hash:

$$\mathbf{H}_{\text{query}} = \operatorname{BLAKE3}\left(\operatorname{Canon}(G_{\text{AST}}) \mathbin{\Vert} \mathbf{P}_{\text{params}}\right)$$

> **Harmonization note (2026-10-22, SQLRDF-18):** the full P28-SIL license form of this
> hash (LICENSE-P28-SIL.md §1.5) carries a third term,
> $\oint_{\partial \Omega} \omega_{\text{context}}$ — the **deployment epoch**: an
> immutable, deployment-scoped `u64` epoch identifier assigned once at substrate
> initialization, not ambient hardware state. The two-term formula above is the
> $\omega_{\text{context}} = 0$ (single-deployment) special case; the epoch term is
> **reserved** and zero-valued in single-deployment mode. Within a deployment the
> epoch is constant, so receipts stay deterministic and replayable; across
> deployments receipts are domain-separated. Conforming implementations MAY ignore
> the term while it is reserved.

This receipt certifies that the state transition $\Delta S_{\text{admitted}}$ conforms strictly to cryptographic resource leases $\mathbf{\Phi}_{\text{lease}}$:

$$\Delta S_{\text{admitted}} + \mathbf{\Phi}_{\text{lease}} \equiv \mathbf{R}_{\text{receipt}} \pmod{\mathbf{\Sigma}_{\text{PQC}}}$$

---

## 2. Conformance & Notation

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** in this document are to be interpreted as described in BCP 14 (RFC 2119, RFC 8174).

* **Namespace IRI:** `http://graphlaw.org/sql#`
* **Normative Prefix:** `sql:`

---

## 3. The `sql-rdf` Vocabulary Specification

### 3.1 Taxonomy of Statement Classes

Every relational command is modeled as an instance of `sql:Statement`, which descends from the root abstract class `sql:ASTNode`. The taxonomy incorporates thirty-five statement alternatives derived directly from the DuckDB 2.0.0-dev PEG grammar (`common.gram`, commit `b6013f7aa2`).

```turtle
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix sql:  <http://graphlaw.org/sql#> .

# Root Classes
sql:ASTNode a rdfs:Class ;
    rdfs:label "AST Node" ;
    rdfs:comment "Abstract superclass of all SQL Abstract Syntax Tree elements." .

sql:Statement a rdfs:Class ;
    rdfs:subClassOf sql:ASTNode ;
    rdfs:label "SQL Statement" ;
    rdfs:comment "Top-level relational statement capable of compilation and execution." .

# Major Statement Families
sql:DataQueryStatement a rdfs:Class ; rdfs:subClassOf sql:Statement .
sql:DataManipulationStatement a rdfs:Class ; rdfs:subClassOf sql:Statement .
sql:DataDefinitionStatement a rdfs:Class ; rdfs:subClassOf sql:Statement .
sql:TransactionStatement a rdfs:Class ; rdfs:subClassOf sql:Statement .
sql:SessionStatement a rdfs:Class ; rdfs:subClassOf sql:Statement .
sql:ExtensionStatement a rdfs:Class ; rdfs:subClassOf sql:Statement .
```

### 3.2 Normative Statement Inventory & Engine Execution Status

Conforming implementations **MUST** recognize all 35 statement classes. The embedded `frozen-duckdb` runtime executes 33 of these classes to completion, while enforcing formal typed refusals for unsupported operations (`ConnectStatement` and `DisconnectStatement`).

| Statement Class IRI | Classification | DuckDB 2.0.0-dev PEG Rule | Execution Status & Behavior |
| --- | --- | --- | --- |
| `sql:SelectStatement` | `sql:DataQueryStatement` | `SelectStatement` | Compiles to parameterized DQL; executes to completion. |
| `sql:InsertStatement` | `sql:DataManipulationStatement` | `InsertStatement` | State mutation; inserts rows into target catalog table. |
| `sql:UpdateStatement` | `sql:DataManipulationStatement` | `UpdateStatement` | State mutation; updates records matching predicate. |
| `sql:DeleteStatement` | `sql:DataManipulationStatement` | `DeleteStatement` | State mutation; removes records matching predicate. |
| `sql:MergeIntoStatement` | `sql:DataManipulationStatement` | `MergeIntoStatement` | Complex analytical upsert combining match criteria. |
| `sql:CreateStatement` | `sql:DataDefinitionStatement` | `CreateStatement` | Schema mutation; creates tables, views, schemas, types. |
| `sql:AlterStatement` | `sql:DataDefinitionStatement` | `AlterStatement` | Schema mutation; alters columns, names, or constraints. |
| `sql:DropStatement` | `sql:DataDefinitionStatement` | `DropStatement` | Schema mutation; drops catalog entities. |
| `sql:TruncateStatement` | `sql:DataDefinitionStatement` | `TruncateStatement` | Storage mutation; empties table contents immediately. |
| `sql:CommentStatement` | `sql:DataDefinitionStatement` | `CommentStatement` | Metadata mutation; sets catalog comments on objects. |
| `sql:CopyStatement` | `sql:DataManipulationStatement` | `CopyStatement` | IO stream; transfers data to/from external formats. |
| `sql:ExportStatement` | `sql:SessionStatement` | `ExportStatement` | Serializes complete database state to external path. |
| `sql:ImportStatement` | `sql:SessionStatement` | `ImportStatement` | Ingests structural schema and contents from export. |
| `sql:PrepareStatement` | `sql:SessionStatement` | `PrepareStatement` | Parameterized plan compilation and registration. |
| `sql:ExecuteStatement` | `sql:SessionStatement` | `ExecuteStatement` | Executes a previously prepared statement by handle. |
| `sql:DeallocateStatement` | `sql:SessionStatement` | `DeallocateStatement` | Removes a prepared execution plan from engine cache. |
| `sql:ExplainStatement` | `sql:SessionStatement` | `ExplainStatement` | Returns physical and logical query execution plans. |
| `sql:VacuumStatement` | `sql:SessionStatement` | `VacuumStatement` | Reclaims storage pages and rebuilds statistics. |
| `sql:AnalyzeStatement` | `sql:SessionStatement` | `AnalyzeStatement` | Computes optimizer distributions across columns. |
| `sql:CheckpointStatement` | `sql:SessionStatement` | `CheckpointStatement` | Flushes write-ahead log (WAL) pages to storage. |
| `sql:CallStatement` | `sql:SessionStatement` | `CallStatement` | Invokes stored procedures or table-valued macros. |
| `sql:PragmaStatement` | `sql:SessionStatement` | `PragmaStatement` | Reads or writes database operational parameters. |
| `sql:SetStatement` | `sql:SessionStatement` | `SetStatement` | Modifies configuration variables for active session. |
| `sql:ResetStatement` | `sql:SessionStatement` | `ResetStatement` | Resets configuration variables to engine defaults. |
| `sql:TransactionStatement` | `sql:TransactionStatement` | `TransactionStatement` | Manages atomicity boundaries (`BEGIN`, `COMMIT`, `ROLLBACK`). |
| `sql:LoadStatement` | `sql:ExtensionStatement` | `LoadStatement` | Loads dynamic shared library extensions into engine. |
| `sql:InstallStatement` | `sql:ExtensionStatement` | `InstallStatement` | Installs extension packages from local or remote repositories. |
| `sql:UpdateExtensionsStatement` | `sql:ExtensionStatement` | `UpdateExtensionsStatement` | Checks and updates installed engine extensions. |
| `sql:ExtensionRepositoryStatement` | `sql:ExtensionStatement` | `ExtensionRepositoryStatement` | Registers secure repository endpoints for extensions. |
| `sql:ExternalResourceStatement` | `sql:SessionStatement` | `ExternalResourceStatement` | Manages credentials and endpoints for external object stores. |
| `sql:AttachStatement` | `sql:SessionStatement` | `AttachStatement` | Mounts external database files into namespace catalog. |
| `sql:DetachStatement` | `sql:SessionStatement` | `DetachStatement` | Unmounts database files from active engine catalog. |
| `sql:UseStatement` | `sql:SessionStatement` | `UseStatement` | Sets default schema and database resolution path. |
| `sql:ConnectStatement` | `sql:SessionStatement` | `ConnectStatement` | **Engine Refusal:** `CONNECT LOCAL is not yet implemented` (illustrative; courts pin loose substrings `not yet implemented` / `no active CONNECT` at `tests/duckdb_statement_kinds.rs:115`). |
| `sql:DisconnectStatement` | `sql:SessionStatement` | `DisconnectStatement` | **Engine Refusal:** `no active CONNECT` (illustrative; same substring pin at `tests/duckdb_statement_kinds.rs:115`). |

### 3.3 Syntactic Clauses and Scoped Expressions

Statement nodes link to their constituent syntactic clauses via functional subproperties of `sql:hasClause`. To enforce sequence without resorting to brittle blank-node chains, clauses containing ordered expressions **MUST** employ standard `rdf:List` collections.

```turtle
# Clause Relations
sql:hasClause a rdf:Property ;
    rdfs:domain sql:Statement ;
    rdfs:range sql:ASTNode .

sql:select a rdf:Property ;
    rdfs:subPropertyOf sql:hasClause ;
    rdfs:domain sql:SelectStatement ;
    rdfs:range rdf:List ;
    rdfs:comment "Ordered projection list evaluated under sql:ViewScope." .

sql:from a rdf:Property ;
    rdfs:subPropertyOf sql:hasClause ;
    rdfs:domain sql:Statement ;
    rdfs:range rdf:List ;
    rdfs:comment "List of data sources, joins, or table-valued functions." .

sql:where a rdf:Property ;
    rdfs:subPropertyOf sql:hasClause ;
    rdfs:domain sql:Statement ;
    rdfs:range sql:Expression ;
    rdfs:comment "Filtering expression evaluated under sql:ProcessScope." .

sql:groupBy a rdf:Property ;
    rdfs:subPropertyOf sql:hasClause ;
    rdfs:domain sql:SelectStatement ;
    rdfs:range rdf:List .

sql:having a rdf:Property ;
    rdfs:subPropertyOf sql:hasClause ;
    rdfs:domain sql:SelectStatement ;
    rdfs:range sql:Expression .

sql:orderBy a rdf:Property ;
    rdfs:subPropertyOf sql:hasClause ;
    rdfs:domain sql:SelectStatement ;
    rdfs:range rdf:List .

sql:limit a rdf:Property ;
    rdfs:subPropertyOf sql:hasClause ;
    rdfs:domain sql:SelectStatement ;
    rdfs:range xsd:integer .

sql:offset a rdf:Property ;
    rdfs:subPropertyOf sql:hasClause ;
    rdfs:domain sql:SelectStatement ;
    rdfs:range xsd:integer .

# Scoping Properties
sql:Scope a rdfs:Class .
sql:ViewScope a sql:Scope ;
    rdfs:comment "Scope for terms exposed directly in query outputs." .
sql:ProcessScope a sql:Scope ;
    rdfs:comment "Scope for terms evaluated internally in predicates or joins." .

sql:scope a rdf:Property ;
    rdfs:domain sql:ASTNode ;
    rdfs:range sql:Scope .
```

---

## 4. Closed-World Validation: Normative SHACL Shapes

To guarantee deterministic execution and reject malformed syntax trees prior to engine evaluation, an implementation **MUST** validate all graphs against the SHACL shapes defined in `ontologies/sql-ast.ttl`.

```turtle
@prefix sh:  <http://www.w3.org/ns/shacl#> .
@prefix sql: <http://graphlaw.org/sql#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

# Statement Invariant Shape
sql:StatementBaseShape a sh:NodeShape ;
    sh:targetClass sql:Statement ;
    sh:property [
        sh:path sql:statementIndex ;
        sh:datatype xsd:integer ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:severity sh:Violation ;
        sh:message "Every statement MUST contain an integer execution sequence index." ;
    ] .

# Select Statement Shape
sql:SelectStatementShape a sh:NodeShape ;
    sh:targetClass sql:SelectStatement ;
    sh:property [
        sh:path sql:select ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:node sql:OrderedListShape ;
        sh:severity sh:Violation ;
        sh:message "A SelectStatement MUST contain exactly one projection list." ;
    ] ;
    sh:property [
        sh:path sql:from ;
        sh:maxCount 1 ;
        sh:node sql:OrderedListShape ;
        sh:severity sh:Violation ;
    ] ;
    sh:property [
        sh:path sql:where ;
        sh:maxCount 1 ;
        sh:class sql:Expression ;
        sh:severity sh:Violation ;
    ] .

# Binary Expression Shape
sql:BinaryExpressionShape a sh:NodeShape ;
    sh:targetClass sql:BinaryExpression ;
    sh:property [
        sh:path sql:left ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:class sql:ASTNode ;
        sh:severity sh:Violation ;
    ] ;
    sh:property [
        sh:path sql:operator ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:datatype xsd:string ;
        sh:in ( "=" "!=" "<" "<=" ">" ">=" "AND" "OR" "+" "-" "*" "/" "%" "LIKE" "ILIKE" ) ;
        sh:severity sh:Violation ;
        sh:message "Expression operator MUST be an approved comparison, logical, or arithmetic token." ;
    ] ;
    sh:property [
        sh:path sql:right ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:class sql:ASTNode ;
        sh:severity sh:Violation ;
    ] .

# Ordered Collection Shape
sql:OrderedListShape a sh:NodeShape ;
    sh:property [
        sh:path rdf:first ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:severity sh:Violation ;
    ] ;
    sh:property [
        sh:path rdf:rest ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:severity sh:Violation ;
    ] .
```

---

## 5. Formal Typed Refusal Taxonomy

When an incoming graph violates syntactic constraints, requests an unexecutable operation, or exceeds environment isolation boundaries, the implementation **MUST NOT** crash or raise generic database exceptions. Instead, it **MUST** emit a structured, typed refusal adhering to `docs/refusals.md`.

| Refusal Code | Category Identifier | Normative Trigger Condition | Recovery / Remediation |
| --- | --- | --- | --- |
| `REFUSAL-01` | `MalformedAstRefusal` | Node fails validation against shapes in `ontologies/sql-ast.ttl`. | Graph returned to sender with SHACL validation report details. |
| `REFUSAL-02` | `UnimplementedStatementRefusal` | Node represents a recognized statement that the engine refuses (`ConnectStatement`, `DisconnectStatement`). | Operation rejected; recorded as a witnessed engine refusal. |
| `REFUSAL-03` | `ScopeIntegrityRefusal` | Term violates operational scope boundaries (e.g., aggregate placed inside `WHERE`). | Compilation rejected; sender must re-scope expression. |
| `REFUSAL-04` | `CapabilityRefusal` | Builtin SQL eval was given a reader that is not document-backed. The wasm module still admits and compiles SQL-in-RDF through `compile_turtle`. | Supply the admitted Turtle document (`query_document_turtle`) and compile with `compile_turtle`. |

---

## 6. Verification and Implementation Invariants

### 6.1 The Chicago (Classicist) Testing Doctrine

Conforming implementations of `sql-rdf` **MUST** be verified according to the **Chicago School of Test-Driven Development**:

1. **Zero Test Doubles:** Execution courts **MUST NOT** employ mocks, fakes, or simulated engine drivers. All tests must execute against real, in-memory `frozen-duckdb` instances.
2. **Anti-Vacuity Guarantees:** Test courts **MUST** enforce anti-vacuity mutation checks. Any test suite that passes in the presence of inverted operators or mutated data tables is non-conforming.
3. **Total Statement Coverage:** All 33 admitted statement types must execute to completion on dedicated connections, verifying engine state changes or valid result sets.

### 6.2 Certified Built-in Extensions

The runtime environment **MUST** certify that built-in engine extensions answer verification queries directly through the generated AST bindings:

| Extension Name | Catalog Status | Operational Verification Test | Certified Result |
| --- | --- | --- | --- |
| `core_functions` | Installed & Loaded | `SELECT list_sum([1, 2, 3])` | Returned integer `6` |
| `icu` | Installed & Loaded | `SELECT CASE WHEN count(*) > 0 THEN 1 ELSE 0 END FROM icu_calendar_names()` | Returned integer `1` |
| `json` | Installed & Loaded | `SELECT json_extract('{"a": 1}', '$.a')::INTEGER` | Returned integer `1` |
| `parquet` | Installed & Loaded | `COPY (...) TO 'surface.parquet'; SELECT count(*) FROM read_parquet(...)` | Returned integer `1` |
| `tpch` | Installed | `CALL dbgen(sf = 0.01); SELECT count(*) FROM nation` | Returned integer `25` (courted at `tests/duckdb_statement_kinds.rs:573`, count pinned at `:605`; supersedes earlier `sf = 0` wording) |

---

## 7. Concrete Example: Analytical Upsert Operation

To demonstrate end-to-end compilation, the following complete Turtle graph models an analytical upsert (`MERGE INTO`), joining a staged invoice aggregation subquery with an operational sales ledger.

```turtle
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix sql:  <http://graphlaw.org/sql#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .

_:mergeStatement a sql:MergeIntoStatement ;
    sql:statementIndex 1 ;
    sql:targetTable [
        a sql:TableIdentifier ;
        sql:schemaName "main" ;
        sql:tableName "regional_sales"
    ] ;
    sql:usingSource [
        a sql:SelectStatement ;
        sql:statementIndex 1 ;
        sql:select (
            [ a sql:ColumnIdentifier ;
              sql:columnName "region_id" ;
              sql:scope sql:ViewScope ]
            [ a sql:FunctionExpression ;
              sql:functionName "sum" ;
              sql:arguments ( [ a sql:ColumnIdentifier ; sql:columnName "amount" ; sql:scope sql:ProcessScope ] ) ;
              sql:alias "total_invoiced" ;
              sql:scope sql:ViewScope ]
        ) ;
        sql:from (
            [ a sql:TableIdentifier ;
              sql:schemaName "staging" ;
              sql:tableName "invoices" ]
        ) ;
        sql:where [
            a sql:BinaryExpression ;
            sql:left [ a sql:ColumnIdentifier ; sql:columnName "status" ; sql:scope sql:ProcessScope ] ;
            sql:operator "=" ;
            sql:right [ a sql:LiteralExpression ; sql:value "SETTLED" ; sql:datatype xsd:string ]
        ] ;
        sql:groupBy (
            [ a sql:ColumnIdentifier ; sql:columnName "region_id" ; sql:scope sql:ProcessScope ]
        )
    ] ;
    sql:mergeOn [
        a sql:BinaryExpression ;
        sql:left [ a sql:ColumnIdentifier ; sql:qualifier "main.regional_sales" ; sql:columnName "region_id" ; sql:scope sql:ProcessScope ] ;
        sql:operator "=" ;
        sql:right [ a sql:ColumnIdentifier ; sql:qualifier "src" ; sql:columnName "region_id" ; sql:scope sql:ProcessScope ]
    ] ;
    sql:whenMatched [
        a sql:MergeUpdateAction ;
        sql:setAssignments (
            [ a sql:Assignment ;
              sql:targetColumn "revenue" ;
              sql:valueExpression [
                  a sql:BinaryExpression ;
                  sql:left [ a sql:ColumnIdentifier ; sql:qualifier "main.regional_sales" ; sql:columnName "revenue" ; sql:scope sql:ProcessScope ] ;
                  sql:operator "+" ;
                  sql:right [ a sql:ColumnIdentifier ; sql:qualifier "src" ; sql:columnName "total_invoiced" ; sql:scope sql:ProcessScope ]
              ]
            ]
        )
    ] .
```

### 7.1 Target Parameterized SQL Projection

The AST compiler converts the graph into safe, parameterized SQL with no intermediate string concatenation:

```sql
MERGE INTO main.regional_sales
USING (
    SELECT region_id, sum(amount) AS total_invoiced
    FROM staging.invoices
    WHERE status = $1
    GROUP BY region_id
) AS src
ON main.regional_sales.region_id = src.region_id
WHEN MATCHED THEN UPDATE SET revenue = (main.regional_sales.revenue + src.total_invoiced);
```

* **Extracted Parameter Buffer:** `[$1 = "SETTLED"^^xsd:string]`
* **Receipt Determinism:** Produces an immutable BLAKE3 query hash binding the canonical graph and extracted literal buffer.

---

## 8. Epilogue: On the Future of Web Intelligence

The integration of relational engines and Semantic Web graphs is not a mere syntactic exercise; it is an architectural necessity. As artificial agents and autonomous software components increasingly mediate human affairs, we cannot rely on stochastic text generation to execute commands across critical relational datastores. By insisting that every query be an immutable graph, every shape be statically verified, and every state mutation be cryptographically provable, we ensure that the Web remains open, safe, and universally intelligible.
