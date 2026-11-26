# SPDX-License-Identifier: MIT
# projection target: ontologies/graphlaw-governance.ttl; hand-written until projected
defmodule GraphlawEx.Refusal.Table do
  @moduledoc """
  Pure data for the closed refusal table: codes, classes, broken terms, docs,
  the engine wire-code map, client codes, constructor codes and payload lift
  keys. No policy lives here; `GraphlawEx.Refusal` owns the logic.

  `:ceiling_unmet` is a client-only entry: it has class, broken term and doc
  here, but is not in `codes/0` (the host table stays closed).

      iex> GraphlawEx.Refusal.Table.class(:saturated)
      :blocked_resource

      iex> GraphlawEx.Refusal.Table.class(:ceiling_unmet)
      :refused_structure

      iex> GraphlawEx.Refusal.Table.class(:nope)
      nil
  """

  @codes [
    :saturated,
    :call_timeout,
    :call_trapped,
    :call_exited,
    :fuel_exhausted,
    :wasm_digest_mismatch,
    :wasm_import_surface_mismatch,
    :malformed_response,
    :invalid_json,
    :invalid_encoding,
    :resource_limit,
    :engine_unclassified,
    :invalid_capability_request,
    :unknown_capability,
    :capability_response_undecodable,
    :policy_refused,
    :not_admitted,
    :plan_refused,
    :receipt_required,
    :engine_refused,
    :lease_revoked,
    :receipt_refused,
    :lease_refused,
    :unverified_lease_refused,
    :fuse,
    :duckdb_failure,
    :batch_member_failed
  ]

  @classes [:blocked_resource, :refused_structure, :refused_identity, :unsupported]

  @broken_terms [:mu_on_O, :R_missing_consequence, :R_missing_identity, :R_missing_standing]

  # code => {class, broken_term, doc}
  @entries %{
    saturated:
      {:blocked_resource, :R_missing_consequence,
       "The host queue is full (max_queue), so the request was shed instead of waiting."},
    call_timeout:
      {:blocked_resource, :R_missing_consequence,
       "The call did not finish within the configured timeout_ms."},
    call_trapped:
      {:blocked_resource, :R_missing_consequence,
       "The WASM call trapped while executing; the instance is recycled."},
    call_exited:
      {:blocked_resource, :R_missing_consequence,
       "The WASM program exited during the call; the instance is recycled."},
    fuel_exhausted:
      {:blocked_resource, :R_missing_consequence,
       "The call used up its fuel budget before the engine returned."},
    wasm_digest_mismatch:
      {:refused_identity, :R_missing_identity,
       "The sha256 of the WASM bytes differs from the pinned digest, so the module is not the admitted GraphLaw release."},
    wasm_import_surface_mismatch:
      {:refused_identity, :R_missing_identity,
       "The module imports outside the manifest's closed WASI allowlist, so it is not the pinned GraphLaw shape."},
    malformed_response:
      {:refused_structure, :mu_on_O,
       "The engine response was valid JSON but lacked the expected envelope fields."},
    invalid_json:
      {:refused_structure, :mu_on_O,
       "A request could not be encoded as JSON, or an engine response was not valid JSON."},
    invalid_encoding:
      {:refused_structure, :mu_on_O,
       "The request contained text that is not valid UTF-8 and was refused before reaching the engine."},
    resource_limit:
      {:blocked_resource, :R_missing_consequence,
       "A request or response exceeded a size cap, or the engine hit a resource limit, before it could decide."},
    engine_unclassified:
      {:unsupported, :R_missing_standing,
       "The engine returned a refusal variant this library does not know; it is carried verbatim and classified unsupported rather than guessed."},
    invalid_capability_request:
      {:refused_structure, :mu_on_O,
       "A typed capability request failed validation before the engine was called."},
    unknown_capability:
      {:refused_structure, :mu_on_O,
       "The named capability is not in the GraphLaw capability registry."},
    capability_response_undecodable:
      {:refused_structure, :mu_on_O,
       "The engine returned a capability response that is not a JSON object, so it cannot be decoded into a typed result."},
    policy_refused:
      {:refused_structure, :mu_on_O, "The engine refused the FOND policy as not strong-cyclic."},
    not_admitted: {:refused_structure, :mu_on_O, "The engine did not admit the law chain."},
    plan_refused: {:refused_structure, :mu_on_O, "The engine refused the plan step."},
    receipt_required:
      {:refused_identity, :R_missing_identity,
       "The engine required a receipt the caller did not supply."},
    engine_refused:
      {:unsupported, :R_missing_standing,
       "The engine refused without a more specific known code."},
    lease_revoked:
      {:refused_identity, :R_missing_standing,
       "An Art.14 operator revocation epoch covers this engine; the call was shed before any engine mutation."},
    receipt_refused:
      {:refused_identity, :R_missing_identity,
       "The engine refused to mint or accept a receipt; the admission has no identity anchor."},
    lease_refused:
      {:refused_identity, :R_missing_standing,
       "The engine refused the lease; the caller holds no standing for the operation."},
    unverified_lease_refused:
      {:refused_structure, :mu_on_O,
       "The lease could not be verified against the admitted structure, so the operation was refused."},
    fuse:
      {:refused_structure, :mu_on_O,
       "The engine's fuse tripped on a malformed or unadmitted structure before any evaluation."},
    duckdb_failure:
      {:blocked_resource, :R_missing_consequence,
       "The engine's DuckDB statement failed to execute, so no consequence was produced."},
    batch_member_failed:
      {:blocked_resource, :R_missing_consequence,
       "One batch member failed; the batch's partial receipt and observed/max counts are carried in details."},
    # client-only (GL28-7 authority pre-gate); not in @codes
    ceiling_unmet:
      {:refused_structure, :R_missing_standing,
       "The authority pre-gate refused the call before engine dispatch: the presented lease claims a ceiling below the op's floor."}
  }

  @engine_codes %{
    "ResourceLimit" => :resource_limit,
    "PolicyRefused" => :policy_refused,
    "NotAdmitted" => :not_admitted,
    "PlanRefused" => :plan_refused,
    "ReceiptRequired" => :receipt_required,
    "ReceiptRefused" => :receipt_refused,
    "LeaseRefused" => :lease_refused,
    "UnverifiedLeaseRefused" => :unverified_lease_refused,
    "Refused" => :engine_refused,
    "Fuse" => :fuse,
    "DuckDb" => :duckdb_failure,
    "BatchMemberFailed" => :batch_member_failed
  }

  @capability_client_codes [
    :invalid_capability_request,
    :unknown_capability,
    :capability_response_undecodable
  ]

  @constructor_codes [
    :lease_revoked,
    :saturated,
    :call_timeout,
    :call_trapped,
    :call_exited,
    :fuel_exhausted,
    :wasm_digest_mismatch,
    :wasm_import_surface_mismatch,
    :malformed_response,
    :invalid_json,
    :invalid_encoding,
    :resource_limit,
    :engine_unclassified
  ]

  # Engine payload keys lifted into `details` per code (kernel json! keys).
  @lift_keys %{
    receipt_refused: ["step", "reason"],
    lease_refused: ["reason", "lease_id", "step"],
    unverified_lease_refused: ["reason", "lease_id", "step"],
    batch_member_failed: ["member", "message", "partial_receipt", "limit", "observed", "max"]
  }

  @doc "Every closed host-table refusal code, in table order."
  @spec codes() :: [atom()]
  def codes, do: @codes

  @doc "Every refusal class."
  @spec classes() :: [atom()]
  def classes, do: @classes

  @doc "Every failure-taxonomy term."
  @spec broken_terms() :: [atom()]
  def broken_terms, do: @broken_terms

  @doc "Engine wire code string to refusal code."
  @spec engine_codes() :: %{String.t() => atom()}
  def engine_codes, do: @engine_codes

  @doc "Capability-layer client codes (subset of `codes/0`)."
  @spec capability_client_codes() :: [atom()]
  def capability_client_codes, do: @capability_client_codes

  @doc "Codes that have a public per-code constructor."
  @spec constructor_codes() :: [atom()]
  def constructor_codes, do: @constructor_codes

  @doc "Payload keys lifted into `details` for a code (`[]` when none)."
  @spec lift_keys(atom()) :: [String.t()]
  def lift_keys(code), do: Map.get(@lift_keys, code, [])

  @doc "Class of a code (closed codes and client-only entries), or `nil`."
  @spec class(term()) :: atom() | nil
  def class(code), do: pick(code, 0)

  @doc "Broken term of a code, or `nil`."
  @spec broken_term(term()) :: atom() | nil
  def broken_term(code), do: pick(code, 1)

  @doc "One-line doc of a code, or `nil`."
  @spec doc(term()) :: String.t() | nil
  def doc(code), do: pick(code, 2)

  defp pick(code, idx) do
    case Map.fetch(@entries, code) do
      {:ok, entry} -> elem(entry, idx)
      :error -> nil
    end
  end
end
