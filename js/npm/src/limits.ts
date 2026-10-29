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
 *   registry_blake3: blake3:e227367e13ce8110d97fe29f252bdb69e22e872a5be4117ccbce4bbc9c6477d7
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
 * | max_outstanding_alloc_bytes  | 16777216  |
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
  max_outstanding_alloc_bytes: 16777216,
  max_plan_actions: 1000,
  max_plan_total_atoms: 100000,
  max_policy_entries: 100000,
  max_request_bytes: 16777216,
  n3_max_derived_facts: 500000,
  n3_max_iterations: 4000000,
  n3_max_match_steps: 200000,
  n3_max_term_bytes: 65536,
  n3_max_total_bytes: 268435456,
  n3_max_backward_depth: 32,
  n3_max_backward_solutions_per_goal: 1024,
  n3_max_term_depth: 64,
  n3_max_total_steps: 50000000,
} as const;

/** Union of the limit names (registry `limits` keys) mirrored in {@link LIMITS}. */
export type LimitName = keyof typeof LIMITS;

/** ABI version this package is built against. Registry `abi_version`. */
export const ABI_VERSION = 1;
