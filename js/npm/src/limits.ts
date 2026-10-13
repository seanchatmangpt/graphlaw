/**
 * ABI limits, mirrored from registry/capability-registry.json `limits`.
 *
 * This file is a HAND-MAINTAINED MIRROR of the registry document (not generated):
 * scripts/gen-types.mjs owns src/generated/types.ts; this module intentionally stays
 * the single source for LIMITS/ABI_VERSION so the npm package does not depend on the
 * generator to ship. Drift is guarded by test/limits.test.ts, which compares these
 * values against registry/capability-registry.json itself — a registry change that is
 * not mirrored here fails the test, and vice versa.
 *
 * Mirrored from registry document with:
 *   registry_sha256: sha256:1c129dd6eed90bc680f8e695070dcef2eccf0eab7ff4d2b9c0462b2439c514b9
 *
 * Registry `limits` (key -> value):
 *
 * | key                          | value     |
 * |------------------------------|-----------|
 * | hooks_max_firings            | 10000     |
 * | hooks_max_rounds             | 64        |
 * | hooks_max_state_quads        | 1000000   |
 * | max_atoms_per_field          | 10000     |
 * | max_json_depth               | 64        |
 * | max_outstanding_alloc_bytes  | 268435456 |
 * | max_plan_actions             | 1000      |
 * | max_plan_total_atoms         | 100000    |
 * | max_policy_entries           | 100000    |
 * | max_request_bytes            | 16777216  |
 * | n3_max_derived_facts         | 500000    |
 * | n3_max_iterations            | 4000      |
 * | n3_max_match_steps           | 50000000  |
 * | n3_max_term_bytes            | 65536     |
 * | n3_max_total_bytes           | 268435456 |
 *
 * To refresh: update LIMITS below to match the registry, update the table above,
 * and run the drift-guard test.
 */
export const LIMITS = {
  hooks_max_firings: 10000,
  hooks_max_rounds: 64,
  hooks_max_state_quads: 1000000,
  max_atoms_per_field: 10000,
  max_json_depth: 64,
  max_outstanding_alloc_bytes: 268435456,
  max_plan_actions: 1000,
  max_plan_total_atoms: 100000,
  max_policy_entries: 100000,
  max_request_bytes: 16777216,
  n3_max_derived_facts: 500000,
  n3_max_iterations: 4000,
  n3_max_match_steps: 50000000,
  n3_max_term_bytes: 65536,
  n3_max_total_bytes: 268435456,
} as const;

/** Union of the limit names (registry `limits` keys) mirrored in {@link LIMITS}. */
export type LimitName = keyof typeof LIMITS;

/** ABI version this package is built against. Registry `abi_version`. */
export const ABI_VERSION = 1;
