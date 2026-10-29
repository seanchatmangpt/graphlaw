# GL28-7 Port Design: Authority Pre-Gate for graphlaw_ex

**Status: LANDED @ 4d2ffd0** (same commit as GL28-6). The implementation
and its courts are committed; this file is a stub. The design narrative
(§0–§3, §5, design-log paragraphs) was historical prose and is deleted —
the landed code is the authority. Retained: §4, the court specification
the GL28-BACKLOG court table cites by path.

- Implementation: `/Users/sac/graphlaw/graphlaw_ex/lib/graphlaw_ex/authority.ex`
  (`GraphlawEx.Authority` — `claim/1`, `check_ceiling/2`, `op_ceiling/1`,
  `check_op/2`, `engine_opts/1`, `identity/1`)
- Gate seam: `GraphlawEx.Capability.API.run/3` pre-gate (ceiling check
  before module dispatch)
- Court: `/Users/sac/graphlaw/graphlaw_ex/test/a2a_authority_ceiling_court_test.exs`
  (`A2AAuthorityCeilingCourtTest`)

## 4. `a2a_authority_ceiling_court` spec

**Claim under test:** an agent presenting no lease (or an unsigned/atom ceiling claim) that invokes law/hooks (construct-ceiling ops) is refused `:ceiling_unmet` before Host dispatch; sniff/parse (`:observe` floor) survive.

### Setup
```elixir
opts = [server: GraphlawEx, timeout_ms: 30_000]   # engine loaded, pinned WASM
lease_observe = %{signed_lease: %{"lease" => %{"ceiling" => "observe"}, "attestation" => %{"key_id" => "k", "signature" => "aa"}}}
lease_construct = %{signed_lease: %{"lease" => %{"ceiling" => "construct", "id" => "lease-1"}, "attestation" => %{"key_id" => "ops-2026"}}}
```
(Full shape: `%{signed_lease: %{"lease" => %{"id" => "lease-1", "ceiling" => "construct"}, "attestation" => telemetry-preserving shape}}` — attestation content is opaque to the Elixir side.)

### Cases
1. **Unsigned agent invokes `law`** — `GraphlawEx.Capability.API.run("law", valid_law_args, %{})` (no lease in opts) → `{:error, %GraphlawEx.Capability.Refusal{code: :ceiling_unmet, details: %{"granted" => :observe, "required" => :construct, "op" => "law"}}}`. Assert: no engine call (assert via a `:server` stub or `GraphlawEx.Host.request/3` Mox/`test` double — the refusal must be returned without WASM dispatch; e.g. assert the pool received zero requests via telemetry `[:graphlaw_ex, :host, :call, :stop]` count == 0).
2. **Unsigned agent invokes `hooks`** — `run("hooks", valid_hooks_args, %{})` → same `:ceiling_unmet` (granted `:observe`, required is `:construct`), pre-dispatch.
3. **Sniff survives unsigned** — `run("sniff", %{"text" => "@prefix ..."}` (no lease) → `{:ok, %GraphlawEx.Capability.Sniff{}}` — real engine, full dispatch, `:observe` floor met trivially. `parse` identical.
4. **`parse`/`sniff` with construct lease also fine** — over-presented authority is allowed (>=); `run("parse", args, lease: lease_construct)` → `:ok` path.
5. **`select`-claiming lease on `law`** — `%{signed_lease: %{"lease" => %{"ceiling" => "select"}}}` → `:ceiling_unmet` (rank 1 < 2).
6. **Self-grant attacks refused at claim/1** — `claim(%{ceiling: :construct})`, `claim(:construct)`, `claim(%{lease: %{"ceiling" => construct}})`, `claim(%{unverified_lease: %{...}})`, `claim(nil)` all → `%{ceiling: :observe, signed_lease: nil}` → `:ceiling_unmet` on `law`.
7. **Forged-ceiling signed lease passes the gate, engine's job** — a signed container claiming `construct` with a garbage attestation passes the pre-gate (presentation check only) and is forwarded; behavior is engine-defined (pinned engine ignores lease keys → runs; a lease-verifying engine would refuse `:lease_refused` — assert only that the gate lets it through and the signed lease reaches the request map).
8. **Unknown op** — `run("nope", %{}, %{})` → `:unknown_capability` (unchanged path).
9. **identity/1 for evidence/receipts** — `GraphlawEx.Authority.identity(lease_construct)` → `%{ceiling: :construct, lease_id: "lease-1", key_id: "ops-2026", lease_digest: <<64 hex>>}`; `identity(nil)` → `nil`.

### Trace assertion helper
- Gate fires pre-dispatch: assert `GraphlawEx.Telemetry` `[:graphlaw_ex, :host, :call, :stop]` event count == 0 in cases 1/2/5/6.

Telemetry point: check the actual event names in `graphlaw_ex/lib/graphlaw_ex/telemetry.ex` during implementation; the design asserts "zero engine dispatch events for refused calls."
