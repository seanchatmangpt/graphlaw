// Tests for initGraphlaw (src/init.ts): digest gate, error taxonomy, and the
// per-process module cache. Real collaborator: the checked-in
// priv/graphlaw.wasm artifact (copied to wasm/ by scripts/build.mjs), no mocks.
import { describe, expect, it } from 'vitest';
import { initGraphlaw, GraphlawInitError, GraphlawIntegrityError } from '../src/init.js';
import { WASM_BLAKE3 } from '../src/generated/artifact-meta.js';

// Not a wasm module: WebAssembly.compile rejects it, so the sha-skipped path
// fails inside instantiateFromBytes with GraphlawInitError (compile error cause).
const GARBAGE_BYTES = new Uint8Array([0, 1, 2, 3]);

// Valid wasm magic/version header with a corrupt body: passes the magic sniff,
// fails module validation -> GraphlawInitError, never the digest gate.
const TAMPERED_BYTES = (() => {
  const b = new Uint8Array(64);
  b.set([0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]); // \0asm v1
  b.fill(0xff, 8);
  return b;
})();

describe('initGraphlaw', () => {
  it(
    'resolves with skipDigestCheck and serves capabilities over the real artifact',
    { timeout: 60_000 },
    async () => {
      const abi = await initGraphlaw({ skipDigestCheck: true });
      const res = (await abi.call({ op: 'capabilities' })) as {
        ok: boolean;
        abi_version?: number;
      };
      expect(res.ok).toBe(true);
      if (res.ok) expect(res.abi_version).toBe(1);
    },
  );

  it(
    'resolves with default options (digest check passes against the real artifact)',
    { timeout: 60_000 },
    async () => {
      const abi = await initGraphlaw();
      expect(abi).toBeDefined();
      const res = (await abi.call({ op: 'capabilities' })) as { ok: boolean };
      expect(res.ok).toBe(true);
    },
  );
});

describe('initGraphlaw cache', () => {
  it('two sequential default calls resolve to the same engine instance', async () => {
    const p1 = initGraphlaw();
    const p2 = initGraphlaw();
    const [a1, a2] = await Promise.all([p1, p2]);
    expect(a1).toBe(a2);
  });
});

describe('initGraphlaw failure paths', () => {
  // The digest gate is `if (!options.skipDigestCheck)` — custom bytes are checked
  // against the pin unless the caller opts out explicitly.
  it('rejects with GraphlawIntegrityError on digest mismatch (wasmBytes, no skip)', async () => {
    await expect(initGraphlaw({ wasmBytes: GARBAGE_BYTES })).rejects.toBeInstanceOf(
      GraphlawIntegrityError,
    );
  });

  it('reports the pinned digest in the mismatch error', async () => {
    const err = await initGraphlaw({ wasmBytes: GARBAGE_BYTES }).catch((e: unknown) => e);
    expect((err as GraphlawIntegrityError).expected).toBe(WASM_BLAKE3);
  });

  it('fails in instantiate, not the digest gate, for tampered bytes with skipDigestCheck', async () => {
    const err = await initGraphlaw({ wasmBytes: TAMPERED_BYTES, skipDigestCheck: true }).catch(
      (e: unknown) => e,
    );
    expect(err).toBeInstanceOf(GraphlawInitError);
    expect(err).not.toBeInstanceOf(GraphlawIntegrityError);
    expect((err as GraphlawInitError).cause).toBeInstanceOf(Error);
  });
});
