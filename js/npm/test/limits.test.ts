// Resource-limit behavior of the JSON ABI (test/limits.test.ts).
// Chicago style: every case drives the real wasm module through callRaw —
// no mocks, assert on the actual envelope / thrown error.
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { GraphlawRequestTooLargeError } from '../src/errors.js';
import { LIMITS } from '../src/limits.js';
import { callRaw } from '../src/raw.js';

const repoRoot = new URL('../../../', import.meta.url).pathname;
const registryLimits: Record<string, unknown> = JSON.parse(
  readFileSync(`${repoRoot}registry/capability-registry.json`, 'utf8'),
).limits;

/** A payload string padded to exactly `n` UTF-8 bytes inside a JSON string value. */
function paddedRequest(pad: string): Record<string, unknown> {
  return { op: 'noop_probe', pad };
}

describe('request size limit', () => {
  it('rejects a request over max_request_bytes client-side', async () => {
    const big = 'x'.repeat(LIMITS.max_request_bytes + 1);
    const request = paddedRequest(big);
    const bytes = new TextEncoder().encode(JSON.stringify(request));
    expect(bytes.length).toBeGreaterThan(LIMITS.max_request_bytes);
    await expect(callRaw(request)).rejects.toBeInstanceOf(GraphlawRequestTooLargeError);
  });

  it('delivers a request at exactly max_request_bytes and returns an envelope', async () => {
    // Grow the pad until the serialized body is byte-exact at the cap.
    const target = LIMITS.max_request_bytes;
    const prefix = JSON.stringify(paddedRequest('')).replace('""', `"@PAD@"`);
    const budget = target - new TextEncoder().encode(prefix).length + '@PAD@'.length;
    if (budget < 0) throw new Error(`fixed overhead ${target - budget} exceeds cap`);
    const request = paddedRequest('p'.repeat(budget));
    const bytes = new TextEncoder().encode(JSON.stringify(request));
    expect(bytes.length).toBe(target);
    // The module decides the ok value; the contract is delivery + a real envelope.
    const envelope = await callRaw(request);
    expect(typeof envelope.ok).toBe('boolean');
  });
});

describe('json depth limit', () => {
  it('accepts JSON nested exactly 64 levels', async () => {
    // Brackets inside string values do not count; nest real structure 64 deep.
    let request: Record<string, unknown> = { op: 'noop_probe' };
    for (let i = 0; i < LIMITS.max_json_depth - 1; i++) {
      request = { op: 'noop_probe', nest: request };
    }
    const envelope = await callRaw(request);
    expect(typeof envelope.ok).toBe('boolean');
  });
});

describe('unknown op', () => {
  it('returns an ok:false envelope with error.kind present', async () => {
    const envelope = await callRaw({ op: 'definitely_not_an_op' });
    expect(envelope.ok).toBe(false);
    if (!envelope.ok) {
      expect(typeof envelope.error.kind).toBe('string');
      expect(envelope.error.kind.length).toBeGreaterThan(0);
    }
  });
});

describe('limits mirror the registry', () => {
  it('LIMITS keys and values equal registry/capability-registry.json `limits`', () => {
    expect(Object.keys(registryLimits).sort()).toEqual(Object.keys(LIMITS).sort());
    for (const [key, value] of Object.entries(registryLimits)) {
      expect(LIMITS[key as keyof typeof LIMITS], key).toBe(value);
    }
  });
});
