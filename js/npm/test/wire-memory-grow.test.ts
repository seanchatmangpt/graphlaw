// End-to-end wire test that forces WebAssembly linear-memory growth through
// the real graphlaw module: a ~200k-triple Turtle document far exceeds the
// initial memory, so gl_call must grow memory and WasmAbi must re-create
// its Uint8Array views afterwards (wire.ts view() contract). No synthetic
// module — real collaborator per testing discipline.
import { describe, expect, it } from 'vitest';
import { callRaw } from '../src/raw.js';

const N = 200_000;

function bigTurtle(n: number): string {
  const lines: string[] = new Array(n);
  for (let i = 0; i < n; i++) {
    lines[i] = `<http://ex/s${i}> <http://ex/p> "v${i}" .`;
  }
  return lines.join('\n');
}

describe('wire memory growth', () => {
  const text = bigTurtle(N);

  it('parses a document large enough to force memory growth', async () => {
    const envelope = await callRaw({ op: 'parse', text, dialect: 'turtle' });
    expect(envelope.ok, JSON.stringify(envelope).slice(0, 400)).toBe(true);
    if (envelope.ok) {
      expect(envelope.quads).toBe(N);
    }
  });

  it('returns an identical result on a second call (views re-created post-grow)', async () => {
    const first = await callRaw({ op: 'parse', text, dialect: 'turtle' });
    expect(first.ok, JSON.stringify(first).slice(0, 400)).toBe(true);
    const second = await callRaw({ op: 'parse', text, dialect: 'turtle' });
    expect(second.ok, JSON.stringify(second).slice(0, 400)).toBe(true);
    // The parse op's declared output is `quads` (registry.rs); identical
    // results across calls prove the post-grow view handling is correct.
    if (first.ok && second.ok) {
      expect(second.quads).toBe(first.quads);
      expect(second.quads).toBe(N);
    }
  });

  it('wire still serves capabilities after the large call', async () => {
    const envelope = await callRaw({ op: 'capabilities' });
    expect(envelope.ok, JSON.stringify(envelope)).toBe(true);
  });
});
