# GL28-7 Port Design: Authority Pre-Gate for graphlaw_ex

Inputs read: `/Users/sac/ash_graphlaw/lib/ash_graphlaw/authority.ex` (source of truth), `/Users/sac/graphlaw/graphlaw_ex/lib/graphlaw_ex/refusal.ex` and `/Users/sac/graphlaw/graphlaw_ex/lib/graphlaw_ex/capability/api.ex` + `capability/refusal.ex` (port target).

---

## 0. Landscape facts that shape the design

- `graphlaw_ex` has **no Spark, no Ash, no DSL, no `Admissions` module, no `Evidence` module, no resource runtime section**. The Ash-side `Authority` depends on `AshGraphLaw.Admissions` (resource capability declarations + runtime config) and `AshGraphLaw.Evidence.canonical_json/1` — neither can port.
- The engine wire surface already carries the lease vocabulary: `GraphlawEx.Capability.Law` request fields include `signed_lease`, `lease`, `trusted_keys`, `max_skew_secs`, `unverified_lease`, `now_unix` (registry lines ~1036–1091), and refusal kinds `lease_ceilings`/`lease_reasons` exist in the registry (`expired, out_of_scope, ceiling, bad_signature, untrusted_key, clock_skew`). So the Elixir side only needs the **presentation pre-gate**; verification stays engine-side.
- `graphlaw_ex` has **two refusal types**: `GraphlawEx.Refusal` (closed 12-code host/transport exception) and `GraphlawEx.Capability.Refusal` (open code struct for the capability layer). The gate sits inside `Capability.API.run/3`, so the ceiling refusal must be a `GraphlawEx.Capability.Refusal` (the layer's own type), not a new host exception code — that preserves the host table's closed-table invariant.
- `run/3` currently dispatches `Registry.module_for/1 → module.run(args, opts)` with no gate at all.

## 1. New module: `GraphlawEx.Authority`

File: `graphlaw_ex/lib/graphlaw_ex/authority.ex` (hand-written, `UNSUPPORTED(generator-capability)` marker, same as the Ash one).

### Ports verbatim (no Spark/DSL dependency in these paths)

| Function | Verbatim? | Notes |
|---|---|---|
| `claim/1` | **Verbatim** | Pure: `@ceiling_rank`, `@ceiling_names`, `signed_lease/1`, `fetch_signed/1`, `ceiling_of/1`, `get/2`, string/atom key tolerance. Only dep is `@ceiling_rank`/`@ceiling_names` literals. Total (any term → `:observe`). |
| `identity/1` | **Verbatim body, swap digest helper** | Replace `Evidence.canonical_json/1` with a local private `canonical_json/1` (sorted-key JSON via `Jason.encode!` on the normalized map with sorted keys) and keep the local `sha256/1` (`:crypto.hash(:sha256, _) |> Base.encode16(case: :lower)`). Public type/contract unchanged. |
| `op_ceiling/1` | **Adapt trivially** | `@op_ceilings` map copies verbatim (identical 14-op split: observe for capabilities/sniff/parse/convert/canonical/sparql/shacl/shex/policy; construct for n3/entail/datalog/hooks/law); `:unknown_capability` refusal becomes `GraphlawEx.Capability.Refusal.build/3` instead of `AshGraphLaw.Refusal.new/3`. |
| `engine_opts/3` | **Adapt** | No `Admissions.runtime/1` exists. Signature becomes `engine_opts(opts)` — runtime `trusted_keys`/`max_skew_secs`/`timeout_ms` move to caller-supplied defaults resolved as `opts[:trusted_keys]`, `opts[:max_skew_secs]`, `opts[:timeout]` (see §2). Never forwards `now_unix`, `lease`, `unverified_lease` (same deny-list as upstream). Appends `signed_lease:` only when `claim/1` yields a signed lease. |
| `check_ceiling/2` | **Adapt** | Upstream takes a `%AshGraphLaw.Dsl.Admission{}`. Here the required ceiling is a plain atom; signature `check_ceiling(required, lease)` where `required ∈ [:observe, :select, :construct]`. Same rank comparison, same refusal shape (`:ceiling_unmet` with `admission/required/granted` — rename `admission` key to `required_by`). |
`:ceiling_unmet` on the **capability** refusal type, not the host exception. |
| `check_op/2` (2-arity form) | **Adapt** | Upstream's `check_op(resource, op, policy)` needs `Admissions.capabilities/1`. Drop the resource/capability-declaration dimension entirely — graphlaw_ex has no resource layer; declaring nothing allows everything there is no analogue for. Port only the **lease-vs-op-floor** half: `check_op(op, lease)` → `{:ok, required} = op_ceiling(op)`; if `required == :construct`, `claim(lease).ceiling` must be `:construct`, else `:ceiling_unmet` with `%{op:, required:, granted:}`. `:observe`-floor ops always pass (back-compat: no lease → pass). |

### Not ported
- `Admissions`-dependent halves (`capabilities/1`, `capability/2`, `runtime/1`), the `struct()`/Admission struct clause of `check_ceiling/2`, evidence stamping into an Ash evidence attribute (callers can call `identity/1` themselves for receipts/logs).

### Ceiling ordering invariant (unchanged)
`observe(0) < select(1) < construct(2)`; a lease satisfies a demand iff `rank(claimed) >= rank(required)`; `:observe` always passes; a lease never widens derivation. A caller cannot self-grant by writing an atom into args — only a well-formed `signed_lease` container claims above `:observe`.

## 2. Pre-gate placement in `Capability.API.run/3`

`run/3` becomes the **only** gate; the 14 typed wrappers (`capabilities/2` … `policy/2`) delegate to `run/3` for gating (they already call `module.run(args, opts)` directly — change them to `run(name, args, opts)` so the gate cannot be bypassed via the typed functions).

```elixir
@spec run(String.t(), args(), keyword()) :: {:ok, term()} | {:error, GraphlawEx.Capability.Refusal.t()}
def run(op_name, args, opts) when is_binary(op_name) do
  with {:ok, module} <- Registry.module_for(op_name),
       :ok <- GraphlawEx.Authority.check_op(op_name, opts[:lease]) <- 1
  do
    module.run(args, engine_opts(opts))
  end
end
```

1. `opts[:lease]` is the lease container (same shape as Ash changeset-context `:graphlaw_lease`: `%{signed_lease: %{...}}`, string or atom keys). `nil`/absent = `:observe`.
2. Order is load-bearing: **ceiling check before `module.run/2` (Host dispatch)**, so a construct-ceiling op (`n3`, `entail`, `datalog`, `hooks`, ` law`) with a lease claiming only `:observe`/`:select` (or no lease at all) is refused `:ceiling_unmet` **before any WASM dispatch, queueing, or fuel spend**. `sniff`/`parse`/`capabilities` etc. are unaffected (floor `:observe`).
3. Options plumbing: `engine_opts(opts)` resolves `server`, `timeout`, `trusted_keys`, `max_skew_secs` from `opts` (no runtime section), then appends `signed_lease:` when one was presented. `signature`/`expiry`/`attestation` are **not inspected** Elixir-side — the signed lease is forwarded verbatim inside the request to the engine (`law` request already has `signed_lease`, `trusted_keys`, `max_skew_secs` fields; per-op forwarding = when `opts[:signed_lease]` present, merge into the built request map before `Host.request/3`). Signature and expiry verification is the engine's job on the same request; the Elixir side checked only the *claimed* ceiling. Note the pinned engine ignores lease keys (`UNSUPPORTED(engine-capability)`), so today the pre-gate is the only enforced check — same posture as upstream.

## 3. Refusal codes

- **No new code on the host table.** `GraphlawEx.Refusal` stays a closed 12-code transport table; the ceiling refusal is a client-side code on `GraphlawEx.Capability.Refusal` (which already carries client-side codes). Add to its moduledoc + `code()` type:
  - **`:ceiling_unmet`** — class `:refused_structure`, broken_term `:R_missing_standing`, message e.g. `"capability \"law\" requires ceiling :construct, lease grants :observe"`, details `%{"op" => op, "required" => required, "granted" => granted}` (string-keyed, wire-safe).
  - `:unknown_capability` already exists there — `op_ceiling/1`'s miss path uses it, so `run/3`'s unknown-name path is unchanged.
- Deliberately **not** added: `:lease_refused` (engine-only, `lease_reasons` wire values stay in `raw`), `:capability_not_declared` (no resource layer).

## 4. `a2a_authority_ceiling_court` spec

**Claim under test:** an agent presenting no lease (or an unsigned/atom ceiling claim) that invokes law/hooks (construct-ceiling ops) is refused `:ceiling_unmet` before Host dispatch; sniff/parse (`:observe` floor) survive.

### Setup
```elixir
opts = [server: GraphlawEx, timeout_ms: 30_000]   # engine loaded, pinned WASM
lease_observe = %{signed_ lease: %{"lease" => %{"ceiling" => "observe"}, "attestation" => %{"key_id" => "k", "signature" => "aa"}}}
lease_construct = %{signed_lease: %{"lease" => %{"ceiling" => "construct", "id" => "lease-1"}, "signed by trusted key" => true}}
```
(Full shape: `%{signed_lease: %{"lease" => %{"id" => "lease-1", "ceiling" => "construct"}, "attestation" => telemetry-preserving shape}}` — attestation content is opaque to the Elixir side.)

### Cases
1. **Unsigned agent invokes `law`** — `GraphlawEx.Capability.API.run("law", valid_law_args, %{})` (no lease in opts) → `{:error, %GraphlawEx.Capability.Refusal{code: :ceiling_unmet, details: %{"granted" => :observe, "required" => :construct, "op" => "law"}}}`. Assert: no engine call (assert via a `:server` stub or `GraphlawEx.Host.request/3` Mox/`test` double — the refusal must be returned without WASM dispatch; e.g. assert the pool received zero requests via telemetry `[:graphlaw_ex, :request, :start]` count == 0).
2. **Unsigned agent invokes `hooks`** — `run("hooks", valid_hooks_args, %{})` → same `:ceiling_unmet` (granted `:observe`, required `:spec`... required is `:construct`), pre-dispatch.
3. **Sniff survives unsigned** — `run("sniff", %{"text" => "@prefix ..."}` (no lease) → `{:ok, %GraphlawEx.Capability.Sniff{}}` — real engine, full dispatch, `:observe` floor met trivially. `parse` identical.
4. **`parse`/`sniff` with construct lease also fine** — over-presented authority is allowed (>=); `run("parse", args, lease: lease_construct)` → `:ok` path.
5. **`select`-claiming lease on `law`** — `%{signed_lease: %{"lease" => %{"ceiling" => "select"}}}` → `:ceiling `:ceiling_unmet` (rank 1 < 2).
6. **Self-grant attacks refused at claim/1** — `claim(%{ceiling: :construct})`, `claim(:construct)`, `claim(%{lease: %{"ceiling" => *construct*"}})`, `claim(%{unverified_lease: %{...}})`, `claim(nil)` all → `%{ceiling: :observe, signed_lease: nil}` → `:ceiling_unmet` on `law`.
7. **Forged-ceiling signed lease passes the gate, engine's job** — a signed container claiming `construct` with a garbage attestation passes the pre-gate (presentation check only) and is forwarded; behavior is engine-defined (pinned engine ignores lease keys → runs; a lease-verifying engine would refuse `:lease_refused` — assert only that the gate lets it through and the signed lease reaches the request map).
8. **Unknown op** — `run("nope", %{}, %{})` → `:unknown_capability` (unchanged path).
9. **identity/1 for evidence/receipts** — `GraphlawEx.Authority.identity(lease_construct)` → `%{ceiling: :construct, lease_id: "lease-情况进行1", key_id: "ops-2026", lease_digest: <<64 hex>>}`; `identity(nil)` → `nil`.

### Trace assertion helper
- Gate fires pre-dispatch: assert `GraphlawEx.Telemetry` `[:graphlaw_ex, :request, :start]` event count == 0 in cases 1/2/5/6.

Telemetry point: check the actual event names in `graphlaw_ex/lib/graphexc/lib/graphlaw_ex/telemetry.ex` during implementation; the design asserts "zero engine dispatch events for refused calls."

## 5. File-by-file change list

| File | Change |
|---|---|
| `graphlaw_ex/lib/graphlaw_ex/authority.ex` | **New.** `claim/1`, `check_ceiling/2`, `op_ceiling/1`, `check_op/2`, `engine_opts/1`, `identity/1`, privates `signed_lease/1`, `fetch_signed/1`, `ceiling_of/1`, `get/2`, `normalize/1`, `canonical_json/1`, ` Evidence.canonical_json` replacement, `sha256/1` |
| `graphlew_ex/lib/graphlaw_ex/capability/refusal.ex` | Add `:ceiling_unmet` (class `:refused_structure`, broken_term `:R_missing_ missing_standing`) to moduledoc + type, `class/broken_term` mapping, `ceiling_unmet/2` constructor |
`GraphlawEx.Capability.Refusal` type + constructor. |
| `graphlaw_ex/lib/graphlaw_ex/capability/api.ex` | `run/3` gains the `check_op` gate before `module.run/2`; 14 typed wrappers route through `run/3` so the gate cannot be bypassed; `engine_opts/1` helper applied to `opts` before dispatch |
| `graphlaw_ex/test/graphlaw_ex/authority_test.exs` | Port `claim/1` totality + self-grant attack unit tests (case 6) |
| `graphlaw_ex/test/capability/api_test.exs` | Gate placement tests (§4 cases 1–8) |
| `graphlaw_ex/test/.../a2a_authority_ceiling_court_test.exs` | The §4 court spec |

---

**Deviations from upstream to record in the design log:** `check_ceiling/2` takes a ceiling atom instead of an `%Admission{}` struct (no DSL); `engine_opts/3 → engine_opts/1` (no runtime section — trust anchors come from caller opts, flagged `UNSUPPORTED(generator-capability)`); `check_op/3 → check_op/2` (no resource capability declarations — the whole resource-declaration dimension is not ported); digest helper inlined (no `Evidence` module).

**Invariant to preserve in review:** the gate is a presentation check only; the signed lease (signature/expiry/attestation) is forwarded to the engine verbatim on the same request, and the Elixir side never verifies a signature.