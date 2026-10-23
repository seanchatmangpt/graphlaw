// runtime.ts — host detection, byte loading, and WASI preview1 import
// assembly. Zero-dependency: all 28 preview1 imports of the DuckDB-linked
// artifact are served by the shared stub builder in wasi-stubs.ts, so the
// same assembly works on Node and browsers (phase 2 swaps loadWasmBytes for
// fetch/URL loading).
import { GraphlawInitError } from './errors.js';
import { callInitialize, hasInitialize, WasmAbi } from './wire.js';
import { buildWasiImports } from './wasi-stubs.js';

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

/**
 * Instantiates the wasip1 module and returns the wire.
 *
 * The artifact is a cdylib: no `_start`/`_initialize`, so `wasi.start()` /
 * `wasi.initialize()` can never run — and Node refuses to serve some preview1
 * imports until one of them has. All 28 preview1 imports (including the
 * DuckDB-linked artifact's fd/path surface) are served by wasi-stubs.ts so
 * the assembly is host-independent.
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

  const wasiSnapshotPreview1 = buildWasiImports(
    mem,
    (buf) => {
      randomFillSync(buf);
    },
    (code: number): never => {
      throw new GraphlawInitError(`graphlaw: module called proc_exit(${code})`);
    },
  );

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
