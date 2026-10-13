// Ergonomic-surface conformance: the typed client (src/client.ts) must stay
// aligned with the generated ABI types (src/generated/types.ts). Type-level
// assertions use the Expect<Equal<T, true>> pattern; runtime assertions use
// the real engine (Chicago-style: no mocks).
import { describe, expect, it } from 'vitest';
import { createClient, Graphlaw } from '../src/client.js';
import { GraphlawRefusalError } from '../src/errors.js';
import type {
  AbiOk,
  AbiResult,
  DataSpec,
} from '../src/types.js';
import type {
  LawRequest,
  LawResponse,
  ParseResponse,
  PolicyRequest,
  PolicyResponse,
  SparqlRequest,
  SparqlResponse,
  SparqlSolutions,
} from '../src/generated/types.js';

// --- type-level equality harness -------------------------------------------

type Equal<X, Y> =
  (<T>() => T extends X ? 1 : 2) extends <T>() => T extends Y ? 1 : 2
    ? true
    : false;
type Expect<T extends true> = T;

// --- type-level assertions -------------------------------------------------

describe('client method signatures match the generated ABI types', () => {
  it('parse / sparql / law / policy', async () => {
    const client = await createClient();

    type ParseMethod = typeof client.parse;
    type _parse = Expect<
      Equal<
        ParseMethod,
        (request: {
          text: string;
          dialect?: string | null;
          hint?: string | null;
          base?: string | null;
        }) => Promise<AbiOk<ParseResponse>>
      >
    >;

    type SparqlMethod = typeof client.sparql;
    type _sparql = Expect<
      Equal<
        SparqlMethod,
        (request: SparqlRequest) => Promise<AbiOk<SparqlResponse>>
      >
    >;

    type LawMethod = typeof client.law;
    type _law = Expect<
      Equal<
        LawMethod,
        (request: LawRequest) => Promise<AbiOk<LawResponse>>
      >
    >;

    type PolicyMethod = typeof client.policy;
    type _policy = Expect<
      Equal<
        PolicyMethod,
        (request: PolicyRequest) => Promise<AbiOk<PolicyResponse>>
      >
    >;

    expect(client).toBeInstanceOf(Graphlaw);
  });

  it('tryX variants return Promise<AbiResult<X>>', async () => {
    const client = await createClient();

    type _tryParse = Expect<
      Equal<
        ReturnType<typeof client.tryParse>,
        Promise<AbiResult<ParseResponse>>
      >
    >;
    type _trySparql = Expect<
      Equal<
        ReturnType<typeof client.trySparql>,
        Promise<AbiResult<SparqlResponse>>
      >
    >;
    type _tryLaw = Expect<
      Equal<ReturnType<typeof client.tryLaw>, Promise<AbiResult<LawResponse>>>
    >;
    type _tryPolicy = Expect<
      Equal<
        ReturnType<typeof client.tryPolicy>,
        Promise<AbiResult<PolicyResponse>>
      >
    >;

    expect(true).toBe(true);
  });
});

// --- runtime assertions (real engine) ---------------------------------------

describe('runtime surface aligns with generated types', () => {
  it('parse({text, dialect:"ntriples"}) resolves to {dialect, quads, id} assignable shape', async () => {
    const client = await createClient();
    const res = await client.parse({
      text: '<http://example.com/s> <http://example.com/p> <http://example.com/o> .',
      dialect: 'ntriples',
    });
    expect(res.ok).toBe(true);
    // Response fields assignable to the generated ParseResponse shape.
    const observed: { dialect?: string; quads?: number; id?: string } = res;
    expect(typeof observed.dialect).toBe('string');
    expect(typeof observed.quads).toBe('number');
    expect(typeof observed.id).toBe('string');
    expect(observed.quads).toBe(1);
  });

  it('sparql SELECT response narrows to the solutions arm of the union', async () => {
    const client = await createClient();
    const res = await client.sparql({
      data: {
        text: '<http://example.com/s> <http://example.com/p> <http://example.com/o> .',
        dialect: 'ntriples',
      } satisfies DataSpec,
      query: 'SELECT ?s ?p ?o WHERE { ?s ?p ?o }',
    });
    expect(res.kind).toBe('solutions');
    if (res.kind === 'solutions') {
      // Discriminated-union narrowing: res is SparqlSolutions here.
      const solutions: SparqlSolutions = res;
      const narrowed: { variables?: string[]; rows?: (unknown | null)[][] } =
        solutions;
      expect(Array.isArray(narrowed.variables)).toBe(true);
      expect(Array.isArray(narrowed.rows)).toBe(true);
      expect(narrowed.rows?.length).toBe(1);
    } else {
      throw new Error(`expected solutions, got kind=${res.kind as string}`);
    }
  });

  it('law() accepts a DataSpec plus a steps array', async () => {
    const client = await createClient();
    const res = await client.law({
      data: {
        text: '<http://example.com/s> <http://example.com/p> <http://example.com/o> .',
        dialect: 'ntriples',
      } satisfies DataSpec,
      steps: [{ step: 'rdfs' }],
    });
    expect(res.ok).toBe(true);
    expect(Array.isArray(res.states)).toBe(true);
    expect(Array.isArray(res.receipts)).toBe(true);
  });

  it('n3 on malformed text rejects with GraphlawRefusalError', async () => {
    const client = await createClient();
    await expect(client.n3({ text: 'bad ((' })).rejects.toBeInstanceOf(
      GraphlawRefusalError,
    );
  });
});
