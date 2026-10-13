// Mirrors tests/wasm_abi.rs sa2a_run12_portability_corpus_executes_through_real_wasi
// (50-vector portable court): parse via real WASM, non-empty quads, stable
// canonical id across two calls, and full ADMIT/REFUSE sa2a: annotation
// classification — no simplification.
import { readdirSync, readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { callRaw } from '../src/raw.js';

const repoRoot = new URL('../../../', import.meta.url).pathname;
const corpusDir = `${repoRoot}conformance/sa2a/portable/run12`;

const files = readdirSync(corpusDir)
  .filter((f) => f.endsWith('.ttl'))
  .sort();

describe('run12 portable corpus', () => {
  it('stays a 50-vector portable court', () => {
    expect(files.length).toBe(50);
  });

  it.each(files.map((f) => [f, f] as const))('%s', async (name) => {
    const text = readFileSync(`${corpusDir}/${name}`, 'utf8');

    const parsed = await callRaw({ op: 'parse', text, dialect: 'turtle' });
    expect(parsed.ok, JSON.stringify(parsed)).toBe(true);
    if (parsed.ok) {
      expect(
        Number(parsed.quads ?? parsed.nquads ?? 0),
        `${name} parsed empty`,
      ).toBeGreaterThan(0);
    }

    const first = await callRaw({
      op: 'canonical',
      data: { text, dialect: 'turtle' },
    });
    const second = await callRaw({
      op: 'canonical',
      data: { text, dialect: 'turtle' },
    });
    expect(first.ok, JSON.stringify(first)).toBe(true);
    expect(second.ok, JSON.stringify(second)).toBe(true);
    if (first.ok && second.ok) {
      expect(first.id, `${name} canonical identity drifted`).toBe(second.id);
    }

    // ADMIT/REFUSE classification via self-describing sa2a: annotations,
    // exactly as the Rust reference test asserts.
    const admits =
      text.includes('sa2a:expected "ADMIT"') ||
      text.includes('sa2a:expectedDecision "ADMIT"');
    if (admits) {
      expect(text.includes('sa2a:authority "NONE"'), `${name} authority widened`).toBe(true);
      expect(
        text.includes('sa2a:consequence "EVIDENCE_ONLY"'),
        `${name} consequence widened`,
      ).toBe(true);
      expect(
        text.includes('sa2a:canonicalization "RDFC-1.0"'),
        `${name} canonicalization widened`,
      ).toBe(true);
    } else {
      expect(
        text.includes('sa2a:expected "REFUSE"') ||
          text.includes('sa2a:expectedDecision "REFUSE"'),
        `${name} missing disposition`,
      ).toBe(true);
      expect(text.includes('sa2a:violation'), `${name} refusal lacks typed violation`).toBe(true);
    }
  });
});
