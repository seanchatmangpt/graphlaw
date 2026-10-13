# GraphLaw documentation

Documentation is organized by Diataxis: [tutorials](#tutorials) teach,
[how-to guides](#how-to-guides) solve, [reference](#reference) describes,
[explanation](#explanation) explains.

## Tutorials

- [Quickstart](quickstart.md) — build a plan, admit it over an RDF state, collect a
  receipt chain, and meet your first refusal. Every command verified against this tree.

## How-to guides

- [Run the WASM module](how-to/run-the-wasm-module.md) — build the WASI module, drive
  the 14 JSON ops from any host, understand the limits.
- [Authorize steps with signed leases](how-to/authorize-with-signed-leases.md) — Ed25519
  leases, ceilings, trusted clocks, and what refuses when.
- [Verify without trusting the operator](how-to/verify-without-trusting-the-operator.md) —
  `graphlaw-verify` offline replay: flags, exit codes, tamper refusals.
- [Use knowledge hooks](how-to/use-knowledge-hooks.md) — `kh:` packs, fixpoint
  materialization, the refuse effect.
- [Use from JavaScript](how-to/use-from-javascript.md) — install the npm package,
  load the WASM module in Node and browsers.

## Reference

- [ABI reference](abi-reference.md) — the JSON envelope, all 14 ops, error kinds.
- [Capability registry](capability-registry.md) — machine-readable op surface
  (`graphlaw-registry --check`), digest rule.
- [Refusals](refusals.md) — every typed refusal code, its `details` fields, remediation.
- [API stability](api-stability.md) — semver policy, `#[non_exhaustive]` list, MSRV.
- [WASM integration](wasm-integration.md) — generated shell vs hand-written safe core.
- [Full generated reference](reference-generated/reference.md) — machine-rendered
  module/symbol tables (regen: [instructions](reference-generated/README.md)).
- [TypeScript API](reference/typescript-api.md) — the npm package's exported
  functions, types, and error mapping.

## Explanation

- [Qualification algebra](QUALIFICATION_ALGEBRA.md) — standing as executable
  falsification, not assertion.
- [Chicago courts](CHICAGO_N3_DATALOG.md) · [FIBO court](CHICAGO_FIBO.md) ·
  [Cross-authority court](CHICAGO_CROSS_AUTHORITY.md) — conformance corpora and why
  they are shaped the way they are.
- [Receipt fields](explanation/receipt-fields.md) — what a receipt carries and how it
  maps to the portfolio receipt concept (informal).
- [Knowledge hook projection](reference/knowledge-hook-projection.md) — the
  powerless state → hook projection contract.

## History (not maintained)

Dated receipts and the archive: `docs/receipt-*.md`,
[archive/branch-disposition.md](archive/branch-disposition.md)
