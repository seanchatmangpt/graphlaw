// Artifact integrity: the copied wasm matches the pinned digest from the
// build (src/generated/artifact-meta.ts), which itself matches the
// repo-root pin (priv/graphlaw.wasm.sha256); bytes are a real wasm module.
import { createHash } from 'node:crypto';
import { readFileSync, statSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { WASM_SHA256 } from '../src/generated/artifact-meta.js';

// js/npm/test/artifact.test.ts -> repo root is 3 levels up.
const repoRoot = new URL('../../../', import.meta.url).pathname;
const wasmPath = new URL('../wasm/graphlaw.wasm', import.meta.url).pathname;

const bytes = readFileSync(wasmPath);

describe('wasm artifact integrity', () => {
  it('exists and its sha256 matches WASM_SHA256 from artifact-meta', () => {
    const sha = createHash('sha256').update(bytes).digest('hex');
    expect(sha).toBe(WASM_SHA256);
  });

  it('WASM_SHA256 matches the repo-root pin (priv/graphlaw.wasm.sha256)', () => {
    const pinned = readFileSync(`${repoRoot}priv/graphlaw.wasm.sha256`, 'utf8')
      .trim()
      .split(/\s+/)[0];
    expect(WASM_SHA256).toBe(pinned);
  });

  it('starts with the wasm magic and version-1 header', () => {
    // magic \0asm + version 1 (little-endian u32)
    expect([...bytes.subarray(0, 8)]).toEqual([
      0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
    ]);
  });

  it('size is within the 5MB-9MB registry sanity band', () => {
    const size = statSync(wasmPath).size;
    expect(size).toBeGreaterThanOrEqual(5 * 1024 * 1024);
    expect(size).toBeLessThanOrEqual(9 * 1024 * 1024);
  });
});
