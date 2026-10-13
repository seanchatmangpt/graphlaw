// index.ts — main export: ergonomic typed API.
export { Graphlaw, createClient } from './client.js';
export type { RawRequest } from './client.js';
export { initGraphlaw } from './init.js';
export type { GraphlawInitOptions } from './init.js';
export {
  GraphlawError,
  GraphlawInitError,
  GraphlawIntegrityError,
  GraphlawAllocRefusedError,
  GraphlawRequestTooLargeError,
  GraphlawRefusalError,
  refusalToError,
} from './errors.js';
export { ABI_VERSION, LIMITS } from './limits.js';
export type {
  AbiOk,
  AbiRefused,
  AbiResult,
  DataSpec,
  Term,
} from './types.js';
export type * from './generated/types.js';
