// Tests for typed error classes (src/errors.ts) plus the ecosystem naming
// regression guard: no class may be literally named `Error`.
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { describe, expect, it } from 'vitest';
import {
  GraphlawAllocRefusedError,
  GraphlawError,
  GraphlawInitError,
  GraphlawIntegrityError,
  GraphlawRefusalError,
  GraphlawRequestTooLargeError,
  refusalToError,
} from '../src/errors.js';

const here = dirname(fileURLToPath(import.meta.url));
const srcDir = join(here, '..', 'src');

const subclassCtors: Array<new (...args: any[]) => GraphlawError> = [
  GraphlawInitError,
  GraphlawIntegrityError,
  GraphlawAllocRefusedError,
  GraphlawRequestTooLargeError,
  GraphlawRefusalError,
];

describe('error class hierarchy', () => {
  it('base is an Error', () => {
    const err = new GraphlawError('test_code', 'boom');
    expect(err).toBeInstanceOf(GraphlawError);
    expect(err).toBeInstanceOf(Error);
    expect(err.code).toBe('test_code');
    expect(err.message).toBe('boom');
  });

  it.each(subclassCtors)('%s is instanceof GraphlawError and Error', (Ctor) => {
    const err = new Ctor('a', 'b');
    expect(err).toBeInstanceOf(Ctor);
    expect(err).toBeInstanceOf(GraphlawError);
    expect(err).toBeInstanceOf(Error);
  });

  it.each(subclassCtors)('%s sets name from new.target.name', (Ctor) => {
    const err = new Ctor('a', 'b');
    expect(err.name).toBe(Ctor.name);
  });
});

describe('payload fields', () => {
  it('GraphlawIntegrityError carries expected/actual', () => {
    const err = new GraphlawIntegrityError('cafe1234', 'dead5678');
    expect(err.expected).toBe('cafe1234');
    expect(err.actual).toBe('dead5678');
    expect(err.code).toBe('integrity');
  });

  it('GraphlawRequestTooLargeError carries observed/max', () => {
    const err = new GraphlawRequestTooLargeError(1025, 1024);
    expect(err.observed).toBe(1025);
    expect(err.max).toBe(1024);
    expect(err.code).toBe('request_too_large');
  });

  it('GraphlawRefusalError exposes refusal envelope and details.code', () => {
    const refusal = {
      kind: 'EngineRejected',
      engine: 'PurRdf',
      dialect: 'Turtle',
      message: 'x',
      details: { code: 'NotAdmitted', violations: [] },
    } as const;
    const err = new GraphlawRefusalError(refusal);
    expect(err.kind).toBe('EngineRejected');
    expect(err.engine).toBe('PurRdf');
    expect(err.dialect).toBe('Turtle');
    expect(err.details?.code).toBe('NotAdmitted');
    expect(err.details?.violations).toEqual([]);
    expect(err.code).toBe('NotAdmitted');
    expect(err.message).toBe('x');
  });

  it('refusalToError returns a GraphlawRefusalError preserving the message', () => {
    const refusal = {
      kind: 'EngineRejected',
      engine: 'PurRdf',
      dialect: 'Turtle',
      message: 'refusal body',
      details: { code: 'NotAdmitted', violations: [] },
    } as const;
    const err = refusalToError(refusal);
    expect(err).toBeInstanceOf(GraphlawRefusalError);
    expect(err.message).toBe('refusal body');
  });
});

describe('no class literally named Error (wasm-bindgen guard)', () => {
  const files = ['errors.ts', 'client.ts', 'raw.ts', 'index.ts'];

  it.each(files)('src/%s declares no `class Error`', (file) => {
    const source = readFileSync(join(srcDir, file), 'utf8');
    const lines = source.split('\n');
    const offenders = lines.filter((line) => /export class Error\b/.test(line) || /class Error /.test(line));
    expect(offenders).toEqual([]);
  });
});
