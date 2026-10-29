defmodule GraphlawEx.Application do
  @moduledoc """
  Supervision tree for the GraphlawEx bridge. `GraphlawEx` runs as a named
  child so callers can use the module name as the server.
  """
  use Application

  @impl true
  def start(_type, _args) do
    # GraphlawEx is started by its consumer with an explicit artifact path
    # (opt or :graphlaw_ex, :wasm_path app env). Auto-start is opt-in so the
    # supervision tree never races a test's own instance or loads the wrong
    # artifact byte-slice.
    children =
      if Application.get_env(:graphlaw_ex, :auto_start, false) do
        [GraphlawEx]
      else
        []
      end

    Supervisor.start_link(children, strategy: :one_for_one, name: GraphlawEx.Supervisor)
  end
end
