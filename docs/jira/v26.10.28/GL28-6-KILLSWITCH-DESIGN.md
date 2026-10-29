# GL28-6 — Art.14 Kill-Switch Design

**Status: LANDED @ 4d2ffd0** (`graphlaw_ex: GL28-6 kill-switch + GL28-7
authority ceiling + GL28-11 consolidation`). The implementation and its
courts are committed; this file is a stub. The design narrative below was
historical prose and is deleted — the landed code is the authority.
Retained: §5, the court specification the GL28-BACKLOG court table cites
by path.

- Implementation: `/Users/sac/graphlaw/graphlaw_ex/lib/graphlaw_ex/kill_switch.ex`
  (`GraphlawEx.KillSwitch` — `install/2`, `check/1`, `current_epoch/0`,
  `canonical_json/1`, `erase/0`)
- Host enforcement: `GraphlawEx.Host.handle_call/3` pre-envelope check,
  after `saturated/1`, before `run_request/5`
- Court: `/Users/sac/graphlaw/graphlaw_ex/test/graphlaw_ex/kill_switch_test.exs`
  (`GraphlawEx.KillSwitchTest`)

## 5. `art14_killswitch_court` spec

Convention: the flip-twin pattern — one honest twin plus a mutated twin; the honest survivor must pass while every mutant fails, and the honest run alone proves *admission*, not just refusal.

1. **valid lease admits** — with no revocation installed (epoch 0), a normal request returns `{:ok, map}` through the full `gl_alloc`/`gl_call` path; `Host.info/2` reports the host still `:loaded` with `recycles` unchanged (zero-mutation witness).
2. **revoked signature refuses** — a revocation whose signature is mutated by one byte: `install/2` refuses at publication (typed, epoch term unchanged), and calls continue to be admitted. The host never trusts an unverified document.
3. **bumped epoch refuses** — a correctly signed revocation at epoch N+1 is installed; the *next* `request/3` returns `{:error, %Refusal{code: :lease_revoked, class: :refused_identity, broken_term: :R_missing_standing}}`; `memory_size/2` is byte-identical before and after (no engine touched); telemetry shows `code: :lease_revoked`.
4. **stale epoch refuses to install** — epoch ≤ current is refused by `install/2`; no rollback to a permissive epoch is possible.
5. **flip-twin honest survivor** — revert to epoch 0 (or install a signed re-admission at N+2): calls admit again, proving the refusal was the revocation, not collateral host damage. Mutant twins (scope digest wrong, wrong operator key, non-monotone epoch) must each produce refusal or install-failure.
