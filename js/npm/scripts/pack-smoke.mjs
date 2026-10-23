// Used by `npm run lint:pkg`/CI; safe to run before build (dist checks downgrade to WARN).
//
// Pack smoke: verifies `npm pack --dry-run` contents against the package.json
// `files` allowlist and structural invariants:
//   - every packed entry is inside the allowlist (FAIL on offenders, listed)
//   - no node_modules/, test/, src/, scripts/, tsconfig*, vitest* leakage (FAIL)
//   - wasm/graphlaw.wasm present and within its class band: legacy
//     pre-duckdb <10 MB, DuckDB-linked 40-64 MiB (FAIL)
//   - total packed size < 68MB (FAIL)
//   - dist-nodejs/index.js, dist-nodejs/raw/index.js, dist-bundler/index.js,
//     dist-web/index.js present (WARN before build, so pre-build runs pass)
import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const PKG_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

const ALLOWED_DIRS = ["dist-nodejs/", "dist-web/", "dist-bundler/", "wasm/"];
const ALLOWED_FILES = ["LICENSE", "README.md", "CHANGELOG.md", "package.json"];
const REQUIRED_DIST = [
  "dist-nodejs/index.js",
  "dist-nodejs/raw/index.js",
  "dist-bundler/index.js",
  "dist-web/index.js",
];
const WASM_ENTRY = "wasm/graphlaw.wasm";
// Size-band classes (mirrors scripts/wasm_audit.sh 64 MiB ceiling):
//   - legacy pre-duckdb artifact: <10 MB (historical; current priv/graphlaw.wasm
//     at ~7.1 MB is this class until the coordinator re-pin)
//   - DuckDB-linked artifact (M4 witness): >=40 MB proves the engine link,
//     hard ceiling 64 MiB (67108864)
const MIB = 1024 * 1024;
const WASM_LEGACY_MIN_BYTES = 1 * MIB; // non-trivial module, any class
const WASM_LEGACY_MAX_BYTES = 10 * MIB;
const WASM_ENGINE_MIN_BYTES = 40 * MIB; // wasm >= 40 MB proves the engine link
const WASM_HARD_CEILING_BYTES = 64 * MIB; // 67108864
const TOTAL_MAX_BYTES = 68 * MIB; // wasm ceiling 64 MiB + package overhead
const FORBIDDEN = [
  /^node_modules\//,
  /^test\//,
  /^tests\//,
  /^src\//,
  /^scripts\//,
  /^tsconfig[^/]*\.json$/,
  /^vitest[^/]*$/,
];

function isAllowed(p) {
  if (ALLOWED_FILES.includes(p)) return true;
  return ALLOWED_DIRS.some((d) => p.startsWith(d));
}

function isForbidden(p) {
  return FORBIDDEN.some((re) => re.test(p));
}

function fmtBytes(n) {
  return `${(n / 1024 / 1024).toFixed(2)}MB`;
}

const fails = [];
const warns = [];

let raw;
try {
  raw = execFileSync("npm", ["pack", "--dry-run", "--json"], {
    cwd: PKG_ROOT,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
    stdio: ["ignore", "pipe", "inherit"],
  });
} catch (err) {
  console.error(`FAIL: npm pack --dry-run --json exited ${err.status ?? "?"}`);
  process.exit(1);
}

// npm may emit non-JSON lines (notices) before the JSON array; take the array.
const start = raw.indexOf("[");
let report;
try {
  report = JSON.parse(raw.slice(start));
} catch (err) {
  console.error(`FAIL: could not parse npm pack JSON output: ${err.message}`);
  process.exit(1);
}
const pkg = report[0];
if (!pkg || !Array.isArray(pkg.files)) {
  console.error("FAIL: unexpected npm pack output shape (no files array)");
  process.exit(1);
}

const entries = pkg.files;
const offenders = entries.filter((f) => !isAllowed(f.path));
if (offenders.length > 0) {
  fails.push(
    `entries outside the files allowlist:\n    ${offenders
      .map((f) => `${f.path} (${fmtBytes(f.size || 0)})`)
      .join("\n    ")}`,
  );
}

const leaked = entries.filter((f) => isForbidden(f.path));
if (leaked.length > 0) {
  fails.push(
    `forbidden paths leaked into the tarball:\n    ${leaked.map((f) => f.path).join("\n    ")}`,
  );
}

const paths = new Set(entries.map((f) => f.path));
const wasm = entries.find((f) => f.path === WASM_ENTRY);
if (!wasm) {
  fails.push(`${WASM_ENTRY} missing from packed output`);
} else {
  const size = wasm.size || 0;
  if (size >= WASM_ENGINE_MIN_BYTES) {
    // Engine-link class: bounded by the 64 MiB hard ceiling.
    if (size > WASM_HARD_CEILING_BYTES) {
      fails.push(
        `${WASM_ENTRY} is ${fmtBytes(size)}, exceeds hard ceiling ${fmtBytes(WASM_HARD_CEILING_BYTES)} (64 MiB)`,
      );
    }
  } else if (size < WASM_LEGACY_MIN_BYTES || size >= WASM_LEGACY_MAX_BYTES) {
    // Legacy pre-duckdb class: historical <10 MB band.
    fails.push(
      `${WASM_ENTRY} is ${fmtBytes(size)}, outside legacy band [${fmtBytes(WASM_LEGACY_MIN_BYTES)}, ${fmtBytes(WASM_LEGACY_MAX_BYTES)}) and below engine-class threshold ${fmtBytes(WASM_ENGINE_MIN_BYTES)}`,
    );
  }
}

const total = entries.reduce((sum, f) => sum + (f.size || 0), 0);
if (total >= TOTAL_MAX_BYTES) {
  fails.push(
    `total packed size ${fmtBytes(total)} >= limit ${fmtBytes(TOTAL_MAX_BYTES)}`,
  );
}

for (const req of REQUIRED_DIST) {
  if (!paths.has(req)) warns.push(`${req} missing (not built yet?)`);
}

console.log(`pack smoke: ${pkg.id}`);
console.log(`  entries: ${entries.length}, total: ${fmtBytes(total)}`);
for (const w of warns) console.log(`  WARN: ${w}`);
for (const f of fails) console.error(`  FAIL: ${f}`);

if (fails.length > 0) {
  console.error(`pack smoke FAILED: ${fails.length} violation(s)`);
  process.exit(1);
}
console.log("pack smoke PASSED");
