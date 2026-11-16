# How to give a coding agent all of GraphLaw

`graphlaw.tools` generates one JSON-schema tool per capability from the registry, so the tool
list cannot drift from the kernel.

```python
from graphlaw import GraphLaw, tools
specs = tools.tool_specs()            # register these with your agent runtime
with GraphLaw() as gl:
    out = tools.dispatch(gl, "graphlaw_sparql", {"data": text, "query": "ASK { ?s ?p ?o }"})
    # {"ok": True, "result": ...}  or  {"ok": False, "error": {kind, message, details}}
```

Refusals come back as data, so the agent loop can repair and retry.

## Tools
<!-- cap: tool:graphlaw_capabilities -->
<!-- cap: tool:graphlaw_sniff -->
<!-- cap: tool:graphlaw_parse -->
<!-- cap: tool:graphlaw_convert -->
<!-- cap: tool:graphlaw_canonical -->
<!-- cap: tool:graphlaw_sparql -->
<!-- cap: tool:graphlaw_shacl -->
<!-- cap: tool:graphlaw_shex -->
<!-- cap: tool:graphlaw_n3 -->
<!-- cap: tool:graphlaw_entail -->
<!-- cap: tool:graphlaw_datalog -->
<!-- cap: tool:graphlaw_hooks -->
<!-- cap: tool:graphlaw_law -->
<!-- cap: tool:graphlaw_policy -->
<!-- cap: tool:graphlaw_duckdb -->

| Tool | Capability |
|---|---|
| `graphlaw_capabilities` | `capabilities` op |
| `graphlaw_sniff` | `sniff` op |
| `graphlaw_parse` | `parse` op |
| `graphlaw_convert` | `convert` op |
| `graphlaw_canonical` | `canonical` op |
| `graphlaw_sparql` | `sparql` op |
| `graphlaw_shacl` | `shacl` op |
| `graphlaw_shex` | `shex` op |
| `graphlaw_n3` | `n3` op |
| `graphlaw_entail` | `entail` op |
| `graphlaw_datalog` | `datalog` op |
| `graphlaw_hooks` | `hooks` op |
| `graphlaw_law` | `law` op |
| `graphlaw_policy` | `policy` op |
| `graphlaw_duckdb` | `duckdb` op |

Operator surfaces that are not ABI ops (`graphlaw-verify`, `graphlaw-attest`,
`graphlaw-registry`) are Python functions: `verify_receipts`, `graphlaw.attest.*`,
`graphlaw.registry.run_registry_cli`. Give the agent those as wrapper tools if it needs them.

## Agent loop

1. Register `tool_specs()`.
2. On each tool call run `dispatch`; feed `error.kind` and `error.details` back verbatim.
3. Gate the work with the capability gate before you trust a new runner build:
   `PYTHONPATH=python python3 -m graphlaw.coverage`.

## See Also

[Handle refusals](handle-refusals.md) | [Python API](../reference/api.md)
