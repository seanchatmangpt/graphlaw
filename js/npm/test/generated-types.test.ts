// Generated-types conformance: src/generated/types.ts must reflect
// registry/capability-registry.json exactly (Chicago-style: assert on the real
// generated artifact and the real module, no mocks).
import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import type {
  Lease,
  SignedLease,
  Receipt,
  RefusalKind,
} from '../src/generated/types.js';

const here = dirname(fileURLToPath(import.meta.url));
const pkgRoot = resolve(here, '..');
const repoRoot = resolve(pkgRoot, '..', '..');
const registryPath = join(repoRoot, 'registry', 'capability-registry.json');
const registry = JSON.parse(readFileSync(registryPath, 'utf8')) as {
  registry_sha256: string;
  ops: { name: string }[];
  models: { name: string }[];
  refusal_kinds: string[];
  refusal_codes: { code: string }[];
  engines: string[];
  model_enums: { name: string; values: string[] }[];
};
const generated = readFileSync(
  join(pkgRoot, 'src', 'generated', 'types.ts'),
  'utf8',
);

describe('generated types vs capability registry', () => {
  it('header carries the current registry_sha256', () => {
    const headerLine = `// registry_sha256: ${registry.registry_sha256}`;
    expect(generated).toContain(headerLine);
    expect(generated.split('\n')[1]).toBe(headerLine);
  });

  it('declares exactly one Request interface per registry op', () => {
    const expected = registry.ops.map(
      (op) => op.name.charAt(0).toUpperCase() + op.name.slice(1) + 'Request',
    );
    const declared = [
      ...generated.matchAll(/^export interface (\w+Request) \{/gm),
    ].map((m) => m[1]);
    expect(declared.sort()).toEqual([...expected].sort());
    expect(expected.length).toBe(14);
  });

  it('declares an interface per registry model', () => {
    for (const model of registry.models) {
      expect(generated).toMatch(
        new RegExp(`^export interface ${model.name} \\{`, 'm'),
      );
    }
  });

  it('embeds registry string values as literals', () => {
    const literals = [
      ...registry.refusal_kinds,
      ...registry.refusal_codes.map((c) => c.code),
      ...registry.engines,
      ...registry.model_enums.flatMap((e) => e.values),
    ];
    for (const value of literals) {
      expect(generated).toContain(JSON.stringify(value));
    }
  });
});

describe('ABI-critical runtime exports', () => {
  it('exposes Lease, SignedLease, Receipt, Ceiling_VALUES and RefusalKind', async () => {
    // Lease/SignedLease/Receipt/RefusalKind are type-space exports (erased at
    // runtime); verified by these compile-time assignments:
    const lease: Lease = {
      id: 'l',
      holder: 'h',
      ceiling: 'construct',
      scope: ['s'],
      expires_unix: 0,
      issued_unix: 0,
    };
    const receipt: Receipt = {
      parent: 'p',
      child: 'c',
      step: 's',
      authority: 'a',
      revision: 'r',
      added: 0,
    };
    const kind: RefusalKind = 'Ambiguous';
    expect(lease.id).toBe('l');
    expect(receipt.step).toBe('s');
    expect(kind).toBe('Ambiguous');

    const mod = (await import('../src/generated/types.js')) as Record<
      string,
      unknown
    >;
    expect(mod).toHaveProperty('Ceiling_VALUES');
    // Runtime consts are real values, not just type names.
    expect(Array.isArray(mod.Ceiling_VALUES)).toBe(true);
    expect([...(mod.Ceiling_VALUES as readonly string[])]).toEqual([
      'observe',
      'select',
      'construct',
    ]);
    expect([...(mod.REFUSAL_KIND_VALUES as readonly string[])]).toEqual(
      registry.refusal_kinds,
    );
    // SignedLease composes the two models it must (compile-time shape check).
    const signed: SignedLease = {
      lease,
      attestation: { payload_sha256: 'x', key_id: 'k', signature: 'sig' },
    };
    expect(signed.lease.holder).toBe('h');
  });
});
