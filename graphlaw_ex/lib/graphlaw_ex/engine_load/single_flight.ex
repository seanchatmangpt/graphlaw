# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
defmodule GraphlawEx.EngineLoad.SingleFlight do
  @moduledoc """
  Process-free single-flight for expensive, cacheable computations.

  The first caller for a key becomes the leader and runs `fun`; concurrent
  callers for the same key send the leader a wait request, monitor it, and
  receive the leader's result. Leadership is claimed atomically with
  `:global.register_name/2` with the name scoped to `node()`, so flights are
  node-local (Wasmex NIF refs never cross nodes). One monotonic deadline is
  fixed per `fetch_or_compile/3` call and shared by every retry. At most 32
  entries stay published; the oldest are evicted.

  Contract for `fun`: `(-> {:ok, value} | {:error, term})`.

    * `{:ok, value}` is published once to `:persistent_term` by the leader
      only, then every waiter is answered with the same tuple. Later callers
      hit the cache directly.
    * `{:error, _}` and raised/thrown/exited failures are NOT cached; the
      leader and its current waiters see the failure, the next caller retries.
    * A leader killed mid-flight releases every waiter through its monitor
      with `{:error, %GraphlawEx.Refusal{code: :call_exited}}`.
    * A waiter that exceeds `:timeout` (default 30_000 ms) gets
      `{:error, %GraphlawEx.Refusal{code: :call_timeout}}` after one last
      cache check.

  ## Usage

      SingleFlight.fetch_or_compile({:engine, digest, true}, fn ->
        {:ok, compile!(bytes)}
      end)

  ## Wiring into `GraphlawEx.EngineLoad`

  `EngineLoad.admit/2` checks the pin per caller, then runs
  `fetch_or_compile({:engine, digest, fuel?}, ...)`; `purge_cache/0` and
  `cached?/2` delegate to `purge/0` and `cached/1`. Errors stay uncached.
  """

  alias GraphlawEx.Refusal

  @default_timeout 30_000
  @max_entries 32

  @type key :: term()
  @type result :: {:ok, term()} | {:error, term()}

  @doc "Returns `{:ok, value}` when `key` was already published, else `:error`."
  @spec cached(key()) :: {:ok, term()} | :error
  def cached(key) do
    case :persistent_term.get(pt_key(key), :__miss__) do
      :__miss__ -> :error
      value -> {:ok, value}
    end
  end

  @doc """
  Returns the published value for `key`, computing it once with `fun` if
  absent. Options: `:timeout` (waiter budget in ms).
  """
  @spec fetch_or_compile(key(), (-> result()), keyword()) :: result()
  def fetch_or_compile(key, fun, opts \\ []) when is_function(fun, 0) and is_list(opts) do
    timeout = Keyword.get(opts, :timeout, @default_timeout)
    deadline = System.monotonic_time(:millisecond) + timeout

    case cached(key) do
      {:ok, value} -> {:ok, value}
      :error -> flight(key, fun, deadline)
    end
  end

  @doc "Erases every published value in a single `:persistent_term` pass."
  @spec purge() :: :ok
  def purge do
    for {{__MODULE__, :done, _} = pt, _} <- :persistent_term.get(), do: :persistent_term.erase(pt)
    :persistent_term.erase({__MODULE__, :index})
    :ok
  end

  # The flight name embeds `node()`: values are Wasmex NIF resources that are
  # node-local, so flights must never coalesce across the cluster.
  defp flight(key, fun, deadline) do
    name = {__MODULE__, :flight, node(), key}

    case :global.register_name(name, self()) do
      :yes -> lead(key, name, fun)
      :no -> follow(key, name, fun, deadline)
    end
  end

  defp lead(key, name, fun) do
    result = run(fun)

    # Publish, then unregister, THEN drain. Any waiter that still sees this
    # leader registered after sending `:wait` sent it before the unregister, so
    # the drain below answers it. A waiter that sees the name gone (or the
    # cache filled) never depends on the drain.
    with {:ok, value} <- result, do: publish(key, value)
    :global.unregister_name(name)
    release(result)
    result
  end

  defp run(fun) do
    case fun.() do
      {:ok, _} = ok -> ok
      {:error, _} = err -> err
      other -> {:error, crashed({:bad_return, other})}
    end
  rescue
    error -> {:error, crashed({:raised, error})}
  catch
    kind, reason -> {:error, crashed({kind, reason})}
  end

  # Answers every wait request already in the mailbox; no blocking.
  defp release(result) do
    receive do
      {__MODULE__, :wait, pid, ref} ->
        send(pid, {__MODULE__, :result, ref, result})
        release(result)
    after
      0 -> :ok
    end
  end

  defp follow(key, name, fun, deadline) do
    case :global.whereis_name(name) do
      :undefined ->
        # Leader finished between register attempt and lookup: re-enter.
        retry(key, fun, deadline)

      leader ->
        ref = Process.monitor(leader)
        send(leader, {__MODULE__, :wait, self(), ref})

        # Re-check after sending: a published value or a departed leader means
        # the drain may have missed our message, so do not wait on it.
        with :error <- cached(key),
             true <- :global.whereis_name(name) == leader do
          await(key, fun, ref, deadline)
        else
          {:ok, value} ->
            abandon(ref)
            {:ok, value}

          false ->
            abandon(ref)
            retry(key, fun, deadline)
        end
    end
  end

  defp abandon(ref) do
    Process.demonitor(ref, [:flush])

    receive do
      {__MODULE__, :result, ^ref, _} -> :ok
    after
      0 -> :ok
    end
  end

  defp await(key, fun, ref, deadline) do
    timeout = max(deadline - System.monotonic_time(:millisecond), 0)

    receive do
      {__MODULE__, :result, ^ref, result} ->
        Process.demonitor(ref, [:flush])
        result

      {:DOWN, ^ref, :process, _pid, :normal} ->
        # Leader returned without answering us: re-resolve.
        abandon(ref)
        retry(key, fun, deadline)

      {:DOWN, ^ref, :process, _pid, reason} ->
        {:error,
         Refusal.new(:call_exited, "single-flight leader exited", %{reason: inspect(reason)})}
    after
      timeout ->
        abandon(ref)

        case cached(key) do
          {:ok, value} ->
            {:ok, value}

          :error ->
            {:error,
             Refusal.new(:call_timeout, "single-flight wait timed out", %{timeout_ms: timeout})}
        end
    end
  end

  # Retries share the caller's ORIGINAL deadline: one budget per call.
  defp retry(key, fun, deadline) do
    case cached(key) do
      {:ok, value} ->
        {:ok, value}

      :error ->
        if System.monotonic_time(:millisecond) >= deadline do
          {:error,
           Refusal.new(:call_timeout, "single-flight wait timed out", %{retries_exhausted: true})}
        else
          flight(key, fun, deadline)
        end
    end
  end

  # Bounded cache: dynamic (:unpinned) modules must not accumulate NIF
  # resources forever. Oldest entries are evicted past `@max_entries`. The
  # index is mutated under a node-local :global lock.
  defp publish(key, value) do
    :persistent_term.put(pt_key(key), value)

    :global.trans(
      {{__MODULE__, :index}, self()},
      fn ->
        index = :persistent_term.get({__MODULE__, :index}, []) |> List.delete(key)
        {keep, evict} = Enum.split([key | index], @max_entries)
        Enum.each(evict, &:persistent_term.erase(pt_key(&1)))
        :persistent_term.put({__MODULE__, :index}, keep)
      end,
      [node()]
    )

    :ok
  end

  defp crashed(reason),
    do:
      Refusal.new(:call_exited, "single-flight computation failed", %{
        reason: inspect(reason, limit: 5, printable_limit: 200)
      })

  defp pt_key(key), do: {__MODULE__, :done, key}
end
