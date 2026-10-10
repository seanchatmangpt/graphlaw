# How to use knowledge hooks

Derive obligations from the state graph itself: declarative `kh:` triggers + SPARQL
actions, run to fixpoint over a `LawState`, every firing recorded in the state.

## Define a hook pack

Hooks live in any RDF pack — see the real one at
[`packs/self-monitoring-pack/hook.ttl`](../../packs/self-monitoring-pack/hook.ttl):
a `kh:Hook` with `kh:kind "sparql"` trigger, a `kh:action` whose SPARQL CONSTRUCT
derives e.g. `smon:EscalationObligation` from repeated grounding questions.

## Load and run

```rust,ignore
use graphlaw::hooks::HookPack;

let pack = HookPack::load(&pack_state)?;      // kh:Hook / kh:Action resources
let materialized = pack.materialize(&state)?; // runs to fixpoint; each firing recorded
```

## The refuse effect

An action may set `Effect::Refuse` instead of emitting a delta: the run then surfaces
`Verdict::Refuse(reason)` carrying the hook's `kh:reason` (default when absent). A refuse
is never an error at materialize — the consumer decides what a refusal means. Emit-delta
packs are unaffected.

## See it live

`tests/knowledge_hooks.rs` — 11 tests, verified 2026-10-10 (`11 passed; 0 failed`):
hook rounds, SPARQL counting of `smon:EscalationObligation`s, and the refuse effect
end to end.