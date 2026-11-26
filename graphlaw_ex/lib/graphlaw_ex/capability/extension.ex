# SPDX-FileCopyrightText: 2026 ash_graphlaw contributors <https://github.com/seanchatmangpt/ash_graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT

defmodule GraphlawEx.Capability.Extension do
  @moduledoc """
  Forwards hand-written per-op overrides into a projected capability module.

  `use GraphlawEx.Capability.Extension, op: "sparql"` emits `defdelegate`, `@doc` and
  `@spec` for each built-in override registered for that op, so the projected module
  carries no hand-written delegate block. The registry is open: pass `overrides:` to
  register overrides for any op without editing this module.

      use GraphlawEx.Capability.Extension,
        op: "shacl",
        overrides: [
          %{
            name: :admit,
            target: MyApp.ShaclAdmit,
            doc: "Routed admission.",
            args: [:data, :shapes, {:opts, []}],
            arg_types: [:binary, :binary, :keyword]
          }
        ]

  An override map carries `name` and `target` (required), plus optional `doc`, `args`
  (atom names; `{name, default}` for a defaulted trailing arg; default
  `[:nquads, :sparql, {:opts, []}]`), `arg_types` (`:binary | :keyword | :map | :term`, default
  `:term` each) and `returns` (a quoted type, default `{:ok, t()} | {:error, Refusal.t()}`).
  `arg_types` is a closed atom vocabulary (anything else raises `ArgumentError` listing it) while
  `returns` is a free type expression spliced as AST; the asymmetry is deliberate (arg types are
  data, a return type may be any typespec). A missing `name`/`target`, or a non-atom `target`,
  raises `ArgumentError`.
  When `overrides:` is omitted the built-in table is used; an op with no overrides raises
  `ArgumentError` at compile time. The built-in default remains `"sparql"`.
  """

  @default_args [:nquads, :sparql, {:opts, []}]
  @default_types [:binary, :binary, :keyword]

  @overrides %{
    "sparql" => [
      %{
        name: :query,
        target: GraphlawEx.Capability.SparqlQuery,
        doc: """
        FR-2 bulk query over raw RDF text. Delegates to `GraphlawEx.Capability.SparqlQuery.query/3`
        (hand-written; see that module for options).
        """,
        args: @default_args,
        arg_types: @default_types
      }
    ]
  }

  @doc "The built-in override table: wire op name to a list of override maps."
  @spec overrides() :: %{optional(String.t()) => [map()]}
  def overrides, do: @overrides

  defmacro __using__(opts) do
    op = Keyword.fetch!(opts, :op)

    entries =
      case Keyword.fetch(opts, :overrides) do
        {:ok, given} ->
          overrides_literal!(given, __CALLER__)

        :error ->
          case Map.fetch(@overrides, op) do
            {:ok, builtin} ->
              builtin

            :error ->
              raise ArgumentError, "no extension overrides registered for op #{inspect(op)}"
          end
      end

    Enum.map(entries, &emit/1)
  end

  # `overrides:` is read as AST, never evaluated. Each override is a literal map; every value is
  # a literal except `returns`, which is a type expression kept as AST and spliced into the @spec.
  defp overrides_literal!(ast, env) when is_list(ast),
    do: Enum.map(ast, &override_literal!(&1, env))

  defp overrides_literal!(ast, _env) do
    raise ArgumentError,
          "extension `overrides:` must be a literal list of maps, got: #{Macro.to_string(ast)}"
  end

  defp override_literal!({:%{}, _, kvs}, env) when is_list(kvs) do
    Map.new(kvs, fn
      {key, value} when is_atom(key) ->
        if key == :returns, do: {key, value}, else: {key, literal!(value, env)}

      {key, _} ->
        raise ArgumentError,
              "extension override keys must be literal atoms, got: #{Macro.to_string(key)}"
    end)
  end

  defp override_literal!(ast, _env) do
    raise ArgumentError,
          "extension override must be a literal map, got: #{Macro.to_string(ast)}"
  end

  @doc false
  @spec literal!(Macro.t(), Macro.Env.t()) :: term()
  def literal!(ast, _env) when is_atom(ast) or is_binary(ast) or is_number(ast), do: ast
  def literal!(ast, env) when is_list(ast), do: Enum.map(ast, &literal!(&1, env))
  def literal!({a, b}, env), do: {literal!(a, env), literal!(b, env)}

  def literal!({:{}, _, items}, env) when is_list(items),
    do: items |> Enum.map(&literal!(&1, env)) |> List.to_tuple()

  def literal!({:%{}, _, kvs}, env) when is_list(kvs),
    do: Map.new(kvs, fn {k, v} -> {literal!(k, env), literal!(v, env)} end)

  def literal!({:__aliases__, _, _} = ast, env) do
    case Macro.expand(ast, env) do
      mod when is_atom(mod) -> mod
      _ -> raise ArgumentError, "non-literal extension override value: #{Macro.to_string(ast)}"
    end
  end

  def literal!(ast, _env) do
    raise ArgumentError, "non-literal extension override value: #{Macro.to_string(ast)}"
  end

  defp emit(%{name: name, target: target} = entry) when is_atom(name) and is_atom(target) do
    doc = Map.get(entry, :doc, "Hand-written override `#{name}`; see `#{inspect(target)}`.")
    args = Map.get(entry, :args, @default_args)
    types = Map.get(entry, :arg_types, List.duplicate(:term, length(args)))

    if length(types) != length(args) do
      raise ArgumentError,
            "extension override #{inspect(name)}: args and arg_types differ in length"
    end

    arg_asts =
      Enum.map(args, fn
        {arg, default} -> {:\\, [], [Macro.var(arg, nil), Macro.escape(default)]}
        arg -> Macro.var(arg, nil)
      end)

    type_asts = Enum.map(types, &type_ast/1)

    returns =
      Map.get_lazy(entry, :returns, fn ->
        quote do: {:ok, t()} | {:error, GraphlawEx.Capability.Refusal.t()}
      end)

    quote do
      @doc unquote(doc)
      @spec unquote(name)(unquote_splicing(type_asts)) :: unquote(returns)
      defdelegate unquote(name)(unquote_splicing(arg_asts)), to: unquote(target)
    end
  end

  defp emit(entry) do
    raise ArgumentError,
          "extension override needs an atom :name and an atom :target module, got: " <>
            "name=#{inspect(Map.get(entry, :name, :missing))} " <>
            "target=#{inspect(Map.get(entry, :target, :missing))}"
  end

  @type_vocabulary [:binary, :keyword, :map, :term]

  defp type_ast(:binary), do: quote(do: binary())
  defp type_ast(:keyword), do: quote(do: keyword())
  defp type_ast(:map), do: quote(do: map())
  defp type_ast(:term), do: quote(do: term())

  defp type_ast(other) do
    raise ArgumentError,
          "unsupported extension arg_type #{inspect(other)}; expected one of " <>
            inspect(@type_vocabulary)
  end
end
