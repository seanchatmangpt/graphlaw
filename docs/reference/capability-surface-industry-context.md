# Capability-surface timings vs industry standards

How to read the [capability-surface census](../../benches/reports/capability_surface.md)
(`benches/capability_surface.rs`) against (1) GraphLaw’s own CTQs, (2) the
regulatory / statutory envelopes already receipted in-tree, and (3) published
RDF / SPARQL / DuckDB practice. Numbers below are from the Measure-lane run
embedded in that report (x86_64 linux, 4 cores, rustc 1.98.0, graphlaw
26.10.28): min-of-batches over positive-control toy requests.

> **Scope honesty.** This census is a *microbenchmark*: one small
> positive-control request per op, not BSBM / LUBM / SP²Bench / WatDiv scale.
> Industry “standards” in this space are mostly **conformance suites** and
> **methodology frameworks**, not a single published latency SLA. Where a
> published figure exists, it is cited as a *band*, not a pass/fail gate.

Companion machine receipt: [`benches/reports/capability_surface.json`](../../benches/reports/capability_surface.json).

---

## 1. How to read the units

| Measured | Human scale | Typical industry language |
|---|---|---|
| 1 000 ns | 1 µs | “sub-microsecond” (rare outside hot paths) |
| 1 000 µs | 1 ms | “sub-millisecond” / “interactive” for a single RPC |
| 1 000 ms | 1 s | “batch” / “interactive query” on large stores |

Rule of thumb used by GraphLaw’s own CTQs and statutory benches:

- **Hot path / receipt fold** — hundreds of **nanoseconds** (USL 250 ns).
- **ABI call budget (BEAM host)** — **milliseconds** (CTQ 1.3: p99 ≤ 1.2 ms).
- **Statutory / EU AI Act operational envelope** — **tens of milliseconds**
  (Art. 12/14/15 courts: 0.5–50 ms ceilings depending on the gate).

So a 100 µs SPARQL SELECT on a toy graph is *not* “slow” relative to store
benchmarks that report milliseconds; it is *not* comparable either — different
workload class.

---

## 2. GraphLaw’s own standards (the primary yardstick)

These are the normative budgets *this repo* gates or receipts against. The
capability census is Measure-lane (ranking + disclosure), not a replacement
for those courts.

| Standard | Target | Source | What the census says |
|---|---|---|---|
| CTQ 1.3 call latency | p99 ≤ **1.2 ms** | [`VOC-CTQ-TURNSTILE.md`](../jira/v26.10.28/VOC-CTQ-TURNSTILE.md) | All 14 registry ops are under 1.2 ms (heaviest: `hooks.inline-pack` ~0.89 ms). Over the budget: duckdb `close` (~1.4 ms), `open` (~11.6 ms), `exec.create` (~14.1 ms) — cold/session costs, not steady query. |
| Receipt fold USL | p99 ≤ **250 ns**, Cpk ≥ 1.33 | `tc_hot_01`, `scripts/spc_court.sh` | **Not measured here.** Census times whole ABI ops, not the fold. |
| Refusal construct | ≤ **100 ns** (PRD) | `tc_hot_01` refusal leg | **Not measured here.** |
| Art. 12 receipt issuance | p99 ≤ **500 µs** | [`sub50ms_statutory_courts.md`](../../benches/reports/sub50ms_statutory_courts.md) | Fold path historically ~125 ns; census does not re-time it. |
| Art. 14 SHACL oversight | p99 ≤ **5 ms** (statutory) | same | Census `shacl.conforming` ~**129 µs** on a tiny conforming graph — well under 5 ms, but fixture is far smaller than the statutory bench’s 64-quad candidate. |
| Art. 15 N3 traceability | p99 ≤ **30 ms** (statutory) | same | Census `n3.reason` ~**9.6 µs** on a one-rule toy — under 30 ms with large headroom; again, not the statutory fixture. |
| Hard envelope | ≤ **50 ms** strict | `statutory_boundary_50ms` | Every census leg is under 50 ms. |

**Reading CTQ 1.3 against the census.** CTQ 1.3 is a *host-call* budget for
the Smalt / BEAM runtime (S1). Registry semantic ops on the positive-control
set clear 1.2 ms comfortably. DuckDB **session open** and **CREATE exec** do
not — and that matches industry expectations for embedded OLAP engines (see
§4): connection / DDL setup is a cold cost, not a per-row hot path. Steady
`duckdb.stream.page` (~7 µs) is the relevant comparison for “keep calling.”

---

## 3. What industry actually standardizes

| Layer | What is “standard” | What is *not* standardized |
|---|---|---|
| RDF / SPARQL | W3C RDF 1.1/1.2, SPARQL 1.1 Query/Update; store methodology via **BSBM**, **LUBM**, **SP²Bench**, **WatDiv**, Trainmarks | A single µs latency SLA for `SELECT` |
| SHACL / ShEx | W3C SHACL, ShEx 2.x conformance; size/degree cliffs are product-specific | A universal validation-time budget |
| N3 / Datalog / OWL-RL | W3C N3 CG drafts; OWL 2 RL/RDF rules; OpenRuleBench / Soufflé for Datalog | Cross-engine µs courts on toy rules |
| SQL / DuckDB | SQL:2016 dialect subsets; DuckDB’s own microbenchmarks & ClickBench-class analytics | “Open connection in &lt; X µs” |
| Regulated AI ops | EU AI Act Art. 12/14/15 *obligations* (logging, oversight, accuracy) — GraphLaw maps these to **latency ceilings** in `statutory_boundary_50ms` | The Act itself does not publish ns numbers |

The capability census therefore answers: *“is every admitted op alive and in
what latency band on a fixed toy?”* Industry suites answer: *“how does the
engine scale under a named corpus?”* Both are needed; neither substitutes.

---

## 4. Per-capability reading (census → industry band)

Latencies are min-of-batches ns/op from the receipt. “Industry band” is the
order of magnitude peers report for *comparable micro workloads* (small
in-memory graphs / single statements), not BSBM Explore mixes.

| Leg | Census | Industry / peer band (micro) | Reading |
|---|---:|---|---|
| `sniff.turtle` | **2.1 µs** | Content routing / MIME sniff: typically low-µs for tiny buffers | Competitive. Pure classification; no store I/O. |
| `capabilities.default` | **11 µs** | ABI/meta introspection: usually &lt;100 µs | Fine. Registry projection, not a semantic engine. |
| `n3.reason` | **9.6 µs** | Classic N3 (cwm/EYE) often **ms+** even on small rules; modern Rust/C reasoners can be µs–ms on toys | Strong on this toy. Do **not** extrapolate to large rule sets — OpenRuleBench / N3 CG corpora are the right next court. |
| `policy.accepted` | **15 µs** | Policy engines vary widely; no shared latency standard | Healthy for a closed vocabulary accept path. |
| `parse.turtle` | **23 µs** | Oxigraph-class Turtle parse of small graphs: tens of µs–low ms; RDFLib/Jena often higher (ms) once JVM/Python overhead enters | Competitive for a tiny graph through a JSON ABI. |
| `canonical.flat` | **25 µs** | RDFC-1.0 / URDNA2015 canonicalization is usually dominated by graph size; toys can be µs–ms | Expected. Cost grows superlinearly with blank-node complexity — not visible here. |
| `convert.turtle` | **26 µs** | Parse + serialize round-trip; same band as parse on toys | Expected. |
| `shex.shexc` | **27 µs** | ShEx validators (shex.js, rudof) often **ms** on non-trivial schemas | Competitive on the toy; schema size and recursion dominate real suites. |
| `entail.simple` | **43 µs** | RDFS/OWL-RL rule materialization: µs–ms on toys; DL reasoners (HermiT/Pellet) are a different class (often ms–s) | Fine for `simple` regime. Compare like-with-like: OWL-RL rule engines, not tableau DL. |
| `datalog.ancestor` | **63 µs** | Soufflé / modern Datalog: µs–ms for tiny EDBs; OpenRuleBench is the scale yardstick | Fine for the ancestor toy. |
| `sparql.select` | **106 µs** | Oxigraph warm in-memory SELECT on small data often **~0.3–1 ms** (e.g. ~0.68 ms on a 200-triple microbench in public Oxigraph comparisons; low-cardinality patterns reported ~126–151 µs). RDFLib SPARQL on similar data often **tens–hundreds of ms**. Jena/RDF4J in-memory SELECTs are typically **ms** once JVM warm-up is paid. | **Sub-millisecond and competitive** for a tiny SELECT through the ABI. Not a BSBM claim. |
| `shacl.conforming` | **129 µs** | pySHACL / TopBraid / Jena SHACL on small conforming graphs: often **ms**; GraphLaw statutory Art. 14 ceiling is **5 ms** | Comfortably inside GraphLaw’s statutory band on this toy. Real cliffs are degree/depth/bytes (see `shacl_boundary_cliff`). |
| `law.shacl-n3` | **226 µs** | Composition of SHACL + N3 steps — no direct industry twin; nearest is “policy pipeline” | Under CTQ 1.3 (1.2 ms) with headroom. |
| `hooks.inline-pack` | **891 µs** | Knowledge-hook orchestration (SPARQL-driven packs): no shared standard | Still under CTQ 1.3. Heaviest *registry* op in the census — expected (multi-firing orchestration). |
| `duckdb.stream.page` | **7.1 µs** | DuckDB / Arrow page fetch after a prepared result: low-µs–tens of µs common in-process | **This is the steady-state DuckDB cost that matters.** Competitive. |
| `duckdb.close.session` | **1.4 ms** | Connection teardown: usually sub-ms–few ms | Acceptable. Above CTQ 1.3 if every host call closes; prefer pooled sessions (matches S1 “pooled isolation” voice). |
| `duckdb.open` | **11.6 ms** | Embedded DuckDB / JDBC-style connection open: commonly **ms–tens of ms** (allocator, extension/catalog init) | **Expected cold cost.** Do not compare to SPARQL µs. Amortize via session reuse (`mode: open` once). |
| `duckdb.exec.create` | **14.1 ms** | In-memory CREATE + catalog update: typically **ms** | Same story as open: DDL / first-statement setup, not scan throughput. GraphLaw’s analytics claim is vectorized scan/agg (authority: frozen-duckdb), measured elsewhere (`tc_hot_02`, columnar benches). |

### Visual band map (log scale)

```
ns        1e3        1e4         1e5         1e6          1e7
|----------|-----------|------------|------------|------------|
 sniff  ██
 stream ███
 n3/cap/policy ████
 parse/canon/convert/shex █████
 entail/datalog ██████
 sparql/shacl ███████          ← still << CTQ 1.3 (1.2 ms)
 law ████████
 hooks █████████              ← ~0.89 ms, under CTQ 1.3
 close ██████████             ← ~1.4 ms, just over CTQ 1.3
 open/exec ████████████████   ← cold DuckDB setup (industry-normal ms)
           ^                 ^              ^
        1 µs              100 µs         1.2 ms CTQ
```

---

## 5. Mapping to named industry frameworks

| Framework | What it measures | Relation to this census |
|---|---|---|
| **BSBM** (Berlin SPARQL Benchmark) | Explore/Update mixes over growing synthetic e-commerce RDF | Scale/throughput. Census `sparql.select` is a *single* toy SELECT — useful as a floor, not a BSBM score. |
| **LUBM** | University ontology + SPARQL over OWL-aware stores | Entailment + query. Census `entail.simple` / `sparql.select` are existence proofs only. |
| **SP²Bench** | DBLP-shaped SPARQL stress | Same caveat as BSBM. |
| **WatDiv** | Diversified SPARQL templates | Same. |
| **Trainmarks** ([datatreehouse](https://datatreehouse.github.io/trainmarks/)) | Cross-engine RDF framework board (Oxigraph, Jena, RDF4J, QLever, …) | Best *contemporary peer ranking* for store-class work. GraphLaw’s census is ABI-op latency, a different axis. |
| **OpenRuleBench** | Datalog / rule engines | Peer class for `datalog` / heavy `n3`. |
| **W3C SHACL / ShEx test suites** | Conformance, not latency | Correctness authority; latency cliffs are GraphLaw’s `shacl_boundary_cliff`. |
| **ClickBench / DuckDB microbenchmarks** | Analytical SQL scan/agg | Peer class for DuckDB *query* power — not for `open`/`CREATE` setup timed here. |
| **EU AI Act Art. 12/14/15** (as operationalized here) | Logging, oversight, accuracy obligations → GraphLaw latency ceilings | Statutory bench is the compliance reading; census shows headroom on toys. |

---

## 6. Takeaways (what the numbers mean in practice)

1. **Semantic registry ops are in the “sub-millisecond ABI” band** that CTQ 1.3
   asks for. Relative to published Oxigraph/Jena/RDFLib micro figures, the toy
   SPARQL/SHACL/parse legs look **competitive**, with the usual caveat that
   peers’ numbers are not byte-identical workloads.
2. **DuckDB cold paths (open / CREATE) are millisecond-class**, which is
   normal for embedded OLAP and **should be amortized** (session pool). The
   steady stream page (~7 µs) is the industry-fair “keep querying” number.
3. **Do not advertise these µs figures as BSBM/LUBM results.** They prove the
   surface is alive and ordered; scale claims need the named suites or
   GraphLaw’s own `tc_hot_02` / columnar / statutory benches.
4. **Regulatory reading:** every census leg is inside the 50 ms statutory
   hard envelope; SHACL/N3 toys sit far under Art. 14/15 ceilings. The
   binding compliance receipt remains `benches/reports/sub50ms_statutory_courts.md`.
5. **Internal hot-path standards (250 ns fold, 100 ns refusal) are a different
   layer** — tighter than anything industry publishes for full SPARQL/SHACL
   calls, and correctly gated by `spc_court.sh` / `tc_hot_01`, not by this
   census.

---

## 7. Reproduce

```bash
CARGO_TARGET_DIR=target-cap-bench \
CAP_BENCH_JSON=benches/reports/capability_surface.json \
CAP_BENCH_WARMUP=5 CAP_BENCH_ITERS=25 CAP_BENCH_BATCHES=10 \
cargo bench --bench capability_surface -- --nocapture
```

Re-run before citing new absolute numbers; keep the industry *bands* and the
CTQ/statutory *ceilings* as the stable interpretive frame.
