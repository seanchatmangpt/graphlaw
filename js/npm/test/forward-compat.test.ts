// Forward-compatibility suite over registry/op-examples.json. Mirrors the
// invariant of tests/op_forward_compat.rs: unknown top-level keys injected
// into valid example requests never change outcomes and never echo back into
// responses; unknown ops get a refused envelope with a typed error; the
// capabilities surface is exactly 14 ops at abi_version 1.
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { callRaw } from '../src/raw.js';

interface OpExample {
  id: string;
  op: string;
  outcome: 'ok' | 'refused';
  request: Record<string, unknown>;
}

const repoRoot = new URL('../../../', import.meta.url).pathname;
const examples: OpExample[] = JSON.parse(
  readFileSync(`${repoRoot}registry/op-examples.json`, 'utf8'),
).examples;

// First `ok` example per op.
const firstOkPerOp: OpExample[] = Object.values(
  examples
    .filter((e) => e.outcome === 'ok')
    .reduce<Record<string, OpExample>>((acc, e) => {
      if (!(e.op in acc)) acc[e.op] = e;
      return acc;
    }, {}),
);

const INJECTED_KEYS = ['pad', 'x', 'deep', 'blob64k'] as const;
const BIG_STRING = 'a'.repeat(64 * 1024);

function injectUnknowns(request: Record<string, unknown>): Record<string, unknown> {
  return {
    ...request,
    pad: null,
    x: 1,
    deep: { nested: { deeper: [1, { still: 'unknown' }] } },
    blob64k: BIG_STRING,
  };
}

function responseContainsKey(value: unknown, keys: readonly string[]): boolean {
  if (Array.isArray(value)) return value.some((v) => responseContainsKey(v, keys));
  if (value !== null && typeof value === 'object') {
    return Object.entries(value).some(
      ([k, v]) => keys.includes(k) || responseContainsKey(v, keys),
    );
  }
  return false;
}

describe('forward compatibility', () => {
  it.each(firstOkPerOp.map((e) => [e.op, e] as const))(
    'unknown top-level keys are inert (%s)',
    async (_op, example) => {
      const baseline = await callRaw(example.request);
      expect(baseline.ok, JSON.stringify(baseline)).toBe(true);

      const injected = await callRaw(injectUnknowns(example.request));
      expect(injected.ok, JSON.stringify(injected)).toBe(true);
      expect(injected).toEqual(baseline);
      expect(responseContainsKey(injected, INJECTED_KEYS)).toBe(false);
    },
  );

  it('unknown op returns refused envelope with typed error', async () => {
    const envelope = await callRaw({ op: 'zzzz_unknown' });
    expect(envelope.ok, JSON.stringify(envelope)).toBe(false);
    if (!envelope.ok) {
      expect(envelope.error.kind).toBeDefined();
      expect(typeof envelope.error.kind).toBe('string');
    }
  });

  it('capabilities exposes exactly 14 ops at abi_version 1', async () => {
    const envelope = await callRaw({ op: 'capabilities' });
    expect(envelope.ok, JSON.stringify(envelope)).toBe(true);
    if (envelope.ok) {
      // Ok envelopes spread payload fields at the top level (AbiOk<T>).
      const caps = envelope as unknown as { ops: unknown[]; abi_version: number };
      expect(Array.isArray(caps.ops)).toBe(true);
      expect(caps.ops).toHaveLength(14);
      expect(caps.abi_version).toBe(1);
    }
  });
});
