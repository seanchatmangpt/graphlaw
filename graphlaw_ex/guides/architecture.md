# Architecture

Why the `graphlaw_ex` runtime is shaped the way it is. For task steps see the Operations guide.

## Boundary

The BEAM holds only the wasm byte-slice. Law admission happens inside the module, so the host's
job is to load a pinned artifact, route calls, enforce limits and authority, and report typed
refusals.

## Supervision tree

Auto-start is opt-in (`auto_start: true`). Otherwise the application starts an empty
`:rest_for_one` supervisor, `GraphlawEx.Supervisor`, plus the kill-switch server when
`:kill_switch_path` is set (`lib/graphlaw_ex/application.ex:61-70`).

With auto-start the children are, in order:

1. `GraphlawEx.KillSwitch.Server` (only with `:kill_switch_path`)
2. `:engine_boot`, a transient one-shot worker
3. `GraphlawEx.Pool`

`:rest_for_one` means a restart of an earlier child restarts all later ones.

## Boot order

The kill switch is first so a persisted revocation is enforced before the engine boots or any
worker exists; a failed restore refuses the whole tree. `:engine_boot` then runs synchronously:
sha256 pin and import-surface admission, compile once, publish the module in `:persistent_term`
(`EngineLoad.boot/2`), and return `:ignore`. Nothing after it starts until the module exists.
BLAKE3 is deliberately not checked at boot; that is `mix graphlaw_ex.verify` at publish time.

If the engine file is unreadable, `start/2` returns `{:error, {:wasm_unreadable, ...}}` before
any supervisor exists. If admission refuses, the application exits with
`{:shutdown, {:failed_to_start_child, :engine_boot, reason}}`.

## Pool, hosts and leases

`GraphlawEx.Pool` is `:rest_for_one` over `[Registry, Pool.Leases, Pool.Workers]`;
`Pool.Workers` is a nested `:one_for_one` supervisor of `GraphlawEx.Host` members, so one Host
crash restarts only that Host. Restart intensity is 10 in 30 seconds (`pool.ex:19-24`).

Here "lease" has two meanings. `Pool.Leases` owns concurrency slots: a caller claims a slot with
a CAS on an `:atomics` array, and the server monitors the holder so a dead caller's slot is
freed. A sweep every 1000 ms frees slots no live holder tracks (`pool/leases.ex:1-12`). Signed
authority leases are a separate mechanism checked by `GraphlawEx.Authority`.

## Kill switch and authority

`KillSwitch.check/1` is one `:persistent_term` integer compare inside `Host.handle_call/3`,
after load shedding and before the request is built, so a revoked call costs no fuel and no
allocation. Authority asks whether the caller presented enough; the kill switch asks whether
the operator withdrew it. A call needs both, and revocation wins
(`lib/graphlaw_ex/kill_switch.ex` moduledoc).

## Failure semantics

A trap, timeout or fuel exhaustion recycles the instance. Engine load failures are retried by
the supervised host with capped exponential backoff. Refusals are values, not exceptions;
see the Refusals and Errors guide.

## See Also

- Operations guide
- Releases guide
- Capabilities guide
