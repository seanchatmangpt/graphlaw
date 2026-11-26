# Quickstart

A tutorial: one path from a clean checkout to an admitted plan and a read receipt. For the
same path with explanations, see [first-law-admission](tutorials/first-law-admission.md).
Needs Rust 1.98 (`rust-toolchain.toml`).

## 1. Build

```sh
cargo check
```

## 2. Admit a plan

```sh
cargo run --example quickstart_plan
```

Output is three `plan-action -> blake3:<64 hex>` lines and a `final state: blake3:<64 hex>`
line that equals the last `plan-action` hash. The three receipt hashes begin `8a0d0202`,
`c66ea90e`, `3be6530b` (asserted at `tests/readme_first_ten_minutes.rs:24`).

The plan is built with typed builders and admitted in two calls
(`examples/quickstart_plan.rs:14`):

```rust
let plan = Plan::builder()
    .action("open").requires(is("urn:v:closed")?).adds(is("urn:v:open")?).deletes(is("urn:v:closed")?)
    .goal(is("urn:v:open")?).build()?;
let admitted = plan.admit(&start)?;
```

## 3. Read the receipt

Each element of `admitted.receipts` is a `Receipt` (`src/law.rs:172-189`): `parent` and
`child` state ids, `step` (`plan-action`), `added` quad count, and `plan_blake3`. The `child`
of one receipt is the `parent` of the next.

## 4. See a refusal

```sh
cargo run --example refused_plan
```

```text
REFUSED at index 1, action `paint`
  unmet: <urn:d:door> <urn:p:has> <urn:v:key> .
```

No receipt is produced for a refused plan. Codes and recovery steps: [refusals.md](refusals.md).

## 5. Go further

```sh
cargo run --example receipt_feedback
cargo run --example policy_admission
cargo run --example sql_in_rdf_duckdb
```

`receipt_feedback` shows an admission requiring a recorded receipt; `policy_admission` runs
under the default `abi` feature; `sql_in_rdf_duckdb` compiles a SQL AST written in Turtle and
runs it on embedded frozen-duckdb (a `REFUSED-01..04` code in its step 3 means the AST failed
the pre-flight gate). Their outputs are asserted or printed by the sources in `examples/`;
this page does not reproduce them.

## See Also

- [First law admission](tutorials/first-law-admission.md)
- [Docs map](index.md)
- [Refusals](refusals.md)
- [How to verify without trusting the operator](how-to/verify-without-trusting-the-operator.md)
