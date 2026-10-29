# GL28-6 — Art.14 Kill-Switch Design

**Operator-held Ed25519 revocation lease/epoch, checked per call, pre-envelope in `GraphlawEx.Host`.**
Status: design only; coordinator dispatches implementation. Input seams read: `/Users/sac/graphlaw/graphlaw_ex/lib/graphlaw_ex/host.ex`, `/Users/sac/graphlaw/graphlaw_ex/lib/graphlaw_ex/engine_load.ex`, `/Users/sac/ash_graphlaw/lib/ash_graphlaw/authority.ex`.

---

## 1. Principle

Art.14 grants the operator a hard, unilateral off-switch: a signed revocation that stops every engine call the instant it is published, with **zero state mutation** on the refused path. The host already has the right shape for this — `Host.handle_call({:request, ...})` is the single serialized gate in front of `run_request/5`, which is the only path to `gl_alloc`. The kill-switch check is a **pre-call envelope**: it runs after server-side shedding (`saturated/1`) but **before** `run_request/5` touches the store — so a revoked call costs no fuel, no `gl_alloc`, no write, no `gl_call`, and no recycle.

Like `EngineLoad.admit/2` and `Authority.check_ceiling/2`, this is a host-side presentation check. The engine never learns the call happened; nothing is cached as admitted; no evidence of success can exist.

## 2. The revocation document

Mirrors `Authority`'s signed-lease container (`%{"lease" => ..., "attestation" => ...}`) but inverted in polarity — a lease *grants*, a revocation *withdraws*:

```elixir
%{
  "revocation" => %{
    "epoch" => 7,                      # monotonically increasing integer
    "scope" => "all",                  # or a wasm_sha256 / engine-id prefix
    "issued_unix" => 1_769_000_000,
    "reason" => "art14-order-2026-10-28"
  },
  "attestation" => %{
    "key_id" => "ops-2026",            # must be in trusted operator keys
    "alg" => "ed25519",
    "signature" => "<hex Ed25519 over canonical_json(revocation)>"
  }
}
```

Canonicalization reuses `AshGraphLaw.Evidence.canonical_json/1` semantics (sorted keys, string-normalized); signature bytes are Ed25519 over that canonical form. **Digest scope**: `"scope": "all"` revokes every engine; a scope holding a `wasm_sha256` revokes only the engine whose digest equals `state.wasm_sha256` — matching the pin identity the host already carries from `EngineLoad.admit/2`.

**Verification happens in Elixir, once, at publication** — not per call. This is the deliberate inversion of `Authority` (where the engine verifies per request). Rationale: per-call crypto on the hot path reintroduces exactly the cost the kill-switch must avoid; the operator signature is checked when the epoch is installed, and per call only the *epoch integer* is compared.

## 3. Propagation: `:persistent_term` epoch compare

**Per call, the check is one `:persistent_term.get` of an integer — no file I/O, no watcher on the call path.**

- `GraphlawEx.KillSwitch.install(doc, public_keys)` verifies the Ed25519 signature against the operator trust anchors (analogous to `Authority.engine_opts/3` taking `trusted_keys` only from runtime config — caller context can never supply keys), checks `scope`, then writes `%{epoch, scope_digest, key_id, installed_at}` under key `{GraphlawEx.KillSwitch, :epoch}` via `:persistent_term.put/2`. A single scheduler-wide-literal write is one global GC flip; installs are rare operator events, so this is acceptable.
- The host's `handle_call({:request, ...})` reads the epoch **per call** (`GraphlawEx.KillSwitch.current_epoch/0` → `:persistent_term.get`). No file watcher: file watching is only how an operator *tool* invokes `install/2` out of band, never part of the call path. Re-reading the file per call would make a stale/missing file a failure mode; the epoch term makes the fast path one read and makes revocation take effect for every in-flight caller's *next* enqueued request (in-transaction calls complete; nothing is interrupted mid-`gl_call` — Art.14 stops admission, it does not yank the instance).
- Epochs are monotone: `install/2` refuses an epoch ≤ current (typed `:stale_epoch` refusal in details, not a crash), so a replayed old revocation cannot roll state back.

## 4. The typed refusal — `LeaseRevoked`

Zero mutation: the check returns before `run_request/5`, so store, fuel, `recycles`, `status`, and the engine instance are untouched. No recycle is triggered; `@recycle_codes` is not extended.

Add to the **closed** `Refusal` code table (following the `class_of/1` + `broken_term_of/1` pattern in `/Users/sac/graphlaw/graphlaw_ex/lib/graphlaw_ex/refusal.ex`):

- code `:lease_revoked`
- class `:refused_identity` (the operator's standing to call is what is missing — same class as `:wasm_digest_mismatch`)
- broken term `:R_missing_standing`
- details: `%{epoch, key_id, scope, wasm_sha256, reason}`

Telemetry reuses `[:graphlaw_ex, :host, :call, :stop]` with `outcome: :error, code: :lease_revoked` — no new event; `emit_call/3` already carries any `Refusal` code.

Check placement, concretely, in `Host`:

```elixir
def handle_call({:request, body, op, timeout}, from, state) do
  case saturated(state) do
    nil ->
      case GraphlawEx.KillSwitch.check(state.wasm_sha256) do
        :ok -> run_request(body, op, timeout, from, state)
        {:error, %Refusal{code: :lease_revoked} = refusal} ->
          GenServer.reply(from, {:error, refusal})   # zero state change
          {:noreply, state}
      end
    refusal -> {:reply, {:error, refusal}, state}
  end
end
```

`request/3` and `request_raw/3` need no changes — the refusal surfaces as `{:error, %Refusal{code: :lease_revoked}}` through the existing `GenServer.call` return.

## 5. `art14_killswitch_court` spec

Convention: the flip-twin pattern — one honest twin plus a mutated twin; the honest survivor must pass while every mutant fails, and the honest run alone proves *admission*, not just refusal.

1. **valid lease admits** — with no revocation installed (epoch 0), a normal request returns `{:ok, map}` through the full `gl_alloc`/`gl_call` path; `Host.info/2` reports the host still `:loaded` with `recycles` unchanged (zero-mutation witness).
2. **revoked signature refuses** — a revocation whose signature is mutated by one byte: `install/2` refuses at publication (typed, epoch term unchanged), and calls continue to be admitted. The host never trusts an unverified document.
3. **bumped epoch refuses** — a correctly signed revocation at epoch N+1 is installed; the *next* `request/3` returns `{:error, %Refusal{code: :lease_revoked, class: :refused_identity, broken_term: :R_missing_standing}}`; `memory_size/2` is byte-identical before and after (no engine touched); telemetry shows `code: :lease_revoked`.
4. **stale epoch refuses to install** — epoch ≤ current is refused by `install/2`; no rollback to a permissive epoch is possible.
5. **flip-twin honest survivor** — revert to epoch 0 (or install a signed re-admission at N+2): calls admit again, proving the refusal was the revocation, not collateral host damage. Mutant twins (scope digest wrong, wrong operator key, non-monotone epoch) must each produce refusal or install-failure.

## 6. Non-goals

- No per-call Ed25519 verification (publication-time only).
- No interruption of in-flight transactions; Art.14 is an admission gate.
- No `Authority`/`AshGraphLaw` changes — the ceiling machinery is untouched; the kill-switch is a host-layer standing check that composes with, and precedes, any lease ceiling.