// init.ts — loads, integrity-checks, and instantiates the engine once.
import { GraphlawInitError, GraphlawIntegrityError } from './errors.js';
import { WASM_SHA256 } from './generated/artifact-meta.js';
import type { WasmAbi } from './wire.js';

/** Host runtime used to load and instantiate the wasm module. */
export type GraphlawRuntime = 'node' | 'web';

/** Options for {@link initGraphlaw}. */
export interface GraphlawInitOptions {
  /** Override the module bytes (tests / alternative artifacts). */
  wasmBytes?: Uint8Array;
  /** Skip the pinned-digest check (dev only — you are trusting the bytes). */
  skipShaCheck?: boolean;
  /**
   * Host runtime selector. Defaults to auto-detection: `'web'` only when
   * both `window` and `document` are defined, else `'node'`.
   */
  runtime?: GraphlawRuntime;
}

const isWeb =
  typeof window !== 'undefined' && typeof document !== 'undefined';

/** Resolve the byte-loader/instantiator pair for the requested host. */
async function loadRuntime(
  runtime: GraphlawRuntime,
): Promise<typeof import('./runtime.js')> {
  return runtime === 'web' ? import('./runtime.web.js') : import('./runtime.js');
}

let cached: Promise<WasmAbi> | undefined;

/**
 * Loads the pinned graphlaw wasm module (once per process) and returns the
 * raw wire handle. The ergonomic entry point is {@link ./client.js}.
 */
export async function initGraphlaw(options: GraphlawInitOptions = {}): Promise<WasmAbi> {
  if (options.wasmBytes === undefined && cached) return cached;
  const promise = (async () => {
    const rt = await loadRuntime(options.runtime ?? (isWeb ? 'web' : 'node'));
    const bytes = options.wasmBytes ?? (await rt.loadWasmBytes());
    if (!options.skipShaCheck) {
      const actual = await sha256Hex(bytes);
      if (actual !== WASM_SHA256) throw new GraphlawIntegrityError(WASM_SHA256, actual);
    }
    try {
      return await rt.instantiateFromBytes(bytes);
    } catch (e) {
      if (e instanceof GraphlawInitError) throw e;
      throw new GraphlawInitError('graphlaw: init failed', { cause: e });
    }
  })();
  if (options.wasmBytes === undefined) cached = promise;
  return promise;
}

async function sha256Hex(bytes: Uint8Array): Promise<string> {
  const digest = await crypto.subtle.digest('SHA-256', bytes as unknown as BufferSource);
  return [...new Uint8Array(digest)].map((b) => b.toString(16).padStart(2, '0')).join('');
}

export { GraphlawInitError, GraphlawIntegrityError };
