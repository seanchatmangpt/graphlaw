// Typed error classes. Convention: every class name starts with `Graphlaw`
// and ends with `Error` — NEVER a class literally named `Error` (wasm-bindgen
// JS-global clash; also our lint rule forbids it).
import type { AbiRefused, RefusalDetails } from './types.js';

/** Base class for all errors this package throws. */
export class GraphlawError extends Error {
  readonly code: string;
  constructor(code: string, message: string) {
    super(message);
    this.name = new.target.name;
    this.code = code;
    // Required so instanceof works when targeting ES2022 output.
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

/** Module load, instantiation, or WASI import failure. */
export class GraphlawInitError extends GraphlawError {
  constructor(message: string, options?: { cause?: unknown }) {
    super('init', message);
    if (options?.cause !== undefined) this.cause = options.cause;
  }
}

/** The wasm artifact's SHA-256 did not match the pinned digest. */
export class GraphlawIntegrityError extends GraphlawError {
  readonly expected: string;
  readonly actual: string;
  constructor(expected: string, actual: string) {
    super('integrity', `wasm artifact digest mismatch: expected ${expected}, got ${actual}`);
    this.expected = expected;
    this.actual = actual;
  }
}

/** gl_alloc refused (request over limits or outstanding-bytes cap). */
export class GraphlawAllocRefusedError extends GraphlawError {
  constructor(message: string) {
    super('alloc_refused', message);
  }
}

/** Serialized request exceeds MAX_REQUEST_BYTES (16 MiB). */
export class GraphlawRequestTooLargeError extends GraphlawError {
  readonly observed: number;
  readonly max: number;
  constructor(observed: number, max: number) {
    super(
      'request_too_large',
      `request is ${observed} bytes; the ABI limit is ${max} (ResourceLimit/request_bytes)`,
    );
    this.observed = observed;
    this.max = max;
  }
}

/** An ABI refusal ({ok:false}) surfaced as a typed exception. */
export class GraphlawRefusalError extends GraphlawError {
  readonly kind: string;
  readonly engine: string | null;
  readonly dialect: string | null;
  readonly details?: RefusalDetails;
  constructor(refusal: AbiRefused['error']) {
    const code = refusal.details?.code;
    super(code ?? 'refused', refusal.message);
    this.kind = refusal.kind;
    this.engine = refusal.engine;
    this.dialect = refusal.dialect;
    if (refusal.details !== undefined) this.details = refusal.details;
  }
}

/** Maps a refusal envelope to the typed error. */
export function refusalToError(refusal: AbiRefused['error']): GraphlawRefusalError {
  return new GraphlawRefusalError(refusal);
}
