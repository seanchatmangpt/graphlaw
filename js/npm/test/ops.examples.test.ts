// Table-driven golden-vector suite over registry/op-examples.json (74
// examples, 44 ok / 30 refused, all 14 ops). Mirrors the guarantees of
// tests/op_examples.rs: for `ok` assert envelope.ok; for `refused` assert
// error.kind and error.details?.code match the declared refusal identity.
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { callRaw } from '../src/raw.js';

interface OpExample {
  id: string;
  op: string;
  description: string;
  outcome: 'ok' | 'refused';
  positive_control: boolean;
  refusal_kind: string | null;
  refusal_code: string | null;
  request: Record<string, unknown>;
}

// js/npm/test/ops.examples.test.ts -> repo root is 3 levels up.
const repoRoot = new URL('../../../', import.meta.url).pathname;
const doc: unknown = JSON.parse(
  readFileSync(`${repoRoot}registry/op-examples.json`, 'utf8'),
);
expect(doc, 'registry/op-examples.json must be an object').toHaveProperty(
  'examples',
);
const examples: OpExample[] = (doc as { examples: OpExample[] }).examples;
expect(examples.length, 'examples file must be non-empty').toBe(74);

describe('op-examples golden vectors', () => {
  it.each(examples.map((e) => [e.id, e] as const))('%s', async (_id, example) => {
    const envelope = await callRaw(example.request);
    if (example.outcome === 'ok') {
      expect(envelope.ok, JSON.stringify(envelope)).toBe(true);
    } else {
      expect(envelope.ok, JSON.stringify(envelope)).toBe(false);
      if (!envelope.ok) {
        expect(envelope.error.kind).toBe(example.refusal_kind);
        const code = envelope.error.details?.code ?? null;
        expect(code).toBe(example.refusal_code);
      }
    }
  });
});
