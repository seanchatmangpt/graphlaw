# SPDX-License-Identifier: MIT
# UNSUPPORTED(generator-capability): hand-written canonical result layer over the
# generated `GraphlawEx.Result.*` decoders.

defmodule GraphlawEx.Result do
  @moduledoc """
  The canonical result a caller receives, validated before it is returned.

  Fields:

    * `:bindings` - solution rows as maps `variable => GraphlawEx.Result.Term.t() | nil`
      (`nil` is an unbound cell). Empty for results without solutions.
    * `:quads` - N-Quads lines of the result graph (blank lines dropped).
    * `:receipt_blake3` - 64-char lowercase hex BLAKE3 identity of the result
      (canonical `id`, or the last receipt's `plan_blake3`), or `nil` when the
      engine response carries none.
    * `:stats` - `%{rows: n, quads: n, receipts: n}` counted from the decoded data.

  `from_decoded/1` accepts an already decoded `GraphlawEx.Result.Sparql`,
  `GraphlawEx.Result.Canonical` or `GraphlawEx.Result.Admitted` struct and
  returns `{:error, %GraphlawEx.Refusal{code: :malformed_response}}` when the
  data is internally inconsistent (ragged rows, a declared quad count that
  disagrees with the N-Quads, a malformed digest). It never raises. A result is
  an observation; it grants no authority.
  """

  alias GraphlawEx.Capability.Refusal, as: CapRefusal
  alias GraphlawEx.Error
  alias GraphlawEx.Refusal
  alias GraphlawEx.Result.{Admitted, Canonical, Sparql, Term}

  @enforce_keys [:bindings, :quads, :receipt_blake3, :stats]
  defstruct [:bindings, :quads, :receipt_blake3, :stats]

  @type binding :: %{optional(String.t()) => Term.t() | nil}

  @type stats :: %{rows: non_neg_integer(), quads: non_neg_integer(), receipts: non_neg_integer()}

  @type t :: %__MODULE__{
          bindings: [binding()],
          quads: [String.t()],
          receipt_blake3: String.t() | nil,
          stats: stats()
        }

  @type source :: Sparql.t() | Canonical.t() | Admitted.t()

  @doc """
  Opt-in canonicalization of a capability return value.

  Accepts a decoded result struct, or the `{:ok, struct}` / `{:error, refusal}`
  tuple a capability function returns, so it composes at the end of a pipeline
  without changing what the capability itself returns:

      iex> alias GraphlawEx.Result.Sparql
      iex> s = Sparql.from_map(%{"variables" => ["x"], "rows" => [[nil]]})
      iex> {:ok, r} = GraphlawEx.Result.new({:ok, s})
      iex> r.bindings
      [%{"x" => nil}]

      iex> refusal = GraphlawEx.Refusal.new(:call_timeout, "late")
      iex> GraphlawEx.Result.new({:error, refusal}) == {:error, refusal}
      true

  `{:error, %GraphlawEx.Refusal{}}` and `{:error, %GraphlawEx.Error{}}` pass
  through unchanged. `{:error, %GraphlawEx.Capability.Refusal{}}` (what the
  capability facade returns) is normalized to the canonical
  `GraphlawEx.Refusal` (same code, message, details, raw and broken term; class
  filled from the closed table, `nil` for client-only codes), so `new/1` never
  returns the capability struct. Anything else is
  `{:error, %GraphlawEx.Refusal{code: :malformed_response}}`.
  """
  @spec new(
          source()
          | {:ok, source()}
          | {:error, Refusal.t() | CapRefusal.t() | Error.t()}
          | term()
        ) ::
          {:ok, t()} | {:error, Refusal.t() | Error.t()}
  def new({:ok, decoded}), do: from_decoded(decoded)
  def new({:error, %Refusal{}} = error), do: error
  def new({:error, %CapRefusal{} = cap}), do: {:error, normalize_refusal(cap)}
  def new({:error, %Error{}} = error), do: error
  def new(decoded), do: from_decoded(decoded)

  @doc """
  Like `new/1` but returns the result or raises `GraphlawEx.Error.wrap/1` of the
  original refusal (its code and class are preserved, never rewritten); the
  offending value is never echoed.
  """
  @spec new!(
          source()
          | {:ok, source()}
          | {:error, Refusal.t() | CapRefusal.t() | Error.t()}
          | term()
        ) ::
          t()
  def new!(input) do
    case new(input) do
      {:ok, result} -> result
      {:error, refusal} -> raise Error.wrap(refusal)
    end
  end

  @doc """
  Builds and validates a canonical result from a decoded result struct.

      iex> alias GraphlawEx.Result.Sparql
      iex> s = Sparql.from_map(%{"variables" => ["x"], "rows" => [[%{"type" => "uri", "value" => "a"}]]})
      iex> {:ok, r} = GraphlawEx.Result.from_decoded(s)
      iex> {r.stats.rows, hd(r.bindings)["x"].value}
      {1, "a"}

      iex> alias GraphlawEx.Result.Sparql
      iex> s = Sparql.from_map(%{"variables" => ["x", "y"], "rows" => [[nil]]})
      iex> {:error, %GraphlawEx.Refusal{code: code}} = GraphlawEx.Result.from_decoded(s)
      iex> code
      :malformed_response
  """
  @spec from_decoded(term()) :: {:ok, t()} | {:error, Refusal.t()}
  def from_decoded(%Sparql{} = s) do
    with {:ok, bindings} <- bindings(s.variables, s.rows),
         quads = split_quads(s.nquads),
         :ok <- check_count(s.quads, quads),
         do: build(bindings, quads, nil, 0)
  end

  def from_decoded(%Canonical{} = c) do
    quads = split_quads(c.nquads)

    with :ok <- check_count(c.quads, quads),
         {:ok, id} <- digest(c.id),
         do: build([], quads, id, 0)
  end

  def from_decoded(%Admitted{} = a) do
    receipts = a.receipts
    last = receipts |> List.last() |> then(&(&1 && &1.plan_blake3))

    with {:ok, id} <- digest(last),
         do: build([], split_quads(a.nquads), id, length(receipts))
  end

  def from_decoded(other) do
    # Never echo the offending value: it may be a raw request (nquads, query
    # text, lease secret). Only its type and size are reported.
    {:error,
     Refusal.malformed_response("not a decodable result struct", %{
       "type" => type_name(other),
       "bytes" => :erlang.external_size(other)
     })}
  end

  defp normalize_refusal(%CapRefusal{} = cap) do
    %Refusal{
      code: cap.code,
      class: GraphlawEx.Refusal.Table.class(cap.code),
      message: cap.message,
      details: cap.details || %{},
      broken_term: cap.broken_term,
      raw: cap.raw
    }
  end

  defp type_name(%{__struct__: mod}), do: inspect(mod)
  defp type_name(v) when is_map(v), do: "map"
  defp type_name(v) when is_binary(v), do: "binary"
  defp type_name(v) when is_list(v), do: "list"
  defp type_name(v) when is_tuple(v), do: "tuple"
  defp type_name(v) when is_atom(v), do: "atom"
  defp type_name(_), do: "term"

  defp build(bindings, quads, id, receipts) do
    {:ok,
     %__MODULE__{
       bindings: bindings,
       quads: quads,
       receipt_blake3: id,
       stats: %{rows: length(bindings), quads: length(quads), receipts: receipts}
     }}
  end

  defp bindings(nil, nil), do: {:ok, []}
  defp bindings(nil, _rows), do: invalid("rows without variables")
  defp bindings(_vars, nil), do: {:ok, []}

  defp bindings(vars, rows) when is_list(vars) and is_list(rows) do
    if Enum.all?(vars, &is_binary/1),
      do: rows_to_bindings(vars, rows),
      else: invalid("variables must be strings")
  end

  defp bindings(_vars, _rows), do: invalid("variables and rows must be lists")

  defp rows_to_bindings(vars, rows) do
    width = length(vars)

    rows
    |> Enum.reduce_while({:ok, []}, fn
      row, {:ok, acc} when is_list(row) and length(row) == width ->
        cells = Enum.map(row, &cell/1)
        {:cont, {:ok, [vars |> Enum.zip(cells) |> Map.new() | acc]}}

      _row, _acc ->
        {:halt, invalid("row width does not match #{width} variables")}
    end)
    |> case do
      {:ok, acc} -> {:ok, Enum.reverse(acc)}
      error -> error
    end
  end

  defp cell(nil), do: nil
  defp cell(%Term{} = term), do: term
  defp cell(term), do: Term.from_map(term)

  defp split_quads(text) when is_binary(text) do
    text |> String.split("\n", trim: true) |> Enum.reject(&(String.trim(&1) == ""))
  end

  defp split_quads(_), do: []

  defp check_count(n, quads) when is_integer(n) do
    if n == length(quads), do: :ok, else: invalid("declared #{n} quads, found #{length(quads)}")
  end

  defp check_count(nil, _quads), do: :ok
  defp check_count(_other, _quads), do: invalid("quad count is not an integer")

  defp digest(nil), do: {:ok, nil}

  defp digest(hex) when is_binary(hex) do
    if hex =~ ~r/\A[0-9a-f]{64}\z/,
      do: {:ok, hex},
      else: invalid("receipt_blake3 is not 64 lowercase hex chars")
  end

  defp digest(_), do: invalid("receipt_blake3 is not a string")

  defp invalid(reason), do: {:error, Refusal.malformed_response(reason, %{"reason" => reason})}
end

defimpl Inspect, for: GraphlawEx.Result do
  # Counts only: bindings, quads and the receipt digest never reach logs.
  def inspect(%GraphlawEx.Result{stats: stats}, _opts) do
    stats = if is_map(stats), do: stats, else: %{}

    "#GraphlawEx.Result<rows: #{Map.get(stats, :rows, 0)}, quads: #{Map.get(stats, :quads, 0)}, " <>
      "receipts: #{Map.get(stats, :receipts, 0)}>"
  end
end
