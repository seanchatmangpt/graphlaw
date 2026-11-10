# Edge Sovereignty & Zero-Trust Semantic Compaction

At the extreme edge—whether on industrial IoT controllers, aerospace and maritime vehicles, clinical medical devices, or browser client sandboxes—systems generate vast, continuous event telemetry streams.

Traditional systems suffer from a severe architectural tension:
1. **Full Telemetry Streaming to Cloud**: Raw logs are continuously streamed to centralized triple-stores or warehouses. This incurs prohibitive bandwidth costs, introduces latency, causes battery drain, introduces privacy risks, and fails entirely in denied, degraded, intermittent, or limited (DDIL) connectivity environments.
2. **Naive Edge Summarization**: Edge nodes use ad-hoc shell scripts or unverified Python jobs to emit coarse statistics. This strips away provenance, breaks causal event lineage, and renders regulatory compliance (such as FDA 21 CFR Part 11 or EU Annex 11) mathematically impossible to verify.

GraphLaw with embedded `frozen-duckdb` introduces a sovereign third paradigm: **Local, Sovereign Analytical Ingestion with Zero-Trust Semantic Compaction**.

---

## 1. Zero-Trust Semantic Compaction

Rather than transmitting every raw reading across the network, GraphLaw compresses event streams **semantically and cryptographically**:

* **Local Ingestion Layer**: Raw readings (temperatures, vibrations, network
  telemetry, OCEL v2 logs) are appended to high-throughput, columnar in-memory
  buffers or local Parquet partitions managed by `frozen-duckdb`. Throughput is
  workload-dependent; no fixed append-latency or row-count figure is asserted.
* **Declarative Reduction**: At defined window boundaries, GraphLaw evaluates declarative SQL-in-RDF ASTs over the local partition. Zone maps and dictionary encoding allow instantaneous columnar aggregations without full decompression.
* **Cryptographic Emission**: Instead of streaming multi-megabyte log files, GraphLaw materializes a compact set of RDF state transitions sealed with a **BLAKE3 execution receipt**.
* **Bandwidth Multiplier**: Event bursts reduce to signed RDF triples whose size and
  compaction ratio are workload-dependent; no fixed ratio is asserted. The
  court-witnessed reduction is `tests/sieve_court.rs`: a 10^6-row in-engine GROUP BY
  lands inside the evaluator's default 64-row result envelope, and result sets over
  the envelope refuse as a typed `ResourceLimit` (`duckdb_result_rows`).

---

## 2. Hardened Edge Boundaries & Deterministic Memory

Running complex analytical engines inside edge containers or WASM sandboxes has historically been plagued by out-of-memory crashes. Status below as of 2026-11-09:

* **Single WASM unit**: GraphLaw's core and `frozen-duckdb` link into one module, `graphlaw-wasm`. As of 2026-11-09 the pinned artifact is 62,909,537 bytes (~60.0 MiB on disk, `registry/ARTIFACTS.blake3`). The size is the file, not a memory bound: `graphlaw-run-eh` (wasmtime 48) caps the request frame (`WALL_BYTES`, 64 MiB) and the outstanding allocation (`MAX_OUTSTANDING_ALLOC_BYTES`, 16 MiB), but provides **no linear-memory ceiling** (`docs/WASM.md`, sandbox qualification section).
* **Deterministic Allocation**: The `frozen-duckdb` buffer manager enforces hard memory accounting on operator pipelines (HashJoin, AggregateHashTable), avoiding swap thrashing or allocator aborts inside edge micro-VMs.
* **Zero-Copy Arrow ABI**: Direct `ValueRef` inspection allows moving values directly from vector chunks into typed RDF literals (`TermValue::Literal`) without intermediate string parsing, double-buffering, or memory fragmentation.

---

## 3. Disconnected / DDIL Fail-Closed Sovereignty

Edge units often operate with zero connectivity for days or weeks (e.g., cold-chain transit containers, autonomous underwater vehicles, remote clinical devices).

GraphLaw guarantees **local fail-closed sovereignty**:

```text
[ Sensor Excursion Detected ]
              │
              ▼
[ Local frozen-duckdb Aggregation ]
              │
              ▼
[ PurRDF N3 / Datalog Rule Evaluation ]
              │
      ┌───────┴───────┐
      ▼               ▼
[ Compliant ]   [ Out of Spec ]
      │               │
  Emits PASS      Locks Physical Actuator / Shuts Down Transit
  Receipt         Asserts Quarantined State Locally
                  Syncs Tamper-Proof Audit Proof Upon Reconnection
```

* **No Remote Coordinator Needed**: The full deductive logic, schema validation (SHACL), and columnar verification run entirely inside the local node.
* **Immediate Local Enforcement**: Safety breaches or excursions trigger instant actuator lockouts, physical quarantines, or alarm states without awaiting cloud consensus.
* **Anti-Injection SHACL Pre-Flight**: Malformed queries, attempts to bypass bounding clauses (WHERE clause omissions), or invalid operators are rejected by GraphLaw's structural SHACL gate prior to query dispatch.
