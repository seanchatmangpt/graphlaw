# Eyeron

**Eyeron** combines **Eye** with the sound of **iron**, reflecting explainable reasoning and its place in the Eyereasoner family.

Eyeron is a Rust reasoner for **Notation3 (N3)**, turning facts and rules into conclusions with verifiable proofs. It can be used as a command-line program, embedded as a Rust library, or run in a browser through WebAssembly.

> [!TIP]
> [Try Eyeron in the browser](https://eyereasoner.github.io/eyeron/playground).

Read [how Eyeron reasons](https://github.com/eyereasoner/eyeron/blob/main/docs/reasoning.md)
for what the evaluator actually does, and the
[Eyeron guide](https://github.com/eyereasoner/eyeron/blob/main/docs/guide.md) for building,
embedding, testing, and the code layout. (Upstream's `docs/` and `examples/` trees are not
carried in this vendored copy; see `UPSTREAM.md`.)

## Input syntaxes

| Syntax | File extension |
| --- | --- |
| Notation3 (N3) | `.n3` |
| Turtle, TriG | `.ttl`, `.trig` |
| N-Triples, N-Quads | `.nt`, `.nq` |
| RDF Message Log | any, recognized by its `VERSION "*-messages"` directive |

Rules are written in N3; the RDF syntaxes carry the data those rules reason over, and any mixture of them can be given in one run. Eyeron dispatches on a file's extension or by content sniffing. [Upstream's `docs/n3.md`](https://github.com/eyereasoner/eyeron/blob/main/docs/n3.md) covers the syntax, semantics, CLI flags, internals, and known limitations in depth.

```bash
cargo build --release
# Example programs (examples/ is not carried in this vendored copy; fetch from upstream):
#   ./target/release/eyeron examples/socrates.n3    # forward/backward Horn rules over an RDF graph
#   ./target/release/eyeron examples/rdf-messages.n3 examples/input/rdf-messages.trig
```

## Documentation

(Upstream docs, not carried in this vendored copy.)

- [Upstream `docs/n3.md`](https://github.com/eyereasoner/eyeron/blob/main/docs/n3.md) — syntax, semantics, resource limits, CLI flags, internals, and known limitations.
- [Upstream `docs/reasoning.md`](https://github.com/eyereasoner/eyeron/blob/main/docs/reasoning.md) — how Eyeron evaluates rules, and what it prints and records in proofs.
- [Upstream `docs/guide.md`](https://github.com/eyereasoner/eyeron/blob/main/docs/guide.md) — building and running, proofs, the Rust library and browser APIs, testing, and the architecture.
- [Upstream `docs/proof-checking.md`](https://github.com/eyereasoner/eyeron/blob/main/docs/proof-checking.md) — what makes a proof document valid for a program, and what `--check-proof` verifies.

## License

MIT. See `LICENSE.md`.
