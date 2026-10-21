// blake3.ts — self-contained pure-TypeScript BLAKE3 (hash mode, arbitrary
// length input, incremental update). Port of the official public-domain
// reference implementation (BLAKE3-team/BLAKE3, src/lib/reference_impl.rs,
// CC0). No dependencies, node/web safe.
//
// Validated against the `b3sum` CLI (external authority) on empty, short,
// block-boundary, multi-chunk and multi-megabyte inputs; see
// test/blake3.test.ts.

// SHA-256 IV (BLAKE3 reuses it as its initialization vector).
const IV = new Uint32Array([
  0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
  0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
]);

// Message-word permutation applied between rounds (reference table).
const MSG_PERMUTATION = [2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8];

const CHUNK_START = 1 << 0;
const CHUNK_END = 1 << 1;
const PARENT = 1 << 2;
const ROOT = 1 << 3;

const BLOCK_LEN = 64;
const CHUNK_LEN = 1024;
const OUT_LEN = 32;

function rotr(x: number, n: number): number {
  return ((x >>> n) | (x << (32 - n))) >>> 0;
}

type State = Uint32Array; // 16 words

function g(state: State, a: number, b: number, c: number, d: number, mx: number, my: number): void {
  let x = state[a]!;
  let y = state[b]!;
  const w = state[c]!;
  const z = state[d]!;
  x = (x + y + mx) >>> 0;
  state[a] = x;
  state[d] = rotr(z ^ x, 16);
  state[c] = (w + state[d]!) >>> 0;
  state[b] = rotr(y ^ state[c]!, 12);
  y = state[b]!;
  x = (x + y + my) >>> 0;
  state[a] = x;
  state[d] = rotr(state[d]! ^ x, 8);
  state[c] = (state[c]! + state[d]!) >>> 0;
  state[b] = rotr(state[b]! ^ state[c]!, 7);
}

function round(state: State, m: Uint32Array): void {
  // Column step, then diagonal step.
  g(state, 0, 4, 8, 12, m[0]!, m[1]!);
  g(state, 1, 5, 9, 13, m[2]!, m[3]!);
  g(state, 2, 6, 10, 14, m[4]!, m[5]!);
  g(state, 3, 7, 11, 15, m[6]!, m[7]!);
  g(state, 0, 5, 10, 15, m[8]!, m[9]!);
  g(state, 1, 6, 11, 12, m[10]!, m[11]!);
  g(state, 2, 7, 8, 13, m[12]!, m[13]!);
  g(state, 3, 4, 9, 14, m[14]!, m[15]!);
}

function permute(m: Uint32Array): void {
  const old = m.slice();
  for (let i = 0; i < 16; i++) m[i] = old[MSG_PERMUTATION[i]!]!;
}

// The compression function. Inputs other than the 16 state words are mixed
// in before the rounds: counter into state[12..13], block length into
// state[14], and flags into state[15], exactly as in the reference.
function compress(
  cv: Uint32Array, // 8 chaining-value words
  blockWords: Uint32Array, // 16 message words
  counter: number,
  blockLen: number,
  flags: number,
): State {
  const state: State = new Uint32Array(16);
  state.set(cv.subarray(0, 8), 0);
  state.set(IV, 8);
  state[12] = counter >>> 0; // low 32 bits of the 64-bit counter
  state[13] = 0; // high 32 bits (chunk counts far below 2^32 here)
  state[14] = blockLen >>> 0;
  state[15] = flags >>> 0;
  const m = blockWords.slice();
  for (let r = 0; r < 7; r++) {
    round(state, m);
    if (r < 6) permute(m);
  }
  for (let i = 0; i < 8; i++) {
    state[i] = (state[i]! ^ state[i + 8]!) >>> 0;
    state[i + 8] = (state[i + 8]! ^ cv[i]!) >>> 0;
  }
  return state;
}

class Output {
  constructor(
    readonly inputChainingValue: Uint32Array,
    readonly blockWords: Uint32Array,
    readonly counter: number,
    readonly blockLen: number,
    readonly flags: number,
  ) {}

  chainingValue(): Uint32Array {
    return compress(
      this.inputChainingValue,
      this.blockWords,
      this.counter,
      this.blockLen,
      this.flags,
    ).subarray(0, 8).slice();
  }

  rootOutput(out: Uint32Array): void {
    const state = compress(
      this.inputChainingValue,
      this.blockWords,
      this.counter,
      this.blockLen,
      this.flags | ROOT,
    );
    out.set(state.subarray(0, 8));
  }
}

class ChunkState {
  cv = new Uint32Array(IV);
  chunkCounter: number;
  block = new Uint8Array(BLOCK_LEN);
  blockLen = 0;
  blocksCompressed = 0;

  constructor(chunkCounter: number) {
    this.chunkCounter = chunkCounter;
  }

  len(): number {
    return BLOCK_LEN * this.blocksCompressed + this.blockLen;
  }

  startFlag(): number {
    return this.blocksCompressed === 0 ? CHUNK_START : 0;
  }

  update(input: Uint8Array): void {
    while (input.length > 0) {
      // If the block is full, compress it and start a new one.
      if (this.blockLen === BLOCK_LEN) {
        const blockWords = wordsFromLEBytes(this.block);
        this.cv = compress(
          this.cv,
          blockWords,
          this.chunkCounter,
          BLOCK_LEN,
          this.startFlag(),
        ).subarray(0, 8).slice();
        this.blocksCompressed += 1;
        this.block.fill(0);
        this.blockLen = 0;
      }
      const want = BLOCK_LEN - this.blockLen;
      const take = Math.min(want, input.length);
      this.block.set(input.subarray(0, take), this.blockLen);
      this.blockLen += take;
      input = input.subarray(take);
    }
  }

  output(): Output {
    const flags = this.startFlag() | CHUNK_END;
    return new Output(this.cv, wordsFromLEBytes(this.block), this.chunkCounter, this.blockLen, flags);
  }
}

function parentOutput(
  leftChildCv: Uint32Array,
  rightChildCv: Uint32Array,
  key: Uint32Array,
): Output {
  const blockWords = new Uint32Array(16);
  blockWords.set(leftChildCv.subarray(0, 8), 0);
  blockWords.set(rightChildCv.subarray(0, 8), 8);
  return new Output(key, blockWords, 0, BLOCK_LEN, PARENT);
}

function parentCv(leftChildCv: Uint32Array, rightChildCv: Uint32Array, key: Uint32Array): Uint32Array {
  return parentOutput(leftChildCv, rightChildCv, key).chainingValue();
}

class Hasher {
  chunkState = new ChunkState(0);
  key = new Uint32Array(IV);
  chunkStack: Uint32Array[] = []; // completed subtree chaining values

  update(input: Uint8Array): this {
    while (input.length > 0) {
      // Finish the current chunk if it is already full.
      if (this.chunkState.len() === CHUNK_LEN) {
        const chunkCv = this.chunkState.output().chainingValue();
        const totalChunks = this.chunkState.chunkCounter + 1;
        this.addChunkCv(chunkCv, totalChunks);
        this.chunkState = new ChunkState(totalChunks);
      }
      const want = CHUNK_LEN - this.chunkState.len();
      const take = Math.min(want, input.length);
      this.chunkState.update(input.subarray(0, take));
      input = input.subarray(take);
    }
    return this;
  }

  private addChunkCv(newCv: Uint32Array, totalChunks: number): void {
    // Merge full subtrees: while the running chunk count is even, fold the
    // top of the stack into a parent (reference add_chunk_chaining_value).
    let chunks = totalChunks;
    while (chunks % 2 === 0 && this.chunkStack.length > 0) {
      const left = this.chunkStack.pop()!;
      newCv = parentCv(left, newCv, this.key);
      chunks = Math.floor(chunks / 2);
    }
    this.chunkStack.push(newCv);
  }

  finalize(): Uint8Array {
    let output = this.chunkState.output();
    for (let i = this.chunkStack.length - 1; i >= 0; i--) {
      output = parentOutput(this.chunkStack[i]!, output.chainingValue(), this.key);
    }
    const words = new Uint32Array(8);
    output.rootOutput(words);
    const out = new Uint8Array(OUT_LEN);
    for (let i = 0; i < 8; i++) {
      const w = words[i]!;
      out[i * 4] = w & 0xff;
      out[i * 4 + 1] = (w >>> 8) & 0xff;
      out[i * 4 + 2] = (w >>> 16) & 0xff;
      out[i * 4 + 3] = (w >>> 24) & 0xff;
    }
    return out;
  }
}

function wordsFromLEBytes(bytes: Uint8Array): Uint32Array {
  const words = new Uint32Array(16);
  for (let i = 0; i < 16; i++) {
    const o = i * 4;
    words[i] =
      ((bytes[o] ?? 0) |
        ((bytes[o + 1] ?? 0) << 8) |
        ((bytes[o + 2] ?? 0) << 16) |
        ((bytes[o + 3] ?? 0) << 24)) >>>
      0;
  }
  return words;
}

/** Incremental BLAKE3 hasher (hash mode). */
export class Blake3 {
  private hasher = new Hasher();

  update(data: Uint8Array): this {
    this.hasher.update(data);
    return this;
  }

  /** Returns the 32-byte digest. */
  digest(): Uint8Array {
    return this.hasher.finalize();
  }

  digestHex(): string {
    return toHex(this.digest());
  }
}

/** One-shot BLAKE3 hash of `data`; returns the 32-byte digest. */
export function blake3(data: Uint8Array): Uint8Array {
  return new Blake3().update(data).digest();
}

/** One-shot BLAKE3 hash of `data` as lowercase hex. */
export function blake3Hex(data: Uint8Array): string {
  return toHex(blake3(data));
}

export function toHex(bytes: Uint8Array): string {
  let s = '';
  for (let i = 0; i < bytes.length; i++) s += bytes[i]!.toString(16).padStart(2, '0');
  return s;
}
