# SPDX-License-Identifier: Apache-2.0
# UNSUPPORTED(generator-capability): hand-written exception wrapper. graphlaw_ex
# has no Splode dependency, so this is a plain `defexception` mirroring the
# shape of ash_graphlaw's `AshGraphLaw.Error.Refused` (fields `[:refusal]`,
# message "GraphLaw refused: <code>").

defmodule GraphlawEx.Error.Refused do
  @moduledoc """
  Exception wrapping a capability-layer `GraphlawEx.Capability.Refusal`.

  Raised by every `GraphlawEx.Capability.API.op!/2` function when the
  corresponding `op/2` returns `{:error, refusal}`. The typed refusal stays
  intact on the exception: recover it with `refusal/1`, read its code with
  `code/1`, its lossless engine error map with `raw/1` and its Chatman
  failure-taxonomy term with `broken_term/1`.
  """

  defexception [:refusal]

  alias GraphlawEx.Capability.Refusal

  @type t :: %__MODULE__{refusal: Refusal.t() | nil}

  @doc "The refusal code (for example `:invalid_capability_request`), or `nil` when no refusal is attached."
  @spec code(t()) :: atom() | nil
  def code(%__MODULE__{refusal: %Refusal{code: code}}), do: code
  def code(%__MODULE__{}), do: nil

  @doc "The whole engine error map the refusal was decoded from (lossless), or `nil` for a client-side refusal."
  @spec raw(t()) :: map() | nil
  def raw(%__MODULE__{refusal: %Refusal{} = refusal}), do: Map.get(refusal, :raw)
  def raw(%__MODULE__{}), do: nil

  @doc "The Chatman failure-taxonomy term of the refusal, or `nil`."
  @spec broken_term(t()) :: atom() | nil
  def broken_term(%__MODULE__{refusal: %Refusal{broken_term: term}}), do: term
  def broken_term(%__MODULE__{}), do: nil

  @doc """
  Human-readable message: `"GraphLaw refused: <code>"`. A fallback clause
  covers an error built without a refusal (`"GraphLaw refused: unknown"`).
  """
  @impl true
  @spec message(t()) :: String.t()
  def message(%__MODULE__{refusal: %Refusal{code: code}}) when not is_nil(code),
    do: "GraphLaw refused: #{code}"

  def message(%__MODULE__{}), do: "GraphLaw refused: unknown"
end
