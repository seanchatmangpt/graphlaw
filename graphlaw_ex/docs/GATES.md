# GATES

The seven Hex publish gates for `graphlaw_ex`. Run from `/Users/sac/graphlaw/graphlaw_ex`.
Gates 1-4 are named in the test sources; gates 5-7 are CI steps after `mix test`
(`.github/workflows/elixir.yml`). Set `GRAPHLAW_REQUIRE_ENGINE=1` so an absent engine
fails instead of loud-skipping. A loud SKIP is not a pass.

Aggregate alias (format, warnings-as-errors compile, test, verify):

```bash
GRAPHLAW_REQUIRE_ENGINE=1 mix gates
```

A second alias, `mix ci` (`mix.exs:72-79`), runs format, warnings-as-errors compile, test,
`graphlaw_ex.verify`, `dialyzer` and `hex.build`. It does not run `reuse lint` or
`ggen sync`. Both aliases prefer the `:test` env (`mix.exs:83-85`).

```bash
GRAPHLAW_REQUIRE_ENGINE=1 mix ci
```

Scope: `mix gates` covers gates 1-4 only (format, compile, the test courts, BLAKE3
verify). Gates 5 (hex tarball), 6 (licensing) and 7 (ggen sync) run as separate commands
and are NOT inside the alias.

## Gate 1: artifact integrity

- Boot checks only the sha256 pin in `priv/graphlaw/MANIFEST.json`.
- The BLAKE3 check is NOT run at boot. It runs at publish/CI via the mix task, which
  exits non-zero with `REFUSED_ARTIFACT_INTEGRITY` on a mismatch.
- Court: `test/integrity_test.exs`.

```bash
mix graphlaw_ex.verify
mix test test/integrity_test.exs
b3sum "$(mix run --no-start -e 'IO.puts(GraphlawEx.WasmConfig.wasm_path())')"
# manual equivalent; compare to artifact.blake3 in MANIFEST.json
```

## Gate 2: differential (goldens)

- Goldens live in `test/fixtures/golden/`. The wasm engine reached through the facade
  must match them byte for byte.
- Goldens are regenerated via the engine, then cross-checked against the native crate
  (`/Users/sac/graphlaw`). An engine-only golden is not confirmed; see
  `test/fixtures/golden/README.md` for per-file provenance and the absent `id.blake3`
  (the id court loud-skips while it is absent).
- Court: `test/differential_test.exs` (`PRIV_WASM` overrides the artifact path).

```bash
GRAPHLAW_REQUIRE_ENGINE=1 mix test test/differential_test.exs
```

## Gate 3: pool isolation

- A 64-way storm over a real `GraphlawEx.Pool` and the real engine: timeout, OOM and
  unknown-op killers, callers killed mid-call, plus healthy queries.
- The "trap" killer is an unknown-op refusal (`{"op": "definitely-not-an-op"}`), not a
  wasm trap. At the transport layer it is `{:ok, %{"ok" => false, ...}}`.
- Final-state assertions: zero healthy failures, every member available again, no
  orphan Wasmex instance.
- Court: `test/pool_isolation_test.exs`.

```bash
GRAPHLAW_REQUIRE_ENGINE=1 mix test test/pool_isolation_test.exs
```

## Gate 4: zero egress

- Scans `lib/` and `mix.exs` for HTTP/socket clients, pure-Elixir RDF/SPARQL deps and
  `path:` deps. Includes an anti-vacuity court on synthetic offenders.
- `max_rows` note (SPARQL): the engine wire has no row cap. `:max_rows` (default 10_000)
  is enforced AFTER the engine response, on the single response; above it the call
  refuses `:resource_limit`. It bounds returned rows, not engine work.
- Court: `test/zero_egress_gate_test.exs`.

```bash
mix test test/zero_egress_gate_test.exs
```

## Gate 5: hex tarball

- `mix hex.build` must produce `graphlaw_ex-*.tar` no larger than
  `HEX_TARBALL_MAX_BYTES` (8000000, the hex.pm 8 MB compressed limit). The package ships
  `lib mix.exs README.md CHANGELOG.md LICENSE* LICENSES` and the three pin files
  `priv/graphlaw/MANIFEST.json`, `priv/graphlaw/graphlaw.wasm.blake3`,
  `priv/graphlaw/graphlaw.wasm.sha256`. Not `priv` wholesale: `graphlaw.wasm` is excluded
  and obtained via `mix graphlaw_ex.fetch` or `GRAPHLAW_WASM_PATH`.

```bash
mix hex.build
tarball="$(ls -1 graphlaw_ex-*.tar | head -n1)"
test "$(wc -c < "$tarball")" -le 8000000 && echo OK
```

## Gate 6: licensing

- Every file in the tree is covered by `.reuse/dep5` and `LICENSES/MIT.txt`.

```bash
pipx run reuse lint
```

## Gate 7: ggen sync (projection drift)

- `graphlaw_ex/lib` projections are regenerated, never patched. Any drift fails.

```bash
ggen sync && git diff --exit-code -- graphlaw_ex/lib
```

## Spec gates (7)

Seven spec gates run alongside the seven publish gates. Their names and courts are
registered in the test sources under `test/`; read them there. This file does not
restate their contents.

## Chicago and no-skip rule

- Chicago (classicist): the real wasm artifact is the collaborator; no mocks or stubs.
- No-skip: a loud SKIP is not a pass. Run with `GRAPHLAW_REQUIRE_ENGINE=1` so an absent
  engine flunks the court instead of skipping it.

## Full CI order

```bash
mix deps.get && mix deps.compile
mix format --check-formatted
mix compile --force --warnings-as-errors
mix dialyzer
mix graphlaw_ex.verify
GRAPHLAW_REQUIRE_ENGINE=1 mix test
pipx run reuse lint
mix hex.build
```

The `mix ci` alias is a subset of this order (no `reuse lint`; `dialyzer` runs after
test, not before).

## See Also

`README.md` (digest pin), `../docs/refusals.md` (typed refusals),
`test/fixtures/golden/README.md`.
