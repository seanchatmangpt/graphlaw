// runtime.ts — host detection, byte loading, and WASI preview1 import
// assembly. Zero-dependency: the module imports only 7 trivial preview1
// functions, all stubbed here, so the same assembly works on Node and
// browsers (phase 2 swaps loadWasmBytes for fetch/URL loading).
import { GraphlawInitError } from './errors.js';
import { callInitialize, hasInitialize, WasmAbi } from './wire.js';

function isNode(): boolean {
  return (
    typeof process !== 'undefined' &&
    typeof (process as { versions?: { node?: string } }).versions?.node === 'string'
  );
}

/** Loads the module bytes from the installed package (Node resolution). */
export async function loadWasmBytes(): Promise<Uint8Array> {
  const { readFile } = await import('node:fs/promises');
  const { fileURLToPath } = await import('node:url');
  const path = fileURLToPath(new URL('../wasm/graphlaw.wasm', import.meta.url));
  return new Uint8Array(await readFile(path));
}

const ERRNO_SUCCESS = 0;

/**
 * Instantiates the wasip1 module and returns the wire.
 *
 * The artifact is a cdylib: no `_start`/`_initialize`, so `wasi.start()` /
 * `wasi.initialize()` can never run — and Node refuses to serve some preview1
 * imports until one of them has. The module needs only 7 trivial imports;
 * serve all of them ourselves so the assembly is host-independent.
 */
export async function instantiateFromBytes(bytes: Uint8Array): Promise<WasmAbi> {
  if (!isNode()) {
    throw new GraphlawInitError(
      'graphlaw: this build targets Node >= 20; browser/bundler support requires the dist-web/dist-bundler entries',
    );
  }
  let randomFillSync: (b: Uint8Array) => Uint8Array;
  try {
    ({ randomFillSync } = (await import('node:crypto')) as typeof import('node:crypto'));
  } catch (e) {
    throw new GraphlawInitError('graphlaw: node:crypto unavailable', { cause: e });
  }

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
      randomFillSync(new Uint8Array(mem().buffer, bufPtr, bufLen));
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
    const module_ = await WebAssembly.compile(bytes as unknown as BufferSource);
    instance = await WebAssembly.instantiate(module_, imports as WebAssembly.Imports);
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
