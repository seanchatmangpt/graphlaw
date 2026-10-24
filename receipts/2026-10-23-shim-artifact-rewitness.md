# Shim-artifact re-witness — identity unification of the wasm shim topology

- **Date**: 2026-10-23
- **Repo**: `/Users/sac/graphlaw` @ main `0d11bc3` (working tree dirty; this lane ran only `b3sum` on in-tree shim sources)
- **Scope**: aggregate three witnesses into one identity decision; unify the shim identity to one canonical recipe.

## 1. Boundary fuzz on the shim-family artifact (fz2 build)

- **Verdict**: 1,172 hostile frames / 0 defects, exit 0.
- **Artifact**: shim-family wasm built under `target-lane-fz2`, blake3 `33083b92…` (digest truncated at dispatch; full digest held by the fuzz lane).
- **Topology caveat**: the fz2 build used the **plain shim + `--allow-multiple-definition`** — a VARIANT of the canonical relink-8 recipe, not the canonical itself. Figures are valid for the shim-family identity only.
- **Provenance gap (explicit, not laundered)**: no prior in-repo receipt carries this run. The canonical prior-art fuzz court is `receipts/gws-9-wasm-boundary-fuzz.md` (312 calls, 2026-10-22 artifact — different subject). The 1,172-frame figures are carried here from the fuzz lane's dispatch; a fuzz-lane receipt embedding the verbatim exit-0 output for the 33083b92 artifact is owed and should be filed by that lane. This row aggregates; it does not re-witness what this lane did not run.

## 2. FIBO 4/0 on the CANONICAL relink-8 artifact

- **Verdict**: GREEN — FIBO battery 4/0.
- **Artifact**: `target-coord/wasm32-wasip1/wasm/graphlaw_wasm.wasm`, 58,214,121 B, blake3 `4d9083af…eb67b`.
- **Receipt cited, not duplicated**: `receipts/2026-10-23-sqlrdf27-verdict.md` (embeds the verbatim `b3sum`, `stat -f %z`, and `test result: ok. 4 passed; 0 failed` output, plus the full relink RUSTFLAGS command). This row adds no new run.

## 3. Identity unification decision

**Canonical identity = relink-8 recipe** (rlib stub = libc-gap symbols only; standalone shim object as sole `__wrap_*` provider, whole-archive; `--wrap=open,--wrap=openat`), as witnessed in `receipts/2026-10-23-sqlrdf27-verdict.md`.

**Shim sources now committed in-tree** at `tests/fixtures/wasm-shim/`, closing the REQUIRED INTEGRATION item from the SQLRDF-27 receipt (artifact replayability precondition):

```
$ b3sum tests/fixtures/wasm-shim/*.c
06456ccf8644a92d136009769167cde8bb95d41f2a0e7bb8ad3db17ab241c43d  tests/fixtures/wasm-shim/glaw_wrap_shim.c
10579bb9192aa159f77b08b41e7a6b07b1b608ba103951e1401487b5975e1f90  tests/fixtures/wasm-shim/glaw_open_shim.c
```

(command executed by this lane, 2026-10-23, real output embedded above)

- `glaw_wrap_shim.c` blake3 `06456ccf…1c43d` (full: `06456ccf8644a92d136009769167cde8bb95d41f2a0e7bb8ad3db17ab241c43d`)
- `glaw_open_shim.c` blake3 `10579bb9…e1f90` (full: `10579bb9192aa159f77b08b41e7a6b07b1b608ba103951e1401487b5975e1f90`)

Note the canonical relink-8 recipe must now reference the in-tree path
`tests/fixtures/wasm-shim/glaw_wrap_shim.c` (not `/tmp/glaw_wrap_shim.c`)
for the artifact to be replayable from the tree.

## Open items

1. **Memory-wall + lifecycle witnesses on the canonical identity** — in flight; owned by their lanes. The lifecycle leg is also OPEN on `docs/jira/v26.10.22/SQLRDF-27.md` per the verdict receipt.
2. **SPC Cpk FAIL under load** — needs a solo-machine rerun; not witnessed by this lane.

## Standing

Aggregation receipt only: identity decision recorded, shim-source integration gap CLOSED (sources in-tree, digests embedded). It re-witnesses nothing run by other lanes; every figure is either cited to an embedded-output receipt or explicitly marked as carried from a lane dispatch.
