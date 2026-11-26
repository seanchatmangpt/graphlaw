# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule Mix.Tasks.GraphlawEx.Verify do
  @shortdoc "Verifies the vendored engine against its pinned BLAKE3 (publish/CI gate)"
  @moduledoc """
  Hashes the engine `graphlaw.wasm` with BLAKE3 and compares it to the pin.
  Exits non-zero with `REFUSED_ARTIFACT_INTEGRITY` on any mismatch.

  This is a publish/CI-time gate; the library never runs it at boot.

      mix graphlaw_ex.verify
      mix graphlaw_ex.verify --path priv/graphlaw/graphlaw.wasm

  ## Options

    * `--path` - engine file to verify (default: `GraphlawEx.WasmConfig.wasm_path/0`).
      A path other than the vendored engine is verified against its
      `<path>.blake3` sidecar; with no sidecar there is no pin and the task fails.

  The pin comes only from `GraphlawEx.WasmConfig` (MANIFEST.json
  `artifact.blake3`, sidecar first as WasmConfig resolves it). When the manifest
  carries `artifact.size`, the byte size must match too.

  See also `mix graphlaw_ex.doctor` for a full environment report.
  """
  use Mix.Task

  alias GraphlawEx.EngineLoad
  alias GraphlawEx.WasmConfig

  @impl Mix.Task
  def run(args) do
    {opts, _rest, invalid} = OptionParser.parse(args, strict: [path: :string])

    if invalid != [] do
      Mix.raise(
        "unknown options: #{inspect(invalid)}. Usage: mix graphlaw_ex.verify [--path FILE]"
      )
    end

    Mix.Task.run("app.config")
    path = Keyword.get(opts, :path) || WasmConfig.wasm_path()

    bytes =
      case File.read(path) do
        {:ok, bytes} ->
          bytes

        {:error, reason} ->
          Mix.raise(
            "cannot read #{path}: #{:file.format_error(reason)}; run mix graphlaw_ex.fetch"
          )
      end

    verify_opts = if Keyword.has_key?(opts, :path), do: [wasm_path: path], else: []

    case EngineLoad.verify_artifact(bytes, verify_opts) do
      :ok ->
        case WasmConfig.expected_blake3(verify_opts) do
          pin when is_binary(pin) ->
            check_size!(path, byte_size(bytes), verify_opts)
            Mix.shell().info("graphlaw_ex: #{path} (#{byte_size(bytes)} bytes)")
            Mix.shell().info("graphlaw_ex: blake3 #{pin} OK")

          _unpinned_or_missing ->
            Mix.raise(
              "REFUSED_ARTIFACT_INTEGRITY: #{path} has no BLAKE3 pin " <>
                "(no MANIFEST artifact.blake3 or #{path}.blake3 sidecar); nothing to verify against"
            )
        end

      {:error, {code, details}} ->
        Mix.raise("#{code}: #{format_details(details)}")
    end
  end

  defp format_details(details) when is_map(details) do
    details
    |> Enum.sort()
    |> Enum.map_join(", ", fn {key, value} -> "#{key}=#{format_value(value)}" end)
  end

  defp format_value(nil), do: "none"
  defp format_value(value) when is_binary(value), do: value
  defp format_value(value) when is_atom(value), do: Atom.to_string(value)
  defp format_value(value), do: inspect(value, limit: 10, printable_limit: 200)

  # The manifest size applies only to the vendored (pinned) engine, never to a
  # foreign --path verified against its own sidecar.
  defp check_size!(path, actual, verify_opts) do
    vendored? =
      case Keyword.fetch(verify_opts, :wasm_path) do
        {:ok, p} -> Path.expand(p) == Path.expand(WasmConfig.vendored_path())
        :error -> true
      end

    with true <- vendored?,
         {:ok, manifest} <- WasmConfig.manifest(),
         size when is_integer(size) <- get_in(manifest, ["artifact", "size"]),
         true <- size != actual do
      Mix.raise(
        "REFUSED_ARTIFACT_INTEGRITY: #{path} size mismatch: " <>
          "expected #{size} bytes, actual #{actual}"
      )
    else
      _ -> :ok
    end
  end
end
