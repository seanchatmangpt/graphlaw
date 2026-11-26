# graphlaw_ex standing (2026-11-17)

Static inspection only; no `mix test` run. /Users/sac/ash_graphlaw not present, so parity was not inspected.

## What it is
BEAM bridge for the graphlaw wasm kernel (v0.1.0, Elixir ~> 1.14, Apache-2.0). Deps wasmex ~> 0.15.1, jason, telemetry, ex_doc (dev). The BEAM holds only the wasm bytes; admission happens in-module.

## Module map (lib/graphlaw_ex/)
graphlaw_ex.ex (facade + defdelegates, bang variants, lines 104-185); application.ex (auto_start opt-in); host.ex (635 L); pool.ex (154); engine_load.ex (254); wasm_config.ex (255); abi.ex (161); authority.ex (434); kill_switch.ex (447); refusal.ex (476) + error/refused.ex; telemetry.ex; capability.ex + capability/* (per-verb builders); result/* typed structs.

## Engine load path
1. wasm_path precedence: opts[:wasm_path] > env GRAPHLAW_WASM_PATH > app env :wasm_path > vendored priv/graphlaw/graphlaw.wasm.
2. Pin = MANIFEST.json artifact.sha256. Explicit opts path differing from vendored => :unpinned; env/app-config paths stay pinned; unreadable manifest => refusal.
3. EngineLoad.admit/2: sha256 pin (:wasm_digest_mismatch) before compile; closed 28-function wasi_snapshot_preview1 allowlist (:wasm_import_surface_mismatch); required exports gl_alloc, gl_call, gl_free, memory; compile; cache in :persistent_term keyed {sha256, fuel?}. Telemetry [:graphlaw_ex,:engine,:admit].
4. Limits: fuel = timeout_ms * fuel_per_ms; max_response_bytes caps copy-out.
Doc nit: engine_load.ex moduledoc item 3 has a garbled sentence.
Pin check: wasm sha256 on disk (62cd95d4...c8f8, 62,909,537 B) == MANIFEST.json; version 26.10.28, abi 1. (This is sha256; the registry pin c92caeb6... is BLAKE3 of the same 62,909,537 B file — different algorithms, not a conflict; not recomputed here.)

## GRAPHLAW_REQUIRE_ENGINE
Set to '1' in elixir.yml; consumed by test/support/engine.ex (flunks on missing artifact, never silent skip). Gap: capability_smoke_test.exs header still describes a "LOUD typed skip" with PRIV_WASM absent; confirm default-path behaviour (43ea53e).

## CI (elixir.yml, graphlaw_ex-courts)
push main + PR on graphlaw_ex/**; ubuntu-latest, 60 min; Elixir 1.18.4 / OTP 27.3; rust 1.98.0; `mix test`. No format/credo/dialyzer, no hex.publish.

## Tests (grep-derived, ~118)
a2a_authority_ceiling 9; graphlaw_ex_test 7; abi 13; authority 26; capability 26; capability_smoke 12; engine_load 5; host 4; kill_switch 13; pool 4; refusal 15; telemetry 6; wasm_config 9. Thin: host, pool, engine_load.

## Gaps
- GL28-11: items (b) re-exports, (c) unit courts, (d) smoke default path look landed; (a) dual refusal vocabulary consolidation has no evidence (GL28-2 Q6 pending). Re-run with GRAPHLAW_REQUIRE_ENGINE=1 and receipt.
- Pin is 26.10.28; no 26.11.x re-vendor.
- 62.9 MB wasm in priv/; hex package files list = lib mix.exs README only (does not ship the engine); GL28-12 undecided.
- Stray graphlaw_ex/tmp/GraphlawEx.WasmConfigTest/.../foreign.wasm test leftover; confirm gitignored.
- ash_graphlaw (GL28-10 = WO-7 PARTIAL): last recorded 55c1bc3, parity pins 26.10.28, consumer MANIFEST lists 7 imports vs canonical 28, consumer sha 34fd4ada... stale; must re-vendor.

## Next actions
Run `GRAPHLAW_REQUIRE_ENGINE=1 mix test` and receipt; close GL28-11 b/c/d, scope (a); re-vendor ash_graphlaw; decide hex packaging; fix moduledoc; clean tmp/.
