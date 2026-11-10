# How to use knowledge hooks

Derive obligations and state transitions directly from the knowledge graph: declarative `kh:` triggers + SPARQL CONSTRUCT actions, run to fixpoint over a `LawState`, with a deterministic BLAKE3-keyed firing marker recorded in the state for every firing.

Knowledge Hooks are implemented in this crate, `graphlaw::hooks` (`src/hooks.rs`); every query is parsed and executed by PurRDF's SPARQL engine. Verified against `src/hooks.rs` on 2026-11-09: there is no separate `purrdf-hooks` crate in the pinned purrdf checkout (rev `2223f89`), and the only supported `kh:kind` and handler are `"sparql"` and `handler#sparql-construct`.

---

## 1. Defining a Hook Pack

Hooks are declared as RDF resources (Turtle or TriG). A Hook Pack contains one or more `kh:Hook` definitions:

```turtle
@prefix kh: <http://seanchatmangpt.github.io/graphlaw/kh#> .
@prefix ex: <http://example.org/> .

# 1. Emit-delta hook: derive a triple per trigger row
ex:auditHook a kh:Hook ;
    kh:name "user-auditor" ;
    kh:kind "sparql" ;
    kh:query "PREFIX ex: <http://example.org/> SELECT ?u WHERE { ?u a ex:User }" ;
    kh:action ex:auditAction ;
    kh:priority 10 ;
    kh:effect "emit-delta" .

ex:auditAction a kh:Action ;
    kh:handler <http://seanchatmangpt.github.io/graphlaw/handler#sparql-construct> ;
    kh:query "PREFIX ex: <http://example.org/> CONSTRUCT { ?u ex:audited true } WHERE {}" .

# 2. Policy fuse: fail-closed refuse verdict, no delta
ex:refuseHook a kh:Hook ;
    kh:name "sanction-fuse" ;
    kh:kind "sparql" ;
    kh:query "PREFIX ex: <http://example.org/> SELECT ?e WHERE { ?e a ex:SanctionedEntity }" ;
    kh:action ex:refuseAction ;
    kh:priority 1 ;
    kh:effect "refuse" ;
    kh:reason "Sanctioned entity detected: execution refused." .

ex:refuseAction a kh:Action ;
    kh:handler <http://seanchatmangpt.github.io/graphlaw/handler#sparql-construct> ;
    kh:query "CONSTRUCT WHERE {}" .
```

A real, tested pack lives at
[`packs/self-monitoring-pack/hook.ttl`](../../packs/self-monitoring-pack/hook.ttl)
(a `kh:Hook` whose action derives `smon:EscalationObligation` from repeated grounding
questions; courts `tests/knowledge_hooks.rs` and `tests/self_monitoring_pack.rs`).

---

## 2. Capabilities Matrix

| Capability | Supported Values / Descriptions |
|---|---|
| **Trigger kind (`kh:kind`)** | `"sparql"` only (a SELECT; one firing per distinct solution row). Any other kind is a typed `Unsupported` refusal, never skipped. |
| **Action handler (`kh:handler`)** | `<http://seanchatmangpt.github.io/graphlaw/handler#sparql-construct>` only (`HANDLER_SPARQL_CONSTRUCT`). |
| **Effect (`kh:effect`)** | `"emit-delta"` (merge the CONSTRUCT into the state) or `"refuse"` (no delta; surfaces `Verdict::Refuse(reason)`, reason from `kh:reason`, default `kh:name`). Anything else is a typed refusal. |
| **Fixpoint ceilings** | Rounds repeat until no new row appears; hard ceilings `MAX_ROUNDS = 64`, `MAX_FIRINGS = 10_000`, `MAX_STATE_QUADS = 1_000_000` are refused, never truncated. Hooks run in ascending `kh:priority` (ties by IRI). |
| **Lineage markers** | Each firing records `<urn:graphlaw:hook-firing:blake3> a kh:Firing ; kh:hook <hook>` in the state, so a row fires at most once per state lineage (re-materializing a saturated state changes nothing). |

SQL, SHACL, `threshold`/`window` triggers and Block/Annotate/Repair enforcement modes are
not implemented in `graphlaw::hooks`; SQL-in-RDF belongs to the `duckdb` op
(`docs/how-to/document-sequences.md`), not to hooks.

---

## 3. Rust Execution API

```rust,ignore
use graphlaw::hooks::{HookPack, Verdict};

// 1. Load the pack from a LawState holding the kh:Hook / kh:Action resources
let pack = HookPack::load(&pack_state)?;

// 2. Materialize to fixpoint over a state
let result = pack.materialize(&state)?;

println!("Rounds executed: {}", result.rounds);
println!("Firings recorded: {}", result.firings.len());

// 3. Inspect verdicts (one per firing, same order as result.firings)
for v in &result.verdicts {
    match &v.verdict {
        Verdict::Fired => println!("{} fired in round {}", v.hook, v.round),
        Verdict::Refuse(reason) => eprintln!("refused by {}: {}", v.hook, reason),
    }
}
// result.state is the saturated LawState
```

A refuse is never an error at `materialize`: the consumer decides what a refusal means.
Emit-delta packs are unaffected.

---

## 4. Verification & Testing

- `tests/knowledge_hooks.rs` — 11 `#[test]` functions (hook rounds, fixpoint, priority
  chaining, SPARQL counting of `smon:EscalationObligation`s, the refuse effect, typed
  refusal of unsupported kind/effect/handler). Re-run 2026-11-09 (`cargo test --test knowledge_hooks`, exit 0):
  `11 passed; 0 failed`.
- `tests/self_monitoring_pack.rs` — the self-monitoring pack end to end.
- Run:
  ```bash
  cargo test --test knowledge_hooks
  cargo test --test self_monitoring_pack
  ```
