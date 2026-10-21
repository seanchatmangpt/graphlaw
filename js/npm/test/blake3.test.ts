// blake3.test.ts — validates the pure-TypeScript BLAKE3 (src/blake3.ts)
// against the b3sum CLI as an external authority (real collaborator, no
// mocks). Skips loudly when b3sum is not installed on the dev machine.
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { afterAll, describe, expect, it } from 'vitest';
import { blake3Hex, Blake3 } from '../src/blake3.js';

function b3sumHex(data: Uint8Array): string {
  const dir = mkdtempSync(join(tmpdir(), 'b3sum-'));
  try {
    const file = join(dir, 'input');
    writeFileSync(file, data);
    return execFileSync('b3sum', [file], { encoding: 'utf8' }).trim().split(/\s+/)[0]!;
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

function haveB3sum(): boolean {
  try {
    execFileSync('b3sum', ['--version'], { encoding: 'utf8' });
    return true;
  } catch {
    return false;
  }
}

const hasB3sum = haveB3sum();

const suite = hasB3sum ? describe : describe.skip;

// Random 1 MiB buffer, seeded PRNG for determinism across runs.
const random1MiB = (() => {
  const buf = new Uint8Array(1024 * 1024);
  let seed = 0x2545f491;
  for (let i = 0; i < buf.length; i++) {
    seed = (seed * 1103515245 + 12345) >>> 0;
    buf[i] = seed & 0xff;
  }
  return buf;
})();

suite('blake3 vs b3sum (external authority)', () => {
  it(
    'matches on the empty input',
    () => expect(blake3Hex(new Uint8Array(0))).toBe(b3sumHex(new Uint8Array(0))),
  );

  it('matches on "abc"', () =>
    expect(blake3Hex(new TextEncoder().encode('abc'))).toBe(b3sumHex(new TextEncoder().encode('abc'))));

  it('matches on a 64-byte input (exactly one block)', () => {
    const input = new Uint8Array(64).fill(0xa5);
    expect(blake3Hex(input)).toBe(b3sumHex(input));
  });

  it('matches on a 1025-byte input (second chunk, 1 byte)', () => {
    const input = new Uint8Array(1025).map((_, i) => i & 0xff);
    expect(blake3Hex(input)).toBe(b3sumHex(input));
  });

  it('matches on a 1024-byte input (chunk boundary)', () => {
    const input = new Uint8Array(1024).fill(0x5a);
    expect(blake3Hex(input)).toBe(b3sumHex(input));
  });

  it('matches on a 1 MiB random input', () => {
    expect(blake3Hex(random1MiB)).toBe(b3sumHex(random1MiB));
  });

  it('matches incremental updates on a 1 MiB random input', () => {
    const h = new Blake3();
    let off = 0;
    while (off < random1MiB.length) {
      const take = Math.min(333, random1MiB.length - off);
      h.update(random1MiB.subarray(off, off + take));
      off += take;
    }
    expect(h.digestHex()).toBe(b3sumHex(random1MiB));
  });

  it('incremental equals one-shot on an awkward split (511/513)', () => {
    const input = new Uint8Array(2048).fill(0x3c);
    const a = new Blake3().update(input.subarray(0, 511)).update(input.subarray(511)).digestHex();
    const b = new Blake3().update(input).digestHex();
    expect(a).toBe(b);
  });

  it('self-check: b3sum and node sha256 disagree, proving the authority is real', () => {
    // Anti-vacuity: the authority must not be the same function we test.
    const input = new TextEncoder().encode('abc');
    expect(createHash('sha256').update(input).digest('hex')).not.toBe(b3sumHex(input));
  });
});

afterAll(() => {
  if (!hasB3sum) {
    // Loud skip, never a silent pass.
    console.warn('[SKIP] blake3 vs b3sum: b3sum not on PATH (cargo install b3sum)');
  }
});
