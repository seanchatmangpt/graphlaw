# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.Pool.Workers do
  @moduledoc false
  # Nested `:one_for_one` supervisor holding the pool's Host members, so a crash
  # of one Host restarts only that Host and never its healthy siblings. The pool
  # supervisor orders it after the Registry and Leases (`:rest_for_one`), so a
  # Registry or Leases restart still restarts every member and re-registers it.
  # Restart intensity is explicit and scales with pool size: max(10, 3n)
  # restarts within 30 seconds.
  use Supervisor

  @host GraphlawEx.Host

  @spec start_link(keyword()) :: Supervisor.on_start()
  def start_link(opts), do: Supervisor.start_link(__MODULE__, opts)

  @impl Supervisor
  def init(opts) do
    n = Keyword.fetch!(opts, :size)
    registry = Keyword.fetch!(opts, :registry)
    host_opts = Keyword.fetch!(opts, :host_opts)

    members =
      for i <- 0..(n - 1)//1 do
        name = if i == 0, do: @host, else: nil

        Supervisor.child_spec(
          {@host, Keyword.merge(host_opts, name: name, registry: registry)},
          id: {@host, i}
        )
      end

    Supervisor.init(members,
      strategy: :one_for_one,
      max_restarts: max(10, 3 * n),
      max_seconds: 30
    )
  end
end
