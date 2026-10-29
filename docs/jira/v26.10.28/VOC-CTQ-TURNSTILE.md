# VOC-CTQ-TURNSTILE

Voice-of-Customer → Critical-to-Quality translation for graphlaw v26.10.28,
as dictated by the operator. The turnstile: raw operator voice in, numbered
CTQs out; every downstream work order in [WORK-ORDERS.md](WORK-ORDERS.md)
cites a CTQ id here.

## Stakeholder segmentation

| Segment | Voice (raw) | What they actually buy |
| --- | --- | --- |
| S1 — Smalt BEAM runtime | Run the kernel inside the Elixir/BEAM host without corrupting the VM; no runaway reduction, no heap creep across calls | Bounded, restartable in-process execution |
| S2 — A2A agents | Same artifact on host and wasm; predictable refusals; receipts they can replay | Deterministic admission + replayable receipts |
| S3 — Platform / security governance | Nothing executes that was not admitted; nothing admitted that cannot be refused | Fail-closed law, typed refusals |
| S4 — CFO / GC compliance | Every consequential act is append-only and traceable to authority | Receipt integrity + axiom/lease traceability |
| S5 — Core dev | Build the Elixir consumer in-tree; no hidden host imports; one canonical forked source | In-tree `graphlaw_ex`, zero host imports, rev-pinned vendor |

## CTQ matrix

### CTQ 1 — Smalt BEAM runtime (S1)

| id | CTQ | Target |
| --- | --- | --- |
| 1.1 | Preemption-safe wasm execution | ≤1000 reductions per call, or pooled isolation so no call starves the scheduler |
| 1.2 | Zero post-call heap growth | Cpk ≥ 1.67 on heap-delta bytes/call (P28-SIL program) |
| 1.3 | Call latency | p99 ≤ 1.2 ms |

### CTQ 2 — Admission law (S2, S3)

| id | CTQ | Target |
| --- | --- | --- |
| 2.1 | Fail-closed admission | 0 false admissions across the refusal corpus; every unknown shape refuses |
| 2.2 | Proof verification per evaluation | Crypto (Ed25519) proof check performed on every evaluated receipt |

### CTQ 3 — SQL-over-RDF (S2, S5)

| id | CTQ | Target |
| --- | --- | --- |
| 3.1 | ANSI SQL over RDF | Compiled and evaluated via embedded frozen-duckdb, in-process |
| 3.2 | Dialect parity | All 35 PEG statement kinds pass on real frozen-duckdb |

### CTQ 4 — Receipts and traceability (S3, S4)

| id | CTQ | Target |
| --- | --- | --- |
| 4.1 | Append-only receipts | 100% of consequential acts emit an append-only receipt |
| 4.2 | Traceability | Every receipt traceable to axioms (`courts/code-partition/axioms.blake3`) and leases |

### CTQ 5 — graphlaw_ex embed (S5)

| id | CTQ | Target |
| --- | --- | --- |
| 5.1 | In-tree build | `graphlaw_ex` builds in-tree (`mix` green) |
| 5.2 | Zero host imports | 0 host imports in the wasm module surface consumed by graphlaw_ex |
| 5.3 | Rev-pinned canonical source | Forked source (purrdf-n3 vendor) pinned to an exact rev |

## Kano classification

| Class | Features |
| --- | --- |
| Must-be (absence = exit) | Fail-closed admission (2.1); scheduler isolation / preemption safety (1.1); zero host imports (5.2) |
| One-dimensional (more is better) | Latency (1.3); concurrency scaling under pooling (1.1 pooling path) |
| Delighter (absence unnoticed, presence sells) | In-process SQL-over-RDF (3.1, 3.2); P28-SIL balance-sheet regularization (1.2 Cpk discipline) |

Must-be features are entry conditions, never selling points; a must-be gap is
a BLOCKED standing on any dependent work order. Delighters are claimed only
with a receipt — an unreceipted delighter is UNVERIFIED, never ALIVE.
