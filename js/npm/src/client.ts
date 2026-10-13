// client.ts — the ergonomic typed API: one method per ABI op.
// Refusals ({ok:false}) are thrown as GraphlawRefusalError; `tryX()` variants
// return Result objects instead.
import { refusalToError } from './errors.js';
import type {
  AbiOk,
  AbiResult,
  DataSpec,
} from './types.js';
import type {
  CanonicalResponse,
  CapabilitiesResponse,
  ConvertResponse,
  DatalogRequest,
  DatalogResponse,
  EntailRequest,
  EntailResponse,
  HooksRequest,
  HooksResponse,
  LawRequest,
  LawResponse,
  N3Request,
  N3Response,
  ParseResponse,
  PolicyRequest,
  PolicyResponse,
  ShaclRequest,
  ShaclResponse,
  ShexRequest,
  ShexResponse,
  SniffResponse,
  SparqlRequest,
  SparqlResponse,
} from './generated/types.js';
import type { WasmAbi } from './wire.js';
import { initGraphlaw } from './init.js';

/** Raw JSON request accepted by `graphlaw/raw`. */
export type RawRequest = Record<string, unknown>;

/** The ergonomic engine client. Construct with {@link createClient}. */
export class Graphlaw {
  private readonly abi: Promise<WasmAbi>;

  /** @internal */
  constructor(abi: Promise<WasmAbi>) {
    this.abi = abi;
  }

  private async dispatch<T>(request: RawRequest): Promise<AbiOk<T>> {
    const wire = await this.abi;
    const envelope = (await wire.call(request)) as AbiOk<T> | { ok: false; error: never };
    if (!envelope.ok) throw refusalToError(envelope.error);
    return envelope;
  }

  private async tryDispatch<T>(request: RawRequest): Promise<AbiResult<T>> {
    const wire = await this.abi;
    return (await wire.call(request)) as AbiResult<T>;
  }

  capabilities(): Promise<AbiOk<CapabilitiesResponse>> {
    return this.dispatch({ op: 'capabilities' });
  }

  tryCapabilities(): Promise<AbiResult<CapabilitiesResponse>> {
    return this.tryDispatch({ op: 'capabilities' });
  }

  sniff(text: string, hint?: string | null): Promise<AbiOk<SniffResponse>> {
    return this.dispatch({ op: 'sniff', text, ...(hint !== undefined ? { hint } : {}) });
  }

  trySniff(text: string, hint?: string | null): Promise<AbiResult<SniffResponse>> {
    return this.tryDispatch({ op: 'sniff', text, ...(hint !== undefined ? { hint } : {}) });
  }

  parse(request: { text: string; dialect?: string | null; hint?: string | null; base?: string | null }): Promise<AbiOk<ParseResponse>> {
    return this.dispatch({ op: 'parse', ...request });
  }

  tryParse(request: { text: string; dialect?: string | null; hint?: string | null; base?: string | null }): Promise<AbiResult<ParseResponse>> {
    return this.tryDispatch({ op: 'parse', ...request });
  }

  convert(request: { text: string; to: string; dialect?: string | null; hint?: string | null; base?: string | null }): Promise<AbiOk<ConvertResponse>> {
    return this.dispatch({ op: 'convert', ...request });
  }

  tryConvert(request: { text: string; to: string; dialect?: string | null; hint?: string | null; base?: string | null }): Promise<AbiResult<ConvertResponse>> {
    return this.tryDispatch({ op: 'convert', ...request });
  }

  canonical(data: DataSpec): Promise<AbiOk<CanonicalResponse>> {
    return this.dispatch({ op: 'canonical', data });
  }

  tryCanonical(data: DataSpec): Promise<AbiResult<CanonicalResponse>> {
    return this.tryDispatch({ op: 'canonical', data });
  }

  sparql(request: SparqlRequest): Promise<AbiOk<SparqlResponse>> {
    return this.dispatch({ op: 'sparql', ...request });
  }

  trySparql(request: SparqlRequest): Promise<AbiResult<SparqlResponse>> {
    return this.tryDispatch({ op: 'sparql', ...request });
  }

  shacl(request: ShaclRequest): Promise<AbiOk<ShaclResponse>> {
    return this.dispatch({ op: 'shacl', ...request });
  }

  tryShacl(request: ShaclRequest): Promise<AbiResult<ShaclResponse>> {
    return this.tryDispatch({ op: 'shacl', ...request });
  }

  shex(request: ShexRequest): Promise<AbiOk<ShexResponse>> {
    return this.dispatch({ op: 'shex', ...request });
  }

  tryShex(request: ShexRequest): Promise<AbiResult<ShexResponse>> {
    return this.tryDispatch({ op: 'shex', ...request });
  }

  n3(request: N3Request): Promise<AbiOk<N3Response>> {
    return this.dispatch({ op: 'n3', ...request });
  }

  tryN3(request: N3Request): Promise<AbiResult<N3Response>> {
    return this.tryDispatch({ op: 'n3', ...request });
  }

  entail(request: EntailRequest): Promise<AbiOk<EntailResponse>> {
    return this.dispatch({ op: 'entail', ...request });
  }

  tryEntail(request: EntailRequest): Promise<AbiResult<EntailResponse>> {
    return this.tryDispatch({ op: 'entail', ...request });
  }

  datalog(request: DatalogRequest): Promise<AbiOk<DatalogResponse>> {
    return this.dispatch({ op: 'datalog', ...request });
  }

  tryDatalog(request: DatalogRequest): Promise<AbiResult<DatalogResponse>> {
    return this.tryDispatch({ op: 'datalog', ...request });
  }

  hooks(request: HooksRequest): Promise<AbiOk<HooksResponse>> {
    return this.dispatch({ op: 'hooks', ...request });
  }

  tryHooks(request: HooksRequest): Promise<AbiResult<HooksResponse>> {
    return this.tryDispatch({ op: 'hooks', ...request });
  }

  law(request: LawRequest): Promise<AbiOk<LawResponse>> {
    return this.dispatch({ op: 'law', ...request });
  }

  tryLaw(request: LawRequest): Promise<AbiResult<LawResponse>> {
    return this.tryDispatch({ op: 'law', ...request });
  }

  policy(request: PolicyRequest): Promise<AbiOk<PolicyResponse>> {
    return this.dispatch({ op: 'policy', ...request });
  }

  tryPolicy(request: PolicyRequest): Promise<AbiResult<PolicyResponse>> {
    return this.tryDispatch({ op: 'policy', ...request });
  }
}

/** Initializes the engine once and returns a typed client. */
export async function createClient(): Promise<Graphlaw> {
  const abi = initGraphlaw();
  // Fail fast on init errors instead of on first call.
  await abi;
  return new Graphlaw(abi);
}
