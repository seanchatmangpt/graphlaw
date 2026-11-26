# Changelog

All notable changes to `graphlaw_ex` are recorded here. Versions are calendar
versions matching `mix.exs`.

## 26.11.25

- Lease policy: `config :graphlaw_ex, :lease_policy` is `:required` (default; a lease-less
  `law` request is refused with `ceiling_unmet`) or `:none_allowed` (`config/test.exs`).
  Request opts can only tighten it (`lib/graphlaw_ex/authority.ex:415-460`,
  `config/config.exs:3`).
- `GraphlawEx.KillSwitch.Server` is a supervised first child when `:kill_switch_path` is set;
  without it the application logs a notice and starts no kill-switch server
  (`lib/graphlaw_ex/application.ex:60-90`).
- `Pool.request` admission saturation returns `%Refusal{code: :saturated}` with `capacity`
  detail (`lib/graphlaw_ex/pool.ex:123-125,230`).
- `GraphlawEx.admit/2` is payload-first (`admit(payload, server: server)`); `admit/3`
  (server first) remains as a `@deprecated` shim (`lib/graphlaw_ex.ex:153-176`).
- `Capability.Extension` reads `overrides:` as AST and never evaluates it
  (`lib/graphlaw_ex/capability/extension.ex:77`).
- Error class parity: the refusal-kind class table in `GraphlawEx.Error.Class` is checked
  against `registry/capability-registry.json` by `test/graphlaw_ex/error_class_parity_test.exs`
  (`lib/graphlaw_ex/error/class.ex:78-84`).
- Manifest authority: `artifact.blake3` in `MANIFEST.json` is the pin; the `.blake3` sidecar
  is only a cross-check and a disagreement fails closed
  (`lib/graphlaw_ex/wasm_config.ex:257-290`).
- Config validation: limit keys must be integers at or above their minimum, `:pool_size` is
  rejected, and `:trusted_keys` must be a list of non-empty strings
  (`lib/graphlaw_ex/config.ex:63-75,112-130`). Validation of `:trusted_keys` is part of
  `Config.validate/1`; the boot call path was not traced this session (UNVERIFIED).
- Option 2 packaging: the Hex package does not ship `graphlaw.wasm` (8 MB Hex limit). It
  ships `priv/graphlaw/MANIFEST.json`, `graphlaw.wasm.blake3` and `graphlaw.wasm.sha256`.
- Added `mix graphlaw_ex.fetch`: downloads the engine from GitHub Release v26.11.25 on
  seanchatmangpt/graphlaw, pins from `MANIFEST.json`. `GRAPHLAW_WASM_PATH` covers
  air-gapped hosts.
- Boot checks the sha256 pin only; `mix graphlaw_ex.verify` now checks BLAKE3.
- Version moves from `0.1.0` to calendar version `26.11.25` (`mix.exs`).
- Typed capability layer (14 verbs with `!` twins), closed refusal table
  (`GraphlawEx.Refusal.codes/0`, 27 codes), kill-switch and authority pre-gate.
- Wasm artifact (fetched, not bundled) pinned in `priv/graphlaw/MANIFEST.json`
  (`artifact.sha256` enforced at boot, `artifact.blake3` published identity; ABI 1).
- Error taxonomy: 10 top-level kinds (`GraphlawEx.Error`), distinct from the refusal table.
- Kill-switch hardening: signed revocation epochs are verified before they shed calls
  (`lease_revoked`).
- Authority hardening: the presentation pre-gate refuses unmet ceilings (`ceiling_unmet`)
  before any bytes reach the engine.
- Pool hardening: lease redesign; a trap, timeout or fuel exhaustion recycles the
  instance and a full queue sheds with `saturated`.
- Removed the `:pool_size` application-config key; pass `size:` to `GraphlawEx.Pool`
  (default `System.schedulers_online/0`).
- WASI lockdown: only the manifest import allowlist is accepted.
- Graph set-algebra blank-node soundness fix.
- License: MIT across the package. Added `LICENSE`, `LICENSES/MIT.txt` and `.reuse/dep5`
  (REUSE metadata covering every file in the package directory).
- Added `guides/ash_r2rml_migration.md`; README rewritten to the real API
  (`Capability.Sparql.query/3`; BLAKE3 is `mix graphlaw_ex.verify`, not boot), with
  zero-ceremony usage, a Releases section, the refusal-code table and sandbox guarantees.
- Package, engine and registry are all aligned at 26.11.25.
