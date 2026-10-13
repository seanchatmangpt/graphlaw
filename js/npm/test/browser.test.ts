// Real-browser suite: GRAPHLAW_BROWSER_TESTS=1 plus an importable
// playwright-core launches headless Chromium against a node:http server
// serving dist-web; the page imports the browser runtime and posts a
// capabilities envelope back to the server, which the test asserts on.
// Without the env var or playwright-core the suite skips gracefully.
import { readFileSync, existsSync } from 'node:fs';
import { join, extname } from 'node:path';
import http from 'node:http';
import { fileURLToPath } from 'node:url';
import { describe, expect, it, afterAll } from 'vitest';

const enabled = process.env.GRAPHLAW_BROWSER_TESTS === '1';

// Dynamic + conditional: playwright-core may not be installed yet (it is a
// devDependency added by a parallel lane); the suite must still load.
let pw: typeof import('playwright-core') | null = null;
if (enabled) {
  try {
    pw = await import('playwright-core');
  } catch {
    pw = null;
  }
}

const npmRoot = fileURLToPath(new URL('..', import.meta.url));
const distWeb = join(npmRoot, 'dist-web');
const distReady = enabled && existsSync(join(distWeb, 'index.js'));

const skipReason = !enabled
  ? 'GRAPHLAW_BROWSER_TESTS !== 1'
  : pw === null
    ? 'playwright-core not importable'
    : !distReady
      ? 'dist-web/index.js not built (run npm run build)'
      : null;

// Shared across the single browser test: latest POSTed /result body.
let lastResult: unknown = null;
let resolveResult: (v: unknown) => void = () => {};
const resultPromise = new Promise<unknown>((res) => {
  resolveResult = res;
});

const MIME: Record<string, string> = {
  '.js': 'text/javascript',
  '.mjs': 'text/javascript',
  '.wasm': 'application/wasm',
  '.json': 'application/json',
  '.html': 'text/html',
};

const server = http.createServer((req, res) => {
  if (req.method === 'POST' && req.url === '/result') {
    const chunks: Buffer[] = [];
    req.on('data', (c: Buffer) => chunks.push(c));
    req.on('end', () => {
      lastResult = JSON.parse(Buffer.concat(chunks).toString('utf8'));
      res.writeHead(204).end();
      resolveResult(lastResult);
    });
    return;
  }
  if (req.url === '/') {
    res
      .writeHead(200, { 'content-type': 'text/html' })
      .end(
        '<!doctype html><html><body><script type="module">' +
          `import { callRaw } from '/index.js';\n` +
          `try {\n` +
          `  const envelope = await callRaw({ op: 'capabilities' });\n` +
          `  await fetch('/result', { method: 'POST', body: JSON.stringify(envelope) });\n` +
          `} catch (e) {\n` +
          `  await fetch('/result', { method: 'POST', body: JSON.stringify({ ok: false, error: String(e) }) });\n` +
          `}\n` +
          '</script></body></html>',
      );
    return;
  }
  const rel = (req.url ?? '/').split('?')[0]!.replace(/^\//, '');
  const file = join(distWeb, rel);
  if (existsSync(file)) {
    res.writeHead(200, { 'content-type': MIME[extname(file)] ?? 'application/octet-stream' }).end(
      readFileSync(file),
    );
    return;
  }
  res.writeHead(404).end();
});

const serverReady = distReady
  ? new Promise<{ port: number }>((resolve, reject) => {
      server.listen(0, '127.0.0.1', () => {
        const addr = server.address();
        if (addr && typeof addr === 'object') resolve({ port: addr.port });
        else reject(new Error('no ephemeral port'));
      });
      server.on('error', reject);
    })
  : null;

afterAll(async () => {
  await new Promise<void>((res) => server.close(() => res()));
});

(skipReason ? describe.skip : describe)(`browser runtime in headless chromium (${skipReason})`, () => {
  it('answers capabilities with ok:true and 14 ops over real fetch + wasm', { timeout: 120_000 }, async () => {
    const { port } = await serverReady!;
    const browser = await pw!.chromium.launch({ headless: true });
    try {
      const page = await browser.newPage();
      await page.goto(`http://127.0.0.1:${port}/`);
      const envelope = (await Promise.race([
        resultPromise,
        new Promise<null>((res) => setTimeout(() => res(null), 90_000)),
      ])) as { ok?: boolean; ops?: unknown[]; error?: string } | null;
      expect(envelope, 'page never posted a result (timeout or module load failure)').not.toBeNull();
      expect(envelope!.ok, JSON.stringify(envelope)).toBe(true);
      expect(Array.isArray(envelope!.ops)).toBe(true);
      expect(envelope!.ops).toHaveLength(14);
      expect(lastResult).toEqual(envelope);
    } finally {
      await browser.close();
    }
  });
});
