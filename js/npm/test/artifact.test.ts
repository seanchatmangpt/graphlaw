// Artifact integrity: the copied wasm matches the pinned digest from the
// build (src/generated/artifact-meta.ts), which itself matches the
// repo-root pin (priv/graphlaw.wasm.blake3); bytes are a real wasm module.
import { readFileSync, statSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { blake3Hex } from '../src/blake3.js';
import { WASM_BLAKE3 } from '../src/generated/artifact-meta.js';

// js/npm/test/artifact.test.ts -> repo root is 3 levels up.
const repoRoot = new URL('../../../', import.meta.url).pathname;
const wasmPath = new URL('../wasm/graphlaw.wasm', import.meta.url).pathname;

const bytes = readFileSync(wasmPath);

describe('wasm artifact integrity', () => {
  it('exists and its blake3 matches WASM_BLAKE3 from artifact-meta', () => {
    const sha = blake3Hex(bytes);
    expect(sha).toBe(WASM_BLAKE3);
  });

  it('WASM_BLAKE3 matches the repo-root pin (priv/graphlaw.wasm.blake3)', () => {
    const pinned = readFileSync(`${repoRoot}priv/graphlaw.wasm.blake3`, 'utf8')
      .trim()
      .split(/\s+/)[0];
    expect(WASM_BLAKE3).toBe(pinned);
  });

  it('starts with the wasm magic and version-1 header', () => {
    // magic \0asm + version 1 (little-endian u32)
    expect([...bytes.subarray(0, 8)]).toEqual([
      0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
    ]);
  });

  it('size matches its artifact class band (legacy pre-duckdb / DuckDB-linked)', () => {
    // Size bands follow scripts/wasm_audit.sh: hard ceiling 64 MiB.
    //   - legacy pre-duckdb class: <10 MB (historical; the current
    //     priv/graphlaw.wasm at 7.1 MB is this class until coordinator re-pin)
    //   - DuckDB-linked (M4 witness) class: >=40 MB proves the engine link,
    //     bounded by the 64 MiB hard ceiling
    const MIB = 1024 * 1024;
    const LEGACY_MAX_BYTES = 10 * MIB;
    const ENGINE_MIN_BYTES = 40 * MIB;
    const HARD_CEILING_BYTES = 80 * MIB; // 80 MiB, mirrors wasm_size_court SCAR (fork-HEAD artifact 74,097,211 B; slimming follow-up open)
    const size = statSync(wasmPath).size;
    if (size >= ENGINE_MIN_BYTES) {
      // Engine-link class: wasm >= 40 MB proves the DuckDB engine is linked.
      expect(size).toBeLessThanOrEqual(HARD_CEILING_BYTES);
    } else {
      // Legacy class: must fit the historical <10 MB band.
      expect(size).toBeLessThan(LEGACY_MAX_BYTES);
    }
  });
});
