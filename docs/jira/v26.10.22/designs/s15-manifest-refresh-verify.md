# S15 — Skeptic verification of commit-manifest refresh #3 (L173)

Subject: tree 615dbfa, /Users/sac/graphlaw, 2026-10-22. Method: programmatic
assignment of every `git status --porcelain` path (276 entries, snapshot
`/tmp/s15-status.txt`) against the bucket path sets from
`designs/commit-manifest.md` + `/tmp/l124-commits/*.txt`; conflicts resolved
by the manifest's own precedence rules (explicit assignment beats prefix;
designs/* → bucket 7 except commit-manifest.md → 6).

## Verdicts

| Check | Verdict |
|---|---|
| 1. Completeness (every path exactly one bucket) | **FAIL — 4 unassigned** |
| 2. Broken-tree rule (facade + dir same bucket) | **PASS** |
| 3. Ordering (bucket 8 last, no earlier bucket touches its paths) | **PASS** |
| 4. OPERATOR-DECISION-PENDING marker on re-pin bucket | **PASS** |
| 5. Draft messages (subjects ≤72, bodies cover buckets) | **FAIL — 3 subjects over; bucket 7 body under-covers** |

## Check 1 — FAIL: 4 unassigned paths

- `.gitignore` (M) — unassigned. Manifest's exclusion section defers a
  ".gitignore follow-up" to the coordinator but never assigns the modified
  `.gitignore` itself to any bucket.
- `deny.toml` (M) — unassigned; named nowhere in the manifest or drafts.
- `scripts/mutation_court.sh` (M) — unassigned. Bucket 5's scripts list is
  {wasm_audit.sh, gen_wasm4pm_graphlaw_vectors.py, gen_w3c_manifest.sh};
  `.github/workflows/mutation.yml` rides bucket 5, but the script it invokes
  does not travel with any bucket.
- `src/bin/dl_dualroute.rs` (M) — unassigned; named nowhere.

Excluded-with-receipt (documented in manifest, counted as the exclusion
bucket, not a gap): `.claude/agents/{court-runner,sql-compiler,wasm-op}.md`
(FLAGGED ambiguous), `.claude/hooks/__pycache__/`,
`.claude/hooks/{disk_guard,manifest_guard}.py`, `.claude/settings.json`.

Double-assigned: **none** after applying the manifest's own precedence.

Census drift: manifest claims 270 status entries; live count is **276**
(rerun `git status --porcelain=v1 | wc -l` → 276). Six paths moved since the
manifest snapshot; the 4 unassigned above may be exactly that delta — the
manifest must be re-derived at integration, not committed as-is.

## Check 2 — PASS: broken-tree rule

All 8 facade/dir pairs verified in the same bucket, and no half appears
elsewhere: parser.rs+parser/ and reasoner.rs+reasoner/ (bucket 1);
sql_ast.rs+sql_ast/ and dialect.rs+dialect/ (2); query_ledger.rs+dir and
receipt_chain.rs+dir (3); registry.rs+registry/ and abi.rs+abi/ (8). All
wired dirs present in status (untracked `?? src/sql_ast/` etc. all seen).
Residual: bucket 2 carries `src/lib.rs` (already flagged in the manifest) —
if it wires registry/abi mods, bucket 8's facade commit would not be the
first touch of that surface; coordinator must grep lib.rs mod lines before
executing.

## Check 3 — PASS: ordering

No bucket 1-7 set contains any of bucket 8's paths (registry.rs, registry/,
abi.rs, abi/, abi_manifest.rs, graphlaw_registry.rs, registry/* pins,
ASSETS.blake3, contracts/abi-manifest.json, registry_dispatch,
authority_manifest) — programmatic intersection empty. Bucket 8's draft
ends with "commit LAST" + re-pin re-run rule. Note the manifest's own
caveat stands: any bucket 1-7 change to hashed inputs requires re-running
the registry sync before writing pins.

## Check 4 — PASS: marker

`OPERATOR-DECISION-PENDING` present verbatim in
`/tmp/l124-commits/08-registry-repin.txt` (bracketed paragraph naming
designs/dual-artifact-policy-design.md, bucket 7) and in the manifest's
bucket-8 row and prose.

## Check 5 — FAIL: draft messages

Subject lengths (git subject convention ≤72):

- 01-pre-writes.txt: 63 — OK
- 02-sql-surface.txt: **90 — over** ("feat(sql): sql surface + seal + refusals
  reconciliation (incl. sql_ast/dialect switchover)")
- 03-ring-ledger.txt: **104 — over** ("feat(evidence): ring/ledger hardening +
  receipt chain sync (incl. query_ledger/receipt_chain switchover)")
- 04: 54 OK · 05: 51 OK · 06: 61 OK · 07: 61 OK
- 08-registry-repin.txt: **98 — over** ("chore(registry): registry/ABI
  switchover + regenerate capability registry + asset pins (generated)")

Body coverage: buckets 1-6 and 8 enumerate their surfaces adequately. Bucket
7's body names ~56 designs files but ~36 status paths under
docs/jira/v26.10.22/designs/ appear in no draft body, incl. modified
ctq-traceability.md, hot-path-baseline.md, P28-SIL-specs.md,
R2-capability-routes.md, spof-scan.md, TC-HOT-benchmarks.md, and untracked
claude-policy-commit-note.md, commit-drafts-validation.md,
commit-receipt-provenance-gap.md, eyeron-switchover-fixforward.md,
g2-repin-prep.md, gate-rule-changes-receipt.md, ggen-toml-tempath-proposal.md,
laundering-count-reconcile.md, m4-wasm-boundary-receipt.md,
OPERATOR-DECISIONS.md, p7-final-regen-prestage.md, post-commit-runbook.md,
receipt-surface-post-switchover.md, registry-drift-scope.md,
repin-staging-gap.md, s1/s2/s4/s7/s8/s9/s10/s11/s13/s14-*-verify.md,
spc-red-remediation-design.md, staleness-76-triage.md,
surface-six-entry-patch.md, switchover-provenance-census.md,
tc-hot-01-errc-receipt.md, vecgen-v1-backmigration.md,
wasm-duckdb-op-wired.md, wasm-repin-receipt.md, wasmi-eh-research.md.
The manifest table's prose ("designs/* EXCEPT commit-manifest.md") covers
them, so this is a body-accuracy gap, not an assignment gap — but a commit
body that enumerates a partial list while the commit carries the rest is a
misdescribing commit.

## Overall

**BLOCKED-ON-MANIFEST**: refresh #3 is not executable as written. Required
before coordinator runs it: assign `.gitignore`, `deny.toml`,
`scripts/mutation_court.sh`, `src/bin/dl_dualroute.rs` (or loudly exclude
each with a reason); shorten subjects 02/03/08 to ≤72; fix bucket 7 body to
"designs/* except commit-manifest.md" form instead of a partial enumeration;
re-run the census (270 → 276 drift); grep `src/lib.rs` for registry/abi mod
wiring before ordering bucket 2 ahead of 8.

Verification commands: `git status --porcelain | sort` (exit 0, 276 lines);
python assignment script (exit 0; unassigned/double lists above); subject
lengths measured from first line of each /tmp/l124-commits/*.txt.
