# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.KillSwitch.Store do
  @moduledoc """
  Durable store for the verified signed revocation documents of
  `GraphlawEx.KillSwitch`, so a trip survives a node restart.

  The file (config `:graphlaw_ex, :kill_switch_path`) holds JSON
  `{"version": 1, "docs": [...]}`: the additive set of signed revocation
  documents (revoked scopes, high-water epoch, and any explicit signed
  re-admit document), ordered by epoch. The store never verifies signatures;
  `GraphlawEx.KillSwitch.Server` re-verifies every document through
  `GraphlawEx.KillSwitch.install/2` on load.

  Writes are atomic: temp file in the same directory, `:file.sync/1`, then
  `File.rename/2`, then an fsync of the parent directory.

  `save/*` and `append/*` are read-modify-write over a caller-held document
  list and are NOT safe for concurrent writers. `GraphlawEx.KillSwitch.Server`
  is the single writer; it passes the path it captured at boot to `append/3`
  rather than re-reading global application env on every call. `append/2`
  (global env path) remains for single-process tooling only.

  ## Public functions

  | function | purpose |
  |---|---|
  | `path/0` | configured file path or `nil` |
  | `load/0`, `load/1` | `{:ok, docs}` (empty when no file) or typed error |
  | `save/1`, `save/2` | atomically persist a document list |
  | `append/2`, `append/3` | add one verified document (dedupe by epoch) and persist |
  | `high_water_epoch/1` | highest epoch in a document list |

  ## Examples

      iex> GraphlawEx.KillSwitch.Store.high_water_epoch([
      ...>   %{"revocation" => %{"epoch" => 3}},
      ...>   %{"revocation" => %{"epoch" => 7}}
      ...> ])
      7
      iex> GraphlawEx.KillSwitch.Store.high_water_epoch([])
      0
  """

  @type doc :: map()
  @type error ::
          {:read_failed, File.posix()}
          | {:write_failed, File.posix()}
          | :corrupt
          | :unsupported_version

  @doc "Configured store path, or `nil` when persistence is not configured."
  @spec path() :: Path.t() | nil
  def path, do: Application.get_env(:graphlaw_ex, :kill_switch_path)

  @doc "Loads from the configured path. No path configured: `{:ok, []}`."
  @spec load() :: {:ok, [doc()]} | {:error, error()}
  def load do
    case path() do
      nil -> {:ok, []}
      p -> load(p)
    end
  end

  @doc "Loads documents from `path`. A missing file is `{:ok, []}`."
  @spec load(Path.t()) :: {:ok, [doc()]} | {:error, error()}
  def load(path) do
    case File.read(path) do
      {:error, :enoent} -> {:ok, []}
      {:error, reason} -> {:error, {:read_failed, reason}}
      {:ok, bin} -> decode(bin)
    end
  end

  @doc "Persists `docs` to the configured path (no-op `:ok` when unconfigured)."
  @spec save([doc()]) :: :ok | {:error, error()}
  def save(docs) do
    case path() do
      nil -> :ok
      p -> save(p, docs)
    end
  end

  @doc "Atomically writes `docs` to `path` (tmp + fsync + rename)."
  @spec save(Path.t(), [doc()]) :: :ok | {:error, error()}
  def save(path, docs) when is_list(docs) do
    tmp = path <> ".tmp." <> Integer.to_string(System.unique_integer([:positive]))
    body = Jason.encode!(%{"version" => 1, "docs" => sort(docs)})

    with :ok <- File.mkdir_p(Path.dirname(path)),
         :ok <- write_synced(tmp, body),
         :ok <- File.rename(tmp, path),
         :ok <- sync_dir(Path.dirname(path)) do
      :ok
    else
      {:error, reason} ->
        _ = File.rm(tmp)
        {:error, {:write_failed, reason}}
    end
  end

  @doc "Appends a verified `doc` to the configured file (deduped by epoch)."
  @spec append([doc()], doc()) :: :ok | {:error, error()}
  def append(existing, doc) do
    case path() do
      nil -> :ok
      p -> append(p, existing, doc)
    end
  end

  @doc "Appends `doc` to `existing` and writes to `path`."
  @spec append(Path.t(), [doc()], doc()) :: :ok | {:error, error()}
  def append(path, existing, doc) do
    epoch = epoch_of(doc)
    kept = Enum.reject(existing, &(epoch_of(&1) == epoch))
    save(path, [doc | kept])
  end

  @doc "Highest `revocation.epoch` across `docs`, `0` when empty."
  @spec high_water_epoch([doc()]) :: non_neg_integer()
  def high_water_epoch(docs), do: docs |> Enum.map(&epoch_of/1) |> Enum.max(fn -> 0 end)

  defp decode(bin) do
    case Jason.decode(bin) do
      {:ok, %{"version" => 1, "docs" => docs}} when is_list(docs) ->
        if Enum.all?(docs, &is_map/1), do: {:ok, sort(docs)}, else: {:error, :corrupt}

      {:ok, %{"version" => _}} ->
        {:error, :unsupported_version}

      _ ->
        {:error, :corrupt}
    end
  end

  defp write_synced(tmp, body) do
    case File.open(tmp, [:write, :raw, :binary]) do
      {:ok, fd} ->
        result =
          with :ok <- :file.write(fd, body), do: :file.sync(fd)

        _ = :file.close(fd)
        result

      {:error, _} = err ->
        err
    end
  end

  # fsync the parent directory so the rename itself is durable. Platforms that
  # cannot open/sync a directory (:eisdir, :einval, :enotsup) are tolerated;
  # any other failure is a real write failure.
  defp sync_dir(dir) do
    case File.open(dir, [:read, :raw]) do
      {:ok, fd} ->
        result = :file.sync(fd)
        _ = :file.close(fd)

        case result do
          :ok -> :ok
          {:error, r} when r in [:einval, :enotsup, :eisdir] -> :ok
          {:error, _} = err -> err
        end

      {:error, r} when r in [:eisdir, :einval, :enotsup] ->
        :ok

      {:error, _} = err ->
        err
    end
  end

  defp sort(docs), do: Enum.sort_by(docs, &epoch_of/1)

  defp epoch_of(%{"revocation" => %{"epoch" => e}}) when is_integer(e), do: e
  defp epoch_of(_), do: 0
end
