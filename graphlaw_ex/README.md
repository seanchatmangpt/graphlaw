# graphlaw_ex

BEAM bridge for the graphlaw WASM law kernel — the deterministic turnstile
boundary for Smalt. The BEAM side holds only the wasm byte-slice; all law
admission happens inside the module. Nothing here interprets RDF, SHACL,
SQL, or receipts.

## Layout

- `lib/graphlaw_ex.ex` — GenServer owning the wasmex store/instance; exposes
  `admit/2` and `stop/1`. The packet-4 stdio ABI framing is a typed stub
  (`{:error, :abi_not_wired}`) naming the ABI entry points until wired.
- `lib/graphlaw_ex/application.ex` — supervision tree, `GraphlawEx` as a
  named child.
- `test/graphlaw_ex_test.exs` — instantiation court against the real
  artifact. Loud typed skip when absent (a skip is not a pass); no mocks.

## Configuration

Artifact path resolution: `:wasm_path` start option → application env
`:graphlaw_ex, :wasm_path` → `../priv/graphlaw.wasm`. In this repo the
artifact lives at `priv/graphlaw.wasm`.

## Run

```bash
cd graphlaw_ex
mix deps.get
PRIV_WASM=$PWD/../priv/graphlaw.wasm mix test
```

Status: scaffold — `mix` was not executed in this environment; compile and
test behavior are UNVERIFIED until `mix test` runs with deps fetched.
