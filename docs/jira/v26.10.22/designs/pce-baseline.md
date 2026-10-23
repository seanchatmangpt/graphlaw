# PCE baseline — charter "<2.8%" REFUTED at the measured operating point (DFLSS Measure-lane B record)

Full 25M-row hive-partitioned parquet corpus (COPY-generated, warm cache,
M3 Max 16 cores, duckdb v2.0.0-dev88595, no throttling). Real prepared raw
floor: prepare ONCE, 5 timed re-executions with typed binds (the earlier
smoke's -2.2%..+4.3% range diffed against a per-rep re-parsed,
literal-inlined query_row and compared p50s — not a fair floor).

## Witnessed (min-of-5, TC_HOT_FULL=1)
- Compiled path (eval_ast_query): p50=12927µs, p95=13847µs, min=12384µs
- Raw prepared floor: p50=8823µs, min=8714µs
- **PCE = 70.36%** (engine 8714 ÷ pipeline 12384)
- **Substrate overhead = 42.12%** — charter's "<2.8%" REFUTED at this
  operating point
- Scanned throughput ~1.92B rows/s vs the TC-HOT-02 budget of >12M rows/s
  (~160x the budget floor; the
  overhead is in the substrate's receipt + triple projection + row
  materialization, none of which the raw path does)

## Honest caveats (carried in the bench output)
- Warm cache, min-of-5 (means under contention not reported as PCE)
- Compiled path includes receipt-hash + triple projection + row
  materialization; raw path includes none — the comparison is pipeline vs
  engine, which is the PRD's own framing
- ONE query shape (WHERE + GROUP BY), 32 emitted rows: selectivity may
  dominate the projection cost. Next rung if the claim is to be rescued:
  vary rows_emitted/selectivity, not rerun this command.

## Implications
- The PRD's "zero hot-path latency" framing holds for the evidentiary fold
  (0.19-0.22µs) but NOT for the full eval pipeline at 25M rows — the
  42% is projection/materialization cost, pointing at the optimization
  surface (lazy projection, materialization bypass for count-style
  aggregates) rather than the receipt layer.
- TC-HOT-02's >12M rows/sec READ throughput is met (~1.9B scanned/s); the
  budget-row conflict (read-throughput vs pipeline-wall-time) needs the
  PRD to pick the metric it means.
