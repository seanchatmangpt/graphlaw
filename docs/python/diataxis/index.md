# GraphLaw for Python coding agents

Diataxis documentation for driving **every** GraphLaw capability from Python, aimed at coding
agents. The `graphlaw` package (in `python/graphlaw/`) wraps the real binaries; it adds no new
semantics. Every capability is machine-checked: see
[the capability gate](explanation/combinatorial-maximalism.md).

| Quadrant | Page | Use it to |
|---|---|---|
| Tutorial | [First session](tutorials/01-first-session.md) | start a runner and call your first ops |
| Tutorial | [Admit a plan with receipts](tutorials/02-admit-a-plan-with-receipts.md) | run a `law` plan end to end and verify it |
| How-to | [Query and validate](how-to/query-and-validate.md) | sniff, parse, convert, canonical, sparql, shacl, shex |
| How-to | [Reason and derive](how-to/reason-and-derive.md) | entail, n3, datalog, hooks, policy |
| How-to | [Govern with law steps and leases](how-to/govern-with-law-steps.md) | all 10 steps, signed leases |
| How-to | [Receipts and attestation](how-to/receipts-and-attestation.md) | sign, verify, require signed receipts |
| How-to | [DuckDB sessions](how-to/duckdb-sessions.md) | the `duckdb` op |
| How-to | [Handle refusals](how-to/handle-refusals.md) | typed refusals as agent feedback |
| How-to | [Stay inside the limits](how-to/stay-inside-the-limits.md) | the 19 resource limits |
| How-to | [Give an agent GraphLaw](how-to/give-an-agent-graphlaw.md) | register tools, run the loop |
| Reference | [Python API](reference/api.md) | signatures and behavior |
| Reference | [Capability matrix](reference/capability-matrix.md) | generated; every capability and its status |
| Explanation | [Combinatorial maximalism](explanation/combinatorial-maximalism.md) | why nothing is left out |
| Explanation | [State and persistence](explanation/state-and-persistence.md) | what lives where |

## Prerequisites

```bash
cargo build --bins --example blackbox_fixture --features runner-eh   # runner, verify, attest, registry
export PYTHONPATH=/Users/sac/graphlaw/python
export GRAPHLAW_WASM=/Users/sac/graphlaw/priv/graphlaw.wasm
```

## See Also

[ABI reference](../../abi-reference.md) | [Refusals](../../refusals.md) |
[Capability registry](../../capability-registry.md)
