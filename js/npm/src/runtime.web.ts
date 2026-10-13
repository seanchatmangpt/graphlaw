// runtime.web.ts — browser/bundler byte loading and WASI preview1 import
// assembly. Zero-dependency: the module imports only 7 trivial preview1
// functions, all stubbed here, so the same assembly works on browsers and
// bundlers without any node: imports.
import { GraphlawInitError } from './errors.js';
import { callInitialize, hasInitialize, WasmAbi } from './wire.js';

const WASM_URL = new URL('../wasm/graphlaw.wasm', import.meta.url);

/**
 * Loads the module bytes via fetch on the package-relative URL.
 *
 * Bundler targets may rewrite the URL at build time; the web target serves
 * the file directly from the package. Returns the bytes so callers can also
 * cache or inspect them before instantiation.
 */
export async function loadWasmBytes(): Promise<Uint8Array> {
  let response: Response;
  try {
    response = await fetch(WASM_URL);
  } catch (e) {
    throw new GraphlawInitError('graphlaw: failed to fetch the wasm module', { cause: e });
  }
  if (!response.ok) {
    throw new GraphlawInitError(
      `graphlaw: failed to fetch the wasm module (HTTP ${response.status})`,
    );
  }
  try {
    return new Uint8Array(await response.arrayBuffer());
  } catch (e) {
    throw new GraphlawInitError('graphlaw: failed to read the wasm module bytes', { cause: e });
  }
}

const ERRNO_SUCCESS = 0;

/**
 * Instantiates the wasip1 module and returns the wire.
 *
 * The artifact is a cdylib: no `_start`/`_initialize` requirement at the host
 * level — the module needs only 7 trivial imports; serve all of them ourselves
 * so the assembly is host-independent. Uses WebAssembly.instantiateStreaming
 * when handed a Response, else compile+instantiate on raw bytes.
 */
export async function instantiateFromBytes(
  bytes: Uint8Array | Response,
): Promise<WasmAbi> {
  // Set once instantiation completes; the stubs are only ever invoked from
  // module code running after that point.
  let memory: WebAssembly.Memory | null = null;
  const mem = (): WebAssembly.Memory => {
    if (!memory) throw new GraphlawInitError('graphlaw: import called before instantiation');
    return memory;
  };

  const wasiSnapshotPreview1: Record<string, unknown> = {
    // Empty environment: sizes_get writes (0, 0); environ_get is a no-op.
    environ_sizes_get: (envCountPtr: number, envBufSizePtr: number): number => {
      const view = new DataView(mem().buffer);
      view.setUint32(envCountPtr, 0, true);
      view.setUint32(envBufSizePtr, 0, true);
      return ERRNO_SUCCESS;
    },
    environ_get: (): number => ERRNO_SUCCESS,
    sched_yield: (): number => ERRNO_SUCCESS,
    // entropy_seed path — real randomness, not zeros.
    random_get: (bufPtr: number, bufLen: number): number => {
      globalThis.crypto.getRandomValues(new Uint8Array(mem().buffer, bufPtr, bufLen));
      return ERRNO_SUCCESS;
    },
    // stdout/stderr from panic paths: accept and discard.
    fd_write: (fd: number, iovecsPtr: number, iovecsLen: number, nwrittenPtr: number): number => {
      void fd;
      const view = new DataView(mem().buffer);
      let written = 0;
      for (let i = 0; i < iovecsLen; i++) {
        written += view.getUint32(iovecsPtr + i * 8 + 4, true);
      }
      view.setUint32(nwrittenPtr, written, true);
      return ERRNO_SUCCESS;
    },
    // Monotonic nanoseconds; performance.now() is monotonic per spec.
    clock_time_get: (clockId: number, precision: bigint, timePtr: number): number => {
      void clockId;
      void precision;
      const view = new DataView(mem().buffer);
      const nanos = BigInt(Math.round(performance.now() * 1e6));
      view.setBigUint64(timePtr, nanos, true);
      return ERRNO_SUCCESS;
    },
    proc_exit: (code: number): never => {
      throw new GraphlawInitError(`graphlaw: module called proc_exit(${code})`);
    },
  };

  const imports = { wasi_snapshot_preview1: wasiSnapshotPreview1 };

  let instance: WebAssembly.Instance;
  try {
    if (bytes instanceof Response) {
      // Streaming path: compile while the response body downloads.
      const result = await WebAssembly.instantiateStreaming(bytes, imports as WebAssembly.Imports);
      instance = result.instance;
    } else {
      const module_ = await WebAssembly.compile(bytes as unknown as BufferSource);
      instance = await WebAssembly.instantiate(module_, imports as WebAssembly.Imports);
    }
  } catch (e) {
    throw new GraphlawInitError('graphlaw: failed to instantiate the wasm module', { cause: e });
  }
  memory = (instance.exports as { memory: WebAssembly.Memory }).memory;
  const exports_ = instance.exports as unknown as Record<string, unknown>;
  if (hasInitialize(exports_)) {
    // Reactor modules must be initialized exactly once before use.
    const init = exports_['_initialize'] as () => void;
    init();
  } else {
    // cdylib artifacts without _initialize need no reactor init.
    callInitialize(exports_);
  }
  return new WasmAbi(exports_ as never);
}
