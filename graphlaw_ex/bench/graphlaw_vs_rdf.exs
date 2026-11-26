# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
# Sparql.query crossover vs triple count (WASM path): 1 warmup + N timed rounds,
# median and p95 reported. The :rdf baseline does parse + the same select.
#
#   mix run bench/graphlaw_vs_rdf.exs
#   BENCH_SIZES="10,100,1000" BENCH_ROUNDS=3 mix run bench/graphlaw_vs_rdf.exs
#
# Env: BENCH_SIZES (comma-separated triple counts), BENCH_ROUNDS (one positive int).
#
# Mix.install is refused inside a Mix project, and :rdf is not a dependency of this
# project, so the :rdf comparison runs only when RDF.Graph is already loadable;
# otherwise the WASM curve alone is reported.

defmodule Bench.GraphlawVsRdf do
  alias GraphlawEx.Capability.Sparql

  @query "SELECT ?s ?o WHERE { ?s <http://example.org/p> ?o } LIMIT 10"
  @default_sizes "10,100,1000,5000,10000,50000"

  def nquads(n) do
    Enum.map_join(1..n, "", fn i ->
      "<http://example.org/s#{i}> <http://example.org/p> \"v#{i}\" .\n"
    end)
  end

  @doc """
  One untimed warmup run, then `rounds` timed runs of `fun`.
  Returns `{median_us, p95_us, last_result}`.
  """
  def stats_us(rounds, fun) do
    _ = fun.()
    timed = Enum.map(1..rounds, fn _ -> :timer.tc(fun) end)
    sorted = timed |> Enum.map(&elem(&1, 0)) |> Enum.sort()
    {_, result} = List.last(timed)
    {percentile(sorted, 50), percentile(sorted, 95), result}
  end

  @doc "Nearest-rank percentile of an ascending-sorted non-empty list."
  def percentile(sorted, p) do
    rank = max(1, ceil(p / 100 * length(sorted)))
    Enum.at(sorted, rank - 1)
  end

  @doc "Parses a single positive integer; halts with a clear message on bad input."
  def parse_positive_int(name, raw) do
    case Integer.parse(String.trim(raw)) do
      {n, ""} when n > 0 -> n
      _ -> abort("#{name}=#{inspect(raw)} must be a single positive integer")
    end
  end

  # Baseline does the same work as the WASM path: parse the N-Quads, then select
  # the first 10 objects of the queried predicate.
  def rdf_baseline(data) do
    graph = apply(RDF.NQuads, :read_string!, [data])

    graph
    |> apply(RDF.Graph, :triples, [])
    |> Enum.filter(fn {_s, p, _o} -> to_string(p) == "http://example.org/p" end)
    |> Enum.take(10)
  end

  @doc "Parses a positive-integer CSV; halts with a clear message on bad input."
  def parse_positive_csv(name, raw) do
    values =
      raw
      |> String.split(",", trim: true)
      |> Enum.map(&Integer.parse(String.trim(&1)))

    if values == [] or Enum.any?(values, &(&1 == :error or elem(&1, 0) < 1)) do
      abort("#{name}=#{inspect(raw)} must be a comma-separated list of positive integers")
    end

    Enum.map(values, &elem(&1, 0))
  end

  def abort(msg) do
    IO.puts(:stderr, "bench: " <> msg)
    System.halt(2)
  end

  def ensure_started do
    case Application.ensure_all_started(:graphlaw_ex) do
      {:ok, _} -> :ok
      {:error, reason} -> abort("could not start :graphlaw_ex: #{inspect(reason)}")
    end
  end

  def run do
    ensure_started()

    sizes = parse_positive_csv("BENCH_SIZES", System.get_env("BENCH_SIZES", @default_sizes))

    rounds = parse_positive_int("BENCH_ROUNDS", System.get_env("BENCH_ROUNDS", "5"))
    rdf? = Code.ensure_loaded?(RDF.Graph) and Code.ensure_loaded?(RDF.NQuads)

    IO.puts("graphlaw_ex #{GraphlawEx.version()}  rounds=#{rounds}  rdf_package=#{rdf?}")

    rows =
      for n <- sizes do
        data = nquads(n)

        {med, p95, result} =
          stats_us(rounds, fn ->
            Sparql.query(data, @query, max_rows: 100_000, timeout_ms: 120_000)
          end)

        status =
          case result do
            {:ok, r} -> "ok rows=#{length(r.rows)}"
            {:error, ref} -> "error #{inspect(ref.code)}"
          end

        rdf_cells =
          if rdf? do
            {rmed, rp95, _} = stats_us(rounds, fn -> rdf_baseline(data) end)
            [Integer.to_string(rmed), Integer.to_string(rp95)]
          else
            []
          end

        [Integer.to_string(n), Integer.to_string(med), Integer.to_string(p95), status] ++
          rdf_cells
      end

    header =
      ["triples", "wasm_med_us", "wasm_p95_us", "status"] ++
        if(rdf?, do: ["rdf_med_us", "rdf_p95_us"], else: [])

    print_table([header | rows])
  end

  def print_table(rows) do
    widths =
      rows
      |> Enum.zip_with(fn col -> col |> Enum.map(&String.length/1) |> Enum.max() end)

    Enum.each(rows, fn row ->
      row
      |> Enum.zip(widths)
      |> Enum.map_join("  ", fn {cell, w} -> String.pad_trailing(cell, w) end)
      |> String.trim_trailing()
      |> IO.puts()
    end)
  end
end

Bench.GraphlawVsRdf.run()
