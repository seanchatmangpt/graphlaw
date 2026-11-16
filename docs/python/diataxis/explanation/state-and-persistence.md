# State and persistence

The runner (`graphlaw-run-eh`) is sterile by design: an empty WASI environment, only clock,
random and stdio capabilities, and no preopened directories. Consequences:

- **Within a process:** the wasm module keeps named DuckDB sessions and cursors in memory.
  Opening a session name twice is refused.
- **Across processes:** nothing carries over. A new runner has no sessions; probed by the
  tests (`duckdb_open("s1")` succeeds again in a fresh process).
- **What is durable:** receipt files written by `graphlaw-attest` / the receipt store, and
  whatever your agent writes. Pass graph state in each request (`law` takes `data`), and persist
  the returned `nquads` and receipts yourself.

## See Also

[DuckDB sessions](../how-to/duckdb-sessions.md) | [Receipts](../how-to/receipts-and-attestation.md)
