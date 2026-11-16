# Combinatorial maximalism: why no capability is left out

**Claim.** Every GraphLaw capability reachable from the kernel has a Python entry point, a
doc page that teaches it, and a probe that exercised it through the real runner this run.

**Method.** Enumerate the capability space as a product of axes, derive each axis mechanically
from a source of truth, and require every cell to be PASS or a registered, reasoned GAP.

| Axis | Derived from |
|---|---|
| op, step, dialect, regime, lease_ceiling, lease_reason, receipt_reason, policy_kind, | `registry/capability-registry.json` |
| refusal_kind, refusal_code, limit, engine, authority | same |
| duckdb_mode | dispatch arms in `src/duckdb_op.rs` (the op is not in the registry) |
| bin | the cargo `[[bin]]` surface (`graphlaw.registry.BINARIES`) |
| op_example | `registry/op-examples.json`, all replayed through the wrapper |
| tool | one agent tool per op, generated from the registry |

A new op, step, limit or enum value appears as a new cell with no probe, so the gate fails
`MISSING` until it is wrapped, probed and taught. Nothing is listed by hand.

**What the gate enforces** (`python3 -m graphlaw.coverage`, exit 1 on any defect):

- `PASS`: a real probe succeeded. Probes drive the real runner and binaries; no mocks.
- `GAP`: registered in `coverage.json` with a reason; printed every run. A gap on a cell that no
  longer exists, a gap with no reason, or a gap whose probe now passes is itself a defect.
- `UNTAUGHT`: a taught-axis cell without a `<!-- cap: axis:cell -->` tag in `tutorials/` or
  `how-to/`. A tag naming a non-cell is `ORPHAN-TAG`.
- `STALE-MATRIX`: the generated `reference/capability-matrix.md` differs from the derived one.

**Non-vacuity.** `--self-check` mutates the inputs (stale gap, reasonless gap, passing-cell gap,
stale matrix, removed gap) and requires each to flip the gate red; a mutation that stays green
is reported as `VACUOUS GATE`.

**Known gaps.** The registered gaps are limits whose probes need heavy workloads or are
unreachable under the 16 MiB frame cap. They are visible, counted and reasoned, not hidden.

## See Also

[Capability matrix](../reference/capability-matrix.md) |
[Give an agent GraphLaw](../how-to/give-an-agent-graphlaw.md)
