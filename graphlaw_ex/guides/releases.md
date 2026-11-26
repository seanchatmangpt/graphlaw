# Releases

The WASM engine ships as a GitHub Release asset, not inside the Hex package. This guide covers
getting it onto hosts, into container images, and onto machines with no network.

## How the engine is located

`GraphlawEx.WasmConfig.wasm_path/1` resolves, in order:

1. the `:wasm_path` option
2. the `GRAPHLAW_WASM_PATH` environment variable
3. `config :graphlaw_ex, :wasm_path`
4. the vendored `priv/graphlaw/graphlaw.wasm`, when it exists
5. the user cache, when the file exists
6. otherwise the vendored path, so errors name the canonical missing location

Whatever the path, the bytes must match the digests pinned in the packaged manifest. Setting
`GRAPHLAW_WASM_PATH` or `:wasm_path` in config does not unpin the engine, so the environment
cannot swap the admission engine. Only an explicit `wasm_path:` option in code opts out, and it
says so at the call site.

## Cache path

The cache holds one directory per pinned BLAKE3:

```text
<cache root>/<blake3>/graphlaw.wasm
```

The cache root is `GRAPHLAW_CACHE_DIR`, else `$XDG_CACHE_HOME/graphlaw_ex`, else
`~/.cache/graphlaw_ex`. With none of those resolvable (no `HOME`), the library returns a typed
`:no_cache_dir` error instead of guessing. A new release has a new digest and therefore a new
directory, so old and new engines coexist.

## Baking the engine into an image

Fetch at build time so containers never need the network at start-up:

```dockerfile
FROM hexpm/elixir:<tag> AS build
WORKDIR /app
COPY mix.exs mix.lock ./
RUN mix deps.get
RUN GRAPHLAW_CACHE_DIR=/opt/graphlaw mix graphlaw_ex.fetch

FROM debian:stable-slim
COPY --from=build /app/_build /app/_build
COPY --from=build /opt/graphlaw /opt/graphlaw
ENV GRAPHLAW_CACHE_DIR=/opt/graphlaw
```

The runtime stage resolves the same pinned cache path because `GRAPHLAW_CACHE_DIR` is identical
in both stages. Alternatively copy the file anywhere and point `GRAPHLAW_WASM_PATH` at it:

```dockerfile
COPY graphlaw.wasm /opt/graphlaw/graphlaw.wasm
ENV GRAPHLAW_WASM_PATH=/opt/graphlaw/graphlaw.wasm
```

Run `mix graphlaw_ex.doctor` as a final build step to fail the image build on a bad digest or
import surface rather than at first request.

## Air-gapped hosts

On a connected machine, download the release asset for the exact package version you deploy and
carry it across. On the isolated host:

```bash
GRAPHLAW_WASM_PATH=/path/to/graphlaw.wasm mix graphlaw_ex.fetch
```

With `GRAPHLAW_WASM_PATH` set, `graphlaw_ex.fetch` makes no network call. It verifies the
SHA-256 and BLAKE3 of that file against the pins, then copies it into the cache path. If either
digest differs the task raises and places nothing. If the cache already holds a matching file,
the task is a no-op.

For releases built without Mix on the target, skip `fetch` and set `GRAPHLAW_WASM_PATH` in the
runtime environment; the loader verifies the pins at boot and refuses a mismatch with
`:wasm_digest_mismatch`.

## Verifying a deployment

```bash
mix graphlaw_ex.doctor
```

Use it in CI and after any artifact change. Its sha256 and blake3 lines must both read `[ok]`.
A mismatch is never a warning: the module is not the admitted GraphLaw release.

## v26.11.25 notes

- The wasm is excluded from the Hex tarball. `mix graphlaw_ex.fetch` pulls the GitHub Release
  asset; `GRAPHLAW_WASM_PATH` is the air-gapped override (sections above).
- Lease policy defaults to `:required`; `config/test.exs` sets `:none_allowed`
  (`graphlaw_ex/config/config.exs:3`, `config/test.exs:5`).
- Deferred to v26.11.26: js/npm, python, Ash.Query IR, NimblePool, streaming hooks,
  receipt.rs builder append, and `max_rows` enforced inside the worker. The `abi_bin` v2 framing
  is experimental and unwired, with a delete criterion of v26.11.26.

## See Also

- Getting Started guide
- Refusals and Errors guide
- Operations guide
