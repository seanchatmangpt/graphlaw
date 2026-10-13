// wire.ts — the JS port of the reference client in tests/wasm_abi.rs:
// gl_alloc/gl_call/gl_free over linear memory with the packed-u64 return
// convention (out_ptr << 32 | out_len). Re-creates memory views after every
// call because gl_call may grow memory.
import {
  GraphlawAllocRefusedError,
  GraphlawRequestTooLargeError,
} from './errors.js';
import { LIMITS } from './limits.js';

/** Minimal shape of an instantiated module the wire needs. */
export interface GlExports {
  memory: WebAssembly.Memory;
  gl_alloc: (len: number) => number;
  gl_call: (ptr: number, len: number) => bigint | number;
  gl_free: (ptr: number, len: number) => void;
}

/** True when the module exports the reactor initializer. */
export function hasInitialize(exports_: Record<string, unknown>): boolean {
  return typeof exports_['_initialize'] === 'function';
}

/** Calls `_initialize` when the module is a reactor. */
export function callInitialize(exports_: Record<string, unknown>): void {
  const init = exports_['_initialize'] as (() => void) | undefined;
  if (typeof init === 'function') init();
}

const enc = new TextEncoder();
const dec = new TextDecoder();

export class WasmAbi {
  private readonly exports: GlExports;

  constructor(exports: GlExports) {
    this.exports = exports;
  }

  /** Serializes, sends, and parses one ABI call. Never throws for ABI
   *  outcomes ({ok:false} passes through); throws typed transport errors. */
  async call(request: unknown): Promise<unknown> {
    const bytes = enc.encode(JSON.stringify(request));
    if (bytes.length > LIMITS.max_request_bytes) {
      throw new GraphlawRequestTooLargeError(bytes.length, LIMITS.max_request_bytes);
    }
    const ptr = this.exports.gl_alloc(bytes.length);
    if (ptr === 0) {
      throw new GraphlawAllocRefusedError(
        `gl_alloc refused ${bytes.length} bytes (over request/limit or outstanding cap)`,
      );
    }
    this.view().set(bytes, ptr);
    const packed = this.exports.gl_call(ptr, bytes.length);
    const packedBig = typeof packed === 'bigint' ? packed : BigInt(packed);
    const outPtr = Number(packedBig >> 32n);
    const outLen = Number(packedBig & 0xffffffffn);
    try {
      const out = dec.decode(this.view().subarray(outPtr, outPtr + outLen));
      return JSON.parse(out);
    } finally {
      this.exports.gl_free(outPtr, outLen);
    }
  }

  /** String-in/string-out variant of {@link call}. */
  async callJson(requestJson: string): Promise<string> {
    const response = (await this.call(JSON.parse(requestJson))) as unknown;
    return JSON.stringify(response);
  }

  /** Fresh view — must not be cached across calls. */
  private view(): Uint8Array {
    return new Uint8Array(this.exports.memory.buffer);
  }
}
