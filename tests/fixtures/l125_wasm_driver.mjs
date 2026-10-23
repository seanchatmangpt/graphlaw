// Lane L125 wasm driver: drives the compiled graphlaw-wasm module through the
// real gl_alloc/gl_call/gl_free ABI in a runtime with wasm-exceptions support
// (wasmi 2.0.0 rejects the frozen-duckdb-linked module — "exceptions proposal
// not enabled"; Node's V8 supports them natively).
//
// Protocol: one JSON request per stdin line -> one JSON response per stdout
// line (line-delimited JSON is lossless for this ABI: responses are always a
// single JSON document, never embed raw newlines).
import { WASI } from 'node:wasi';
import { readFileSync } from 'node:fs';
import { createInterface } from 'node:readline';
import assert from 'node:assert';

const wasmPath = process.argv[2];
assert.ok(wasmPath, 'usage: node l125_wasm_driver.mjs <module.wasm>');

const bytes = readFileSync(wasmPath);
const wasi = new WASI({ version: 'preview1' });
const { instance } = await WebAssembly.instantiate(bytes, {
  wasi_snapshot_preview1: wasi.wasiImport,
});
const { memory, gl_alloc, gl_free, gl_call } = instance.exports;
// Reactor-style modules must be initialized once before use.
if (instance.exports._initialize) instance.exports._initialize();

function request(text) {
  const body = Buffer.from(text, 'utf8');
  const ptr = gl_alloc(body.length);
  assert.notStrictEqual(ptr, 0, 'gl_alloc refused');
  new Uint8Array(memory.buffer, ptr, body.length).set(body);
  const packed = gl_call(ptr, body.length);
  const outPtr = Number(packed >> 32n) >>> 0;
  const outLen = Number(packed & 0xffffffffn) >>> 0;
  const out = Buffer.from(new Uint8Array(memory.buffer, outPtr, outLen));
  gl_free(outPtr, outLen);
  return out.toString('utf8');
}

process.stderr.write(`l125 driver: module ${wasmPath} instantiated\n`);
const rl = createInterface({ input: process.stdin, terminal: false });
for await (const line of rl) {
  if (line.trim() === '') continue;
  process.stdout.write(request(line) + '\n');
}
