# C4 Architecture Documentation — Deterministic Agent Governance Platform

Filed verbatim (operator text, 2026-10-22). Mapped across Level 1 (System
Context), Level 2 (Container), and Level 3 (Component) models using Mermaid.

## Level 1: System Context Diagram

The System Context diagram illustrates the boundary of the Deterministic
Agent Governance Platform, showing how external actors (engineers),
development assets (repositories), verification environments, and audit
sinks interact with the core engine.

```mermaid
C4Context
title System Context: Deterministic Agent Governance Platform

Person(developer, "Software Engineer", "Defines high-level requirements, architectural invariants, and SHACL/ShEx schemas.")
System(graphlaw_system, "Autonomous Lifecycle Platform", "Coordinates heuristic proposal generation (Claude Code) under deterministic WASM admission (GraphLaw).")
System_Ext(vcs, "Code Repository & Filesystem", "Hosts source code, project ontologies, and target AST structures.")
System_Ext(courts, "Verification Courts", "Adversarial mutation suites and anti-vacuity test runners.")
System_Ext(ledger, "Audit Ledger (CHAIN.blake3)", "Append-only, Merkle-linked cryptographic ledger storing signed execution receipts.")

Rel(developer, graphlaw_system, "Submits intent & governance shapes", "CLI / Terminal")
Rel(graphlaw_system, vcs, "Applies admitted projections", "File I/O")
Rel(graphlaw_system, courts, "Dispatches state deltas for adversarial testing", "IPC / Process Exec")
Rel(graphlaw_system, ledger, "Emits sealed transition receipts", "BLAKE3 / Stream")

```

### Context Entities

* **Software Engineer**: Supplies intent and maintains formal constraints
  (ontologies, schemas, policies) rather than writing unconstrained prompts.
* **Autonomous Lifecycle Platform**: The closed-loop system enforcing zero
  unreceipted actuation.
* **Code Repository & Filesystem**: The persistent software baseline modified
  only through admitted AST projections.
* **Verification Courts**: Post-execution environments checking for semantic
  vacuity and test regression before final receipt attestation.
* **Audit Ledger**: The permanent, tamper-evident record establishing
  non-repudiation across all agent iterations.

## Level 2: Container Diagram

The Container diagram decomposes the platform into the agent runtime,
interception hooks, the WASM admission sandbox, code synthesizers, and
persistence layers.

```mermaid
C4Container
title Container Diagram: Runtime & Governance Kernel

Person(developer, "Software Engineer", "Issues prompts and reviews verification receipts.")

System_Boundary(platform_boundary, "Deterministic Governance Platform") {
    Container(agent_runtime, "Claude Code Runtime", "LLM Inference Engine", "Proposes task decompositions, subagent steps, and semantic slot values (Authority: NONE).")
    Container(lifecycle_hook, "Lifecycle Hook Adapter", "TypeScript / Rust Shim", "Intercepts PreToolUse and PostToolUse events; reifies actions into RDF quads.")
    Container(wasm_kernel, "GraphLaw WASM Kernel", "WebAssembly (Rust / Oxigraph)", "Sandboxed kernel evaluating structural shapes, reactive refusals, and temporal leases.")
    Container(projection_engine, "Projection Synthesizer (ggen)", "Rust CLI", "Deterministically projects admitted RDF slot values into valid code files.")
}

System_Ext(vcs, "Source Repository", "Local Git workspace & AST files.")
System_Ext(courts, "Continuous Mutation Courts", "Runs Chicago mutation and boundary tests.")
System_Ext(ledger, "Audit Ledger", "Append-only Merkle receipt log (CHAIN.blake3).")

Rel(developer, agent_runtime, "Inputs task intent", "CLI")
Rel(agent_runtime, lifecycle_hook, "Emits candidate tool invocation", "Hook Event")
Rel(lifecycle_hook, wasm_kernel, "Sends candidate quads (_:act a act:ToolCall)", "WASM Memory")
Rel(wasm_kernel, lifecycle_hook, "Returns Admitted (O*) or Typed Refusal", "Memory Buffer")
Rel(lifecycle_hook, agent_runtime, "Injects typed refusal into prompt context", "Context Feedback")
Rel(lifecycle_hook, projection_engine, "Triggers code generation upon admittance", "IPC")
Rel(projection_engine, vcs, "Emits formatted file mutations", "Disk I/O")
Rel(lifecycle_hook, courts, "Dispatches post-actuation candidate", "Subprocess")
Rel(lifecycle_hook, ledger, "Commits signed transition receipt", "File Append")

```

### Container Responsibilities

* **Claude Code Runtime**: Confined to heuristic candidate synthesis ($O$).
  Has zero direct access to OS syscalls, network sockets, or raw disk files.
* **Lifecycle Hook Adapter**: Acts as the boundary gateway. It normalizes
  unstructured agent requests into formal RDF statements before calling the
  kernel.
* **GraphLaw WASM Kernel**: An isolated, zero-leakage micro-kernel. Executes
  all validation logic deterministically in sub-millisecond execution windows.
* **Projection Synthesizer (`ggen`)**: Eliminates free-form string
  concatenation by compiling admitted graph structures directly into
  validated ASTs.

## Level 3: Component Diagram (GraphLaw WASM Kernel)

The Component diagram inspects the internals of the **GraphLaw WASM
Kernel**, isolating how incoming candidate transitions are validated,
governed, and attested.

```mermaid
C4Component
title Component Diagram: GraphLaw WASM Kernel Internals

Container_Boundary(kernel_boundary, "GraphLaw WASM Kernel") {
    Component(quad_reifier, "Action Reifier", "Rust / WASM Memory", "Deserializes tool parameters into RDF triples conforming to act: vocabulary.")
    Component(shape_engine, "SHACL / ShEx Shape Engine", "Oxigraph Core", "Validates candidate triples against closed-world schema rules and architectural layer constraints.")
    Component(datalog_evaluator, "Datalog Temporal Calculus", "Datalog Engine", "Checks lease duration, monotonic scope counters, and agent privilege grants.")
    Component(sparql_refusal, "SPARQL Reactive Veto", "SPARQL Matcher", "Runs zero-latency refusal queries targeting prohibited patterns.")
    Component(receipt_sealer, "Attestation & Receipt Sealer", "BLAKE3 Cryptographic Engine", "Binds subject identity, tenant ID, rule hash, and AST delta into a verifiable receipt.")
}

Container(lifecycle_hook, "Lifecycle Hook Adapter", "PreToolUse / PostToolUse Dispatcher")
Container(ledger, "Audit Ledger", "Persistent Merkle chain")

Rel(lifecycle_hook, quad_reifier, "Writes raw tool call payload", "Shared Linear Memory")
Rel(quad_reifier, shape_engine, "Transfers candidate graph", "Internal Pointer")
Rel(shape_engine, datalog_evaluator, "Forwards structurally valid transitions", "In-Memory Graph")
Rel(datalog_evaluator, sparql_refusal, "Applies active authority lease context", "Graph Query")
Rel(sparql_refusal, receipt_sealer, "Emits admission status (Admitted / Refused)", "Typed Variant")
Rel(receipt_sealer, lifecycle_hook, "Returns signed execution receipt", "Shared Linear Memory")
Rel(receipt_sealer, ledger, "Streams verified receipt hash", "BLAKE3 Hash Stream")

```

### Component Workflow

1. **Action Reifier**: Receives raw tool parameters and reifies them into
   standard RDF statements (`act:target`, `act:scope`, `act:tool`,
   `act:agent`).
2. **SHACL / ShEx Shape Engine**: Guarantees architectural compliance (e.g.,
   preventing a controller layer from directly accessing an internal database
   driver).
3. **Datalog Temporal Calculus**: Enforces operational constraints including
   token quota bounds, file lock leases, and monotonic call limits.
4. **SPARQL Reactive Veto**: Evaluates targeted patterns that immediately
   trigger refusal before any computational budget is wasted.
5. **Attestation & Receipt Sealer**: Generates the final cryptographic
   envelope, certifying whether an action was admitted or refused with an
   immutable audit hash.
