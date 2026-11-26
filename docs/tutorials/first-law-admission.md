# First law admission

A tutorial. You will build GraphLaw, admit a three-action plan over an RDF state, read the
receipt chain it produces, and then watch a bad plan get refused. It takes about ten minutes
and needs Rust 1.98 (`rust-toolchain.toml`; `rust-version = "1.98"` in `Cargo.toml`).

All commands run from the repository root. Every example below is a file in `examples/`; the
tests in `tests/readme_first_ten_minutes.rs` run them as subprocesses and assert their output.

## 1. Build

```sh
cargo check
```

`cargo check` compiles the crate with its default feature `abi` (`Cargo.toml`:
`default = ["abi"]`). A clean exit means you are ready.

## 2. Admit a plan

Open `examples/quickstart_plan.rs`. The start state is one triple, a door that is closed:

```rust
let start = LawState::parse(b"<urn:d:door> <urn:p:is> <urn:v:closed> .", Dialect::NTriples, None)?;
```

The plan has three actions (`open`, `close`, `lock`). Each names a triple it `requires`, one it
`adds`, one it `deletes`. The goal is that the door is locked:

```rust
let admitted = plan.admit(&start)?;
```

`admit` replays the actions in order. Each action's required triple must hold in the current
state, then `deletes` is removed and `adds` is added. After the last action the goal must hold.
Run it:

```sh
cargo run --example quickstart_plan
```

You should see four lines: three `plan-action -> blake3:<64 hex>` lines and one
`final state: blake3:<64 hex>` line. The test pins the leading hash characters of the three
receipts as `8a0d0202`, `c66ea90e` and `3be6530b` (`tests/readme_first_ten_minutes.rs:24-35`),
so your first three hashes should begin with those.

## 3. Read the receipt

`admitted.receipts` holds one `Receipt` per action (constructed at `src/plan.rs:606-617`). The
example prints two of its fields. All of them are defined at `src/law.rs:172-190`:

| Field | Meaning in this run |
|---|---|
| `parent` | id of the state the action started from |
| `child` | id of the state the action produced (the hash you saw) |
| `step` | the string `plan-action` |
| `added` | net change in quad count (`child` minus `parent`, floored at 0) |
| `plan_blake3` | digest of the whole plan, the same on every receipt in the chain |
| `lease_id`, `subject_blake3` | `None` here; no lease or external subject was supplied |

Two things follow from the table. First, the chain links: each receipt's `parent` is the previous
receipt's `child`, and the `final state:` line equals the last receipt's `child`. Second, every
hash is a function of the state content, so running the example again prints the same lines.

To see more fields, add this loop after the existing one in a scratch copy of the example (do
not edit the shipped file):

```rust
for r in &admitted.receipts {
    println!("{} added={} plan={:?}", r.step, r.added, r.plan_blake3);
}
```

## 4. Refuse a plan

```sh
cargo run --example refused_plan
```

The second action, `paint`, requires a triple no earlier action produced. Output:

```text
REFUSED at index 1, action `paint`
  unmet: <urn:d:door> <urn:p:has> <urn:v:key> .
```

The refusal is a typed value, `LawError::PlanRefused { index, action, missing, .. }`
(`examples/refused_plan.rs:40-50`). No state and no receipt exist for a refused plan.

## What you built

You admitted a plan, saw one content-addressed receipt per action, and saw a plan refused with
the unmet triple named. Next steps are in the How-to quadrant of the docs map.

## See Also

- [Quickstart](../quickstart.md) — the shorter command-only version
- [How to verify without trusting the operator](../how-to/verify-without-trusting-the-operator.md)
- [Receipt fields explained](../explanation/receipt-fields.md)
- [Refusals register](../refusals.md)
- [Docs map](../index.md)
