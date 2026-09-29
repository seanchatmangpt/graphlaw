# GraphLaw quickstart

Admit a plan over an RDF state and get a content-addressed receipt chain. Needs Rust 1.96+.
The runnable source is `examples/quickstart_plan.rs`.

## The core

Build the plan with typed builders (no hand-written N-Triples), then admit it:

```rust
let is = |v| Triple::iri("urn:d:door", "urn:p:is", v);
let plan = Plan::builder()
    .action("open").requires(is("urn:v:closed")?).adds(is("urn:v:open")?).deletes(is("urn:v:closed")?)
    .goal(is("urn:v:open")?).build()?;
let admitted = plan.admit(&start)?;   // replays every action; refuses on the first unmet precondition
```

`Triple::iri` validates IRIs (refusing `<`, `>`, spaces, control characters) and `Triple::literal`
escapes quotes, backslashes and newlines, so a literal can never inject a second triple. The
struct-literal form (`Action { name, pre, add, del }` with N-Triples text) still works and yields the
same plan digest.
Admission replays the actions in order: every `pre` triple must hold, `del` is removed, then `add`
is added, and the `goal` must hold at the end. Each action yields one receipt.

## Run it

```sh
cargo run --example quickstart_plan
```

Expected output:

```text
plan-action -> sha256:e51e3e15db993169ecf5d2171f5bc81b5febb81b2c5b8771a26c6537c11782a2
plan-action -> sha256:01c373eab6e2c7078f39ed3d16d267c7349a6ee747dac8abb2b4e05c75376b72
plan-action -> sha256:77b8d7c989b2717c1300a597db07c2a60722156183ce7e14017aec23e8d033d7
final state: sha256:77b8d7c989b2717c1300a597db07c2a60722156183ce7e14017aec23e8d033d7
```

Each line is a receipt's child state id; the last equals the final state id.

## A refused plan

```sh
cargo run --example refused_plan
```

```text
REFUSED at index 1, action `paint`
  unmet: <urn:d:door> <urn:p:has> <urn:v:key> .
```

No state is produced on refusal. See `docs/refusals.md`.

## Receipt feedback

```sh
cargo run --example receipt_feedback
```

```text
before record: receipt required: no recorded receipt for step `derive:rdfs`
after record:  admitted by `admit:require-receipt`
```

## Policy admission (feature `abi`)

```sh
cargo run --example policy_admission --features abi
```

```text
ADMITTED reachable=["g", "s0"] goals=["g"] entries=[("s0", "flip")]
```

Without `--features abi` the example prints a hint and exits.

## See also

- `docs/refusals.md`, `CHANGELOG.md`, `SECURITY.md`
