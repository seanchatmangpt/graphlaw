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
struct-literal form (`Action { name, pre, pre_not, add, del }` — add
`..Default::default()` to leave `pre_not` empty; `Plan` likewise has `goal_not`) still
works and yields the same plan digest.
Admission replays the actions in order: every `pre` triple must hold, `del` is removed, then `add`
is added, and the `goal` must hold at the end. Each action yields one receipt.

## Run it

```sh
cargo run --example quickstart_plan
```

Expected output:

```text
plan-action -> blake3:8a0d0202d6e32b655448bbcd6ea3e9df0282d5ab75fc866d2370ad8d169561dd
plan-action -> blake3:c66ea90e01dd620a1678cbb36b8d7ed8a719beefd1b7fe6b1b163eb033169cb5
plan-action -> blake3:3be6530bdf3597afdcc5f60a2a7a0c15a952a1e8f8dd440e0a0bf49082e05896
final state: blake3:3be6530bdf3597afdcc5f60a2a7a0c15a952a1e8f8dd440e0a0bf49082e05896
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
- External sibling (`../ggen-marketplace`): [ABI crate how-to](../../ggen-marketplace/docs/how-to/represent-a-rust-abi-crate.md), [rust-wasi-wasmex-pack](../../ggen-marketplace/packs/rust-wasi-wasmex-pack/README.md)
