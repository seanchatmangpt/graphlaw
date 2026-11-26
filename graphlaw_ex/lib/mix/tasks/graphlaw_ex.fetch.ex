# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule Mix.Tasks.GraphlawEx.Fetch do
  @shortdoc "Stages the graphlaw wasm artifact (env path, staged, or GitHub Release)"

  @moduledoc """
  Stages the graphlaw wasm at `GraphlawEx.WasmConfig.cache_path/0`
  (`<cache root>/<blake3>/graphlaw.wasm`; the root is `GRAPHLAW_CACHE_DIR`, else
  `$XDG_CACHE_HOME/graphlaw_ex`, else `~/.cache/graphlaw_ex`), which is where the
  loader looks.
  The wasm is not shipped in the Hex package.

  Resolution order:

    1. `GRAPHLAW_WASM_PATH` set: verify sha256 + BLAKE3 of that file and copy it
       to the cache path. No network.
    2. Cache path present with matching digest: no-op.
    3. Otherwise download the GitHub Release asset for this package version
       (URL derived from graphlaw_ex's own `source_url` and version), verify
       sha256 and BLAKE3 before placing it. Fails closed; partial files are deleted.

  This is the only place in the library that touches the network.
  """

  use Mix.Task

  @max_redirects 5
  @asset "graphlaw.wasm"

  # Captured at compile time from graphlaw_ex's OWN project (when this task is
  # compiled as a dependency, Mix.Project.config() is graphlaw_ex's, not the
  # consumer's). Evaluating it at run time would read the consumer's config.
  @source_url Mix.Project.config()[:source_url]
  @version Mix.Project.config()[:version]

  defp release_url, do: "#{@source_url}/releases/download/v#{@version}/#{@asset}"

  @impl Mix.Task
  def run(args) do
    {opts, _rest} = OptionParser.parse!(args, strict: [path: :string])
    Mix.Task.run("app.config")

    # Stage where WasmConfig actually reads: the pinned user-cache path.
    target =
      case GraphlawEx.WasmConfig.cache_path_result() do
        {:ok, path} -> path
        {:error, {_code, message}} -> fail(message)
      end

    expected = expected_sha256()

    case opts[:path] || System.get_env("GRAPHLAW_WASM_PATH") do
      path when is_binary(path) and path != "" -> from_path(path, target, expected)
      _ -> from_staged_or_release(target, expected)
    end
  end

  defp from_path(path, target, expected) do
    with {:ok, bytes} <- File.read(path),
         :ok <- verify(bytes, expected) do
      place(bytes, target)
      Mix.shell().info("graphlaw_ex.fetch: staged #{path} -> #{target}")
    else
      {:error, reason} -> fail("source #{path}: #{format_error(reason)}")
    end
  end

  defp from_staged_or_release(target, expected) do
    with {:ok, bytes} <- File.read(target),
         :ok <- verify(bytes, expected) do
      Mix.shell().info("graphlaw_ex.fetch: #{target} present, digest ok")
    else
      {:error, {:REFUSED_ARTIFACT_INTEGRITY, _}} ->
        Mix.shell().info("graphlaw_ex.fetch: cache present but digest mismatch, re-downloading")

        download(target, expected)

      _ ->
        download(target, expected)
    end
  end

  defp download(target, expected) do
    url = release_url()
    Mix.shell().info("graphlaw_ex.fetch: downloading #{url}")

    with {:ok, bytes} <- http_get(url, @max_redirects),
         :ok <- verify(bytes, expected) do
      place(bytes, target)
      Mix.shell().info("graphlaw_ex.fetch: verified and placed #{target}")
    else
      {:error, reason} ->
        fail("download failed closed: #{format_error(reason)}")
    end
  end

  defp http_get(_url, redirects) when redirects < 0, do: {:error, :too_many_redirects}

  defp http_get(url, redirects) do
    {:ok, _} = Application.ensure_all_started(:inets)
    {:ok, _} = Application.ensure_all_started(:ssl)

    http_opts = [
      ssl: [
        verify: :verify_peer,
        cacerts: :public_key.cacerts_get(),
        customize_hostname_check: [
          match_fun: :public_key.pkix_verify_hostname_match_fun(:https)
        ]
      ],
      autoredirect: false,
      timeout: 120_000
    ]

    case :httpc.request(:get, {String.to_charlist(url), []}, http_opts, body_format: :binary) do
      {:ok, {{_, 200, _}, _headers, body}} ->
        {:ok, body}

      {:ok, {{_, status, _}, headers, _}} when status in [301, 302, 303, 307, 308] ->
        case List.keyfind(headers, ~c"location", 0) do
          {_, loc} -> http_get(to_string(loc), redirects - 1)
          nil -> {:error, {:redirect_without_location, status}}
        end

      {:ok, {{_, status, _}, _, _}} ->
        {:error, {:http_status, status}}

      {:error, reason} ->
        {:error, {:http, reason}}
    end
  end

  defp verify(bytes, expected) do
    actual = :crypto.hash(:sha256, bytes) |> Base.encode16(case: :lower)

    if actual != expected do
      {:error, {:REFUSED_ARTIFACT_INTEGRITY, %{expected: expected, actual: actual}}}
    else
      GraphlawEx.EngineLoad.verify_artifact(bytes, [])
    end
  end

  defp place(bytes, target) do
    File.mkdir_p!(Path.dirname(target))
    part = target <> ".part"

    try do
      File.write!(part, bytes)
      File.rename!(part, target)
    rescue
      error ->
        File.rm(part)
        fail("cannot place #{target}: #{Exception.message(error)}")
    end
  end

  @remedy "stage a pinned build manually and set GRAPHLAW_WASM_PATH=/abs/graphlaw.wasm " <>
            "(or pass --path), then re-run mix graphlaw_ex.fetch"

  defp format_error({:http_status, status}),
    do: "HTTP #{status} from #{release_url()}; #{@remedy}"

  defp format_error({:http, reason}),
    do: "network error #{inspect(reason, limit: 10, printable_limit: 200)}; #{@remedy}"

  defp format_error({:REFUSED_ARTIFACT_INTEGRITY, %{expected: e, actual: a}}),
    do: "REFUSED_ARTIFACT_INTEGRITY sha256 expected #{e}, actual #{a}; #{@remedy}"

  defp format_error({:REFUSED_ARTIFACT_INTEGRITY, detail}),
    do: "REFUSED_ARTIFACT_INTEGRITY #{inspect(detail, limit: 10)}; #{@remedy}"

  defp format_error(reason) when is_atom(reason) do
    case :file.format_error(reason) do
      ~c"unknown POSIX error" ++ _ -> inspect(reason)
      msg -> to_string(msg)
    end
  end

  defp format_error(reason), do: inspect(reason, limit: 10, printable_limit: 200)

  # One pin source: `priv/graphlaw/MANIFEST.json` (same as `mix graphlaw_ex.doctor`).
  defp expected_sha256 do
    GraphlawEx.WasmConfig.pinned_sha256() ||
      fail("missing or invalid artifact.sha256 in MANIFEST.json")
  end

  @spec fail(String.t()) :: no_return()
  defp fail(msg), do: Mix.raise("graphlaw_ex.fetch: #{msg}")
end
