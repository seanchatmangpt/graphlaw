# HANDWRITTEN

Standing: UNVERIFIED

Ledger of every hand-written file under `graphlaw_ex/lib`. Projections are listed by
`scripts/ggen_sync.sh --list-projections` and are never hand-edited.

## Kernel (irreducible residue)

| Path | Reason |
|---|---|
| lib/graphlaw_ex/host.ex | wasmex host process, engine calls |
| lib/graphlaw_ex/pool.ex | instance pool |
| lib/graphlaw_ex/pool/leases.ex | lease bookkeeping |
| lib/graphlaw_ex/engine_load.ex | engine artifact loading and pin check |
| lib/graphlaw_ex/blake3.ex | blake3 binding |
| lib/graphlaw_ex/engine_load/blake3.ex | blake3 for engine load |
| lib/graphlaw_ex/application.ex | OTP supervision tree |
| lib/graphlaw_ex/kill_switch.ex | runtime kill switch |
| lib/graphlaw_ex/kill_switch/server.ex | supervised kill-switch restore and serialized install |
| lib/graphlaw_ex/kill_switch/store.ex | durable atomic store of signed revocation documents |
| lib/graphlaw_ex/engine_load/single_flight.ex | single-flight engine compile/cache |
| lib/graphlaw_ex/context.ex | non-authoritative caller context and lease resolver behaviour |
| lib/graphlaw_ex/abi.ex | ABI framing and op transport |
| lib/graphlaw_ex/telemetry.ex | telemetry, redact and admit logic |
| lib/graphlaw_ex/wasm_config.ex | wasm path resolution logic |
| lib/graphlaw_ex/capability/coerce.ex | request coercion |
| lib/graphlaw_ex/capability/decode.ex | response decoding |
| lib/graphlaw_ex/capability/graph.ex | graph input handling |
| lib/graphlaw_ex/capability/sql.ex | SQL-in-RDF input handling |
| lib/graphlaw_ex/capability/sparql_query.ex | SPARQL query building |
| lib/graphlaw_ex/result.ex | result decoding dispatch |
| lib/mix/tasks/graphlaw_ex.doctor.ex | mix task |
| lib/mix/tasks/graphlaw_ex.verify.ex | mix task |
| lib/mix/tasks/graphlaw_ex.fetch.ex | mix task |

## Table-projectable (candidates, currently hand-written)

| Path | Source of truth |
|---|---|
| lib/graphlaw_ex/refusal.ex | refusal codes in registry/governance ontology |
| lib/graphlaw_ex/error.ex | error classes |
| lib/graphlaw_ex/error/class.ex | error classes |
| lib/graphlaw_ex/error/refused.ex | refusal struct |
| lib/graphlaw_ex/authority.ex | authority ceilings (governance ontology) |
| lib/graphlaw_ex/config.ex | config keys |
| lib/graphlaw_ex/registry.ex | registry accessors |
| lib/graphlaw_ex/introspection.ex | derived from registry |
| lib/graphlaw_ex/capability/dialect.ex | dialect table |
| lib/graphlaw_ex/capability/refusal.ex | per-op refusal table |
| lib/graphlaw_ex.ex | public facade |
| lib/graphlaw_ex/authority/table.ex | authority ceiling table |
| lib/graphlaw_ex/capability/dialect/tiers.ex | dialect tier table |
| lib/graphlaw_ex/capability/dispatch.ex | op dispatch |
| lib/graphlaw_ex/capability/extension.ex | extension table |
| lib/graphlaw_ex/config/keys.ex | config key table |
| lib/graphlaw_ex/error/table.ex | error table |
| lib/graphlaw_ex/introspection/tables.ex | introspection tables |
| lib/graphlaw_ex/pool/workers.ex | pool worker processes |
| lib/graphlaw_ex/refusal/table.ex | refusal code table |
| lib/graphlaw_ex/result/admitted.ex | admitted result struct |
| lib/graphlaw_ex/result/receipt.ex | receipt result struct |
| lib/graphlaw_ex/telemetry/catalog.ex | telemetry event catalog |

## Unsupported by generator (hand-written, no projection path yet)

| Path | Class | Reason |
|---|---|---|

Kill-switch files: `lib/graphlaw_ex/kill_switch.ex`, `kill_switch/server.ex` and
`kill_switch/store.ex` (all ledgered above under Kernel).
