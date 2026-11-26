# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule Mix.Tasks.GraphlawEx.Doctor do
  @shortdoc "Diagnoses the graphlaw_ex install: artifact, pins, engine load, round trip, pool"
  @moduledoc """
  Prints a checklist for the local `graphlaw_ex` setup and exits non-zero when
  any check fails.

      mix graphlaw_ex.doctor [--path /abs/graphlaw.wasm]

  `--path` checks that artifact instead of the configured one. Unknown flags raise.

  Checks, in order:

    * library version
    * `config :graphlaw_ex` validity (`GraphlawEx.Config.validate/0`)
    * resolved engine artifact path (exists, size)
    * sha256 pin present in `MANIFEST.json`, and the artifact matches it
    * BLAKE3 pin present, and the artifact matches it
    * the engine boots (pin, import surface, required exports)
    * a canonical round trip (`GraphlawEx.Capability.Graph.canonicalize/1`)
      through the running pool, or through an ephemeral `GraphlawEx.Host` when
      `auto_start` is off and no pool runs (a warning, not a failure)
    * pool and limits configuration

  Each line is `[ok]`, `[warn]` or `[FAIL]`. Only `[FAIL]` affects the exit code.
  """
  use Mix.Task

  alias GraphlawEx.EngineLoad
  alias GraphlawEx.EngineLoad.Blake3
  alias GraphlawEx.WasmConfig

  @probe "<urn:graphlaw:doctor> <urn:graphlaw:p> \"ok\" .\n"

  @impl Mix.Task
  def run(args) do
    {opts, _rest} = OptionParser.parse!(args, strict: [path: :string])
    Mix.Task.run("app.config")
    Mix.shell().info("graphlaw_ex doctor")

    path = opts[:path] || WasmConfig.wasm_path()
    results = [version(), config_check(), artifact(path) | pins_and_engine(path)]
    results = results ++ [round_trip(path), pool()]

    failures = Enum.count(results, &(&1 == :fail))

    if failures > 0 do
      Mix.raise("graphlaw_ex doctor: #{failures} check(s) failed")
    else
      Mix.shell().info("graphlaw_ex doctor: all checks passed")
    end
  end

  defp version do
    line(:ok, "version", GraphlawEx.version())
  end

  defp config_check do
    case GraphlawEx.Config.validate() do
      :ok -> line(:ok, "config", "config :graphlaw_ex is valid")
      {:error, errors} -> line(:fail, "config", Enum.join(errors, "; "))
    end
  end

  defp artifact(path) do
    case File.stat(path) do
      {:ok, %{type: :regular, size: size}} ->
        line(:ok, "artifact", "#{path} (#{size} bytes)")

      {:ok, %{type: type}} ->
        line(:fail, "artifact", "#{path} is a #{type}, not a regular file")

      {:error, reason} ->
        line(
          :fail,
          "artifact",
          "#{path}: #{:file.format_error(reason)}; run mix graphlaw_ex.fetch"
        )
    end
  end

  defp pins_and_engine(path) do
    case File.read(path) do
      {:ok, bytes} ->
        [sha256_check(bytes), blake3_check(bytes), engine_check(bytes)]

      {:error, _} ->
        [
          line(:fail, "sha256 pin", "skipped: artifact unreadable; run mix graphlaw_ex.fetch"),
          line(:fail, "blake3 pin", "skipped: artifact unreadable; run mix graphlaw_ex.fetch"),
          line(:fail, "engine load", "skipped: artifact unreadable; run mix graphlaw_ex.fetch")
        ]
    end
  end

  defp sha256_check(bytes) do
    case WasmConfig.pinned_sha256() do
      nil ->
        line(:fail, "sha256 pin", "missing or invalid artifact.sha256 in MANIFEST.json")

      pin ->
        case :crypto.hash(:sha256, bytes) |> Base.encode16(case: :lower) do
          ^pin -> line(:ok, "sha256 pin", pin)
          actual -> line(:fail, "sha256 pin", "mismatch: expected #{pin}, actual #{actual}")
        end
    end
  end

  defp blake3_check(bytes) do
    case WasmConfig.pinned_blake3() do
      nil ->
        line(:fail, "blake3 pin", "no sidecar or manifest artifact.blake3")

      pin ->
        case Blake3.hash_hex(bytes) do
          ^pin -> line(:ok, "blake3 pin", pin)
          actual -> line(:fail, "blake3 pin", "mismatch: expected #{pin}, actual #{actual}")
        end
    end
  end

  defp engine_check(bytes) do
    case EngineLoad.admit(bytes) do
      {:ok, admitted} ->
        line(:ok, "engine load", "admitted sha256 #{admitted.wasm_sha256}")

      {:error, refusal} ->
        line(:fail, "engine load", "#{inspect(refusal.code)}: #{refusal.message}")
    end
  rescue
    error -> line(:fail, "engine load", Exception.message(error))
  end

  defp round_trip(path) do
    Mix.Task.run("app.start")

    if pool_live?() do
      report_round_trip(GraphlawEx.Capability.Graph.canonicalize(@probe), "")
    else
      ephemeral_round_trip(path)
    end
  rescue
    error -> line(:fail, "canonical round trip", Exception.message(error))
  catch
    kind, reason ->
      line(
        :fail,
        "canonical round trip",
        "#{kind}: #{inspect(reason, limit: 10, printable_limit: 200)}"
      )
  end

  defp pool_live? do
    GraphlawEx.Pool.members() != []
  rescue
    _ -> false
  catch
    _, _ -> false
  end

  # auto_start is false and no pool runs: probe through a short-lived, unnamed
  # Host pinned to the manifest digest, then stop it.
  defp ephemeral_round_trip(path) do
    case GraphlawEx.Host.start_link(
           name: nil,
           wasm_path: path,
           expected_sha256: WasmConfig.expected_sha256()
         ) do
      {:ok, host} ->
        try do
          result = GraphlawEx.Capability.Graph.canonicalize(@probe, server: host)
          report_round_trip(result, " (ephemeral host; no pool running, auto_start is off)")
        after
          GenServer.stop(host)
        end

      {:error, reason} ->
        line(
          :fail,
          "canonical round trip",
          "ephemeral host failed to start: #{inspect(reason, limit: 10, printable_limit: 200)}"
        )
    end
  end

  defp report_round_trip({:ok, nquads}, note) when is_binary(nquads) and nquads != "",
    do: line(:ok, "canonical round trip", String.trim(nquads) <> note)

  defp report_round_trip({:ok, other}, _note),
    do: line(:fail, "canonical round trip", "unexpected result: #{inspect(other, limit: 10)}")

  defp report_round_trip({:error, %{message: message} = refusal}, _note)
       when is_binary(message) do
    if message =~ ~r/lease/i do
      line(
        :warn,
        "canonical round trip",
        "skipped under lease_policy #{inspect(GraphlawEx.Authority.lease_policy())}: " <>
          "#{message}; supply a lease or set `lease_policy: :none_allowed` for the probe"
      )
    else
      line(:fail, "canonical round trip", "#{inspect(refusal.code)}: #{message}")
    end
  end

  defp report_round_trip({:error, refusal}, _note),
    do: line(:fail, "canonical round trip", "#{inspect(refusal.code)}: #{refusal.message}")

  defp pool do
    limits = WasmConfig.limits()
    live = safe(fn -> length(GraphlawEx.Pool.members()) end, 0)
    down = safe(fn -> length(GraphlawEx.Pool.unavailable_members()) end, 0)

    summary =
      "size=#{safe(fn -> GraphlawEx.Pool.size() end, :unknown)} live=#{live} " <>
        "unavailable=#{down} " <>
        "available_slots=#{inspect(safe(fn -> GraphlawEx.Pool.available() end, :unknown))} " <>
        "timeout_ms=#{limits.timeout_ms} max_queue=#{limits.max_queue} " <>
        "memory_limit_bytes=#{limits.memory_limit_bytes}"

    cond do
      live > 0 ->
        line(:ok, "pool", summary)

      down > 0 ->
        line(:fail, "pool", "no live member; " <> summary)

      true ->
        line(
          :warn,
          "pool",
          "not running (auto_start is off; set `auto_start: true`); " <> summary
        )
    end
  end

  defp safe(fun, default) do
    fun.()
  rescue
    _ -> default
  catch
    _, _ -> default
  end

  defp line(status, label, detail) do
    tag =
      case status do
        :ok -> "[ok]  "
        :warn -> "[warn]"
        :fail -> "[FAIL]"
      end

    Mix.shell().info("  #{tag} #{String.pad_trailing(label, 22)} #{detail}")
    status
  end
end
