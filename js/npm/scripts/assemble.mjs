// assemble.mjs — assembles the three distribution targets from the tsc output
// (tsc-out, rootDir src):
//
//   dist-nodejs   — as shipped: tsc-out copy + raw/ re-export subpath.
//   dist-web      — tsc-out copy with the runtime module swapped for the web
//                   entry (dist-web/runtime.js re-exports ./runtime.web.js).
//   dist-bundler  — identical assembly to dist-web; bundler consumers get the
//                   same ESM and their bundler resolves the wasm URL.
//
// tsc only compiles files reachable from src/index.ts, so
// tsc-out/runtime.web.js may be absent. In that case the script compiles
// src/runtime.web.ts standalone (its ./errors.js and ./wire.js imports are
// resolved from the already-copied tsc-out files in the target dir) and copies
// the result in.
import { cpSync, existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { dirname } from 'node:path';
import { tmpdir } from 'node:os';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const pkgRoot = resolve(here, '..');

const tscOut = join(pkgRoot, 'tsc-out');
if (!existsSync(join(tscOut, 'index.js'))) {
  console.error('assemble: tsc-out/index.js missing — run `tsc -p tsconfig.build.json` first');
  process.exit(1);
}

// --- dist-nodejs (byte-identical to the previous single-target output) -----

const dist = join(pkgRoot, 'dist-nodejs');
mkdirSync(dist, { recursive: true });
cpSync(tscOut, dist, { recursive: true });

// graphlaw/raw subpath re-export (raw.ts is a sibling module of index.ts).
mkdirSync(join(dist, 'raw'), { recursive: true });
const js = `export { callRaw, callRawJson } from '../raw.js';\n`;
writeFileSync(join(dist, 'raw', 'index.js'), js);
const dts = `export { callRaw, callRawJson } from '../raw.js';\nexport type { AbiRefused, AbiResult } from '../types.js';\n`;
writeFileSync(join(dist, 'raw', 'index.d.ts'), dts);

console.log('assemble: dist-nodejs ready');

// --- dist-web / dist-bundler (web runtime entry) ----------------------------

const runtimeWebReexport = `export * from './runtime.web.js';\n`;
const runtimeWebReexportDts = `export * from './runtime.web.js';\n`;

/**
 * Ensures a compiled runtime.web.js/.d.ts exists in `target` (a fresh tsc-out
 * copy). Compiles src/runtime.web.ts standalone into a temp dir when the main
 * tsc pass did not include it.
 */
function ensureWebRuntime(target) {
  if (existsSync(join(target, 'runtime.web.js'))) return;

  const src = join(pkgRoot, 'src', 'runtime.web.ts');
  if (!existsSync(src)) {
    console.error(`assemble: ${src} missing — cannot assemble web runtime entry`);
    process.exit(1);
  }

  const tmp = mkdtempSync(join(tmpdir(), 'graphlaw-runtime-web-'));
  try {
    execFileSync(
      'npx',
      [
        'tsc', src,
        '--outDir', tmp,
        '--target', 'ES2022',
        '--module', 'NodeNext',
        '--moduleResolution', 'NodeNext',
        '--strict',
        '--skipLibCheck',
      ],
      { cwd: pkgRoot, stdio: 'inherit' },
    );
  } finally {
    // Copy before cleanup if compile succeeded; cleanup regardless.
    const outJs = join(tmp, 'runtime.web.js');
    if (existsSync(outJs)) {
      for (const f of ['runtime.web.js', 'runtime.web.d.ts']) {
        if (existsSync(join(tmp, f))) cpSync(join(tmp, f), join(target, f));
      }
    }
    rmSync(tmp, { recursive: true, force: true });
  }

  if (!existsSync(join(target, 'runtime.web.js'))) {
    console.error('assemble: standalone tsc of src/runtime.web.ts produced no runtime.web.js');
    process.exit(1);
  }
}

for (const name of ['dist-web', 'dist-bundler']) {
  const target = join(pkgRoot, name);
  mkdirSync(target, { recursive: true });
  cpSync(tscOut, target, { recursive: true });

  ensureWebRuntime(target);

  // Replace the node runtime module with a thin re-export of the web entry.
  writeFileSync(join(target, 'runtime.js'), runtimeWebReexport);
  writeFileSync(join(target, 'runtime.d.ts'), runtimeWebReexportDts);

  // graphlaw/raw subpath re-export, same as dist-nodejs.
  mkdirSync(join(target, 'raw'), { recursive: true });
  writeFileSync(join(target, 'raw', 'index.js'), js);
  writeFileSync(join(target, 'raw', 'index.d.ts'), dts);

  console.log(`assemble: ${name} ready (runtime -> runtime.web)`);
}
