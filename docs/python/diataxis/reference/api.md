# Python API reference

Package `graphlaw` (`python/graphlaw/`). Paths default to `target/debug/*` in this checkout;
override with `GRAPHLAW_RUN_EH`, `GRAPHLAW_VERIFY`, `GRAPHLAW_ATTEST`, `GRAPHLAW_REGISTRY`,
`GRAPHLAW_WASM`.

## `GraphLaw(runner=None, wasm=None, expect_blake3=None, epoch_ticks=None)`

Context-managed session over one `graphlaw-run-eh` process. `call(request)` sends any ABI
request and returns the result or raises `GraphLawError`.

| Method | ABI op |
|---|---|
| `capabilities()` | `capabilities` |
| `sniff(text, hint=None)` | `sniff` |
| `parse(text, dialect=None, **kw)` | `parse` |
| `convert(text, to, dialect=None, **kw)` | `convert` |
| `canonical(data)` | `canonical` |
| `sparql(data, query)` | `sparql` |
| `shacl(data, shapes)` | `shacl` |
| `shex(data, schema, map, schema_dialect=None)` | `shex` |
| `n3(text)` | `n3` |
| `entail(data, regime)` | `entail` |
| `datalog(rules, facts)` | `datalog` |
| `hooks(data, pack)` | `hooks` |
| `law(data, steps, **lease_kw)` | `law` |
| `policy(problem, policy)` | `policy` |
| `duckdb_open/exec/stream/close` | `duckdb` modes |
| `op(name, **fields)` | any op |

## Errors

`GraphLawError(.error, .kind)` for ABI refusals; `RunnerExit(.code, .stderr)` when the runner
process exits (typed codes, e.g. 84 digest mismatch, 92 oversize frame).

## Modules

| Module | Contents |
|---|---|
| `graphlaw.steps` | builders for the ten `law` steps |
| `graphlaw.attest` | `pubkey`, `sign`, `verify`, `sign_lease`, `sign_receipt`, `receipt_ntriples` |
| `graphlaw.verify` | `verify_receipts(start, plan, receipts, trusted_keys)` |
| `graphlaw.tools` | `tool_specs()`, `dispatch(gl, name, args)` |
| `graphlaw.registry` | `load_registry`, `load_op_examples`, `duckdb_modes`, `run_registry_cli` |
| `graphlaw.coverage` | the capability gate (`python3 -m graphlaw.coverage`) |

## See Also

[Capability matrix](capability-matrix.md) | [ABI reference](../../../abi-reference.md)
