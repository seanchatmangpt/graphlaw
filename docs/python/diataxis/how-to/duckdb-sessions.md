# How to use the `duckdb` op

The op has four modes. Named sessions live in the runner process only
([state and persistence](../explanation/state-and-persistence.md)).
<!-- cap: duckdb_mode:open -->
<!-- cap: duckdb_mode:exec -->
<!-- cap: duckdb_mode:stream -->
<!-- cap: duckdb_mode:close -->

```python
gl.duckdb_open("s1")                        # mode open; same name twice is refused (DuckDb)
gl.duckdb_exec(session="s1", query_ast=..., ast={"turtle": ...})   # mode exec
gl.duckdb_stream(cursor=...)                # mode stream
gl.duckdb_close(session="s1")               # mode close; unknown handles are idempotent-ok
```

`exec` without `session` uses a fresh in-memory connection. Failures surface as
`kind == "DuckDb"`.

## See Also

[Python API](../reference/api.md)
