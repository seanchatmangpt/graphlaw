// Hand-written shared types: RDF term objects, data specs, and the ABI
// envelope. These are structural/recursive shapes the registry describes
// only loosely (type vocabulary `list<term>`, `data_spec`).
import type {
  Engine,
  RefusalCode,
  RefusalKind,
} from './generated/types.js';

/** An RDF term as it appears in SPARQL solution cells and ShEx entries. */
export type Term =
  | { type: 'uri'; value: string }
  | { type: 'bnode'; value: string }
  | { type: 'literal'; value: string; datatype: string; 'xml:lang'?: string }
  | { type: 'triple'; value: string };

/**
 * Data spec: the shared way to hand the engine a document.
 * The dialect is sniffed when omitted.
 */
export interface DataSpec {
  text: string;
  dialect?: string | null;
  hint?: string | null;
  base?: string | null;
}

/** Machine-readable refusal details; shape depends on `code`. */
export interface RefusalDetails {
  code: RefusalCode;
  [field: string]: unknown;
}

/** Failure envelope. */
export interface AbiRefused {
  ok: false;
  error: {
    kind: RefusalKind;
    engine: Engine | null;
    dialect: string | null;
    message: string;
    details?: RefusalDetails;
  };
}

/** Success envelope for op `T`. */
export type AbiOk<T> = { ok: true } & T;

/** Result returned by `tryX()` ergonomic variants. */
export type AbiResult<T> = AbiOk<T> | AbiRefused;

/** Thrown/mapped refusal payload (see errors.ts). */
export type { RefusalKind, RefusalCode, Engine };
