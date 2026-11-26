# Changelog since v26.10.28 (window v26.11.17)

Generated 2026-11-17 from git. HEAD `dd6a80e` (2026-11-16, "preserve(v26.11.16): migration snapshot").

## Baseline and its limits
The tag is not available in this clone, so the baseline is approximate:
```
$ git tag -l | wc -l                      -> 0
$ git rev-parse --is-shallow-repository   -> true
$ git cat-file -t 82814ac                 -> fatal: Not a valid object name 82814ac
$ git log --oneline | wc -l               -> 50
```
- docs/jira/v26.10.28/TAG records annotated tag 1e72b4c peeling to 82814ac; neither is reachable here (UNVERIFIED).
- Working baseline: `320e2e5..HEAD` (v26.10.28 window close-out receipt, 2026-10-29). `git fetch --unshallow --tags` would make this exact.
- 15 older commits (6a6a863..eba76da, 2026-10-28/29) sit before the anchor; their relation to the tag is UNVERIFIED and they are not counted.

## Counts (320e2e5..HEAD)
```
26 2026-10-29 / 2 2026-11-09 / 1 2026-11-16     (29 commits)
1083 files changed, 10970 insertions(+), 43027 deletions(-)
226 A / 716 D / 137 M / 4 R
```
Deletions are mostly untracked build output and archived docs, not shipped code removal. Top dirs: receipts/ 23.3%, docs/jira/v26.10.22/designs/ 13.4%, graphlaw_ex/_build/** (untracked in 2958db5).

## Commits by theme
**Features:** 14df7b9 GL28-14 MCP manifest projection (emit_mcp.rs, mcp-manifest.json, tests/mcp_manifest.rs); 09bafd2 GL28-13 ICU-root-removal relink (62.9 MB); fc1576f / 199cca1 graphlaw_ex source_digest (GL28-1 b+c); 77016e2 lithography NoReturn.
**graphlaw_ex:** 49b6316 host layer sync; 43ea53e default `mix test` runs the 36 engine courts; 41f1f6a re-vendor kernel wasm pin.
**CI / supply chain:** b2e3fce elixir.yml; 51691ec libcurl apt step; ad98993 wrap-fcntl shim + bindgen sysroot; d57da0a deny wildcards warn; 38124c4 zstd advisory trio receipted.
**Receipts / GL28 closure:** 95cd3df GL28-13 closed; cd69757 GL28-1 ALIVE; f9695dc incident (court inversion raced into eba76da); d94cdba restore src/abi.rs; 6 transition-mint commits (095ce07, 8a19154, 95aa4fc, 9998ac8, ccaa171, dd6a80e).
**Docs:** 9b407d6, 3147213, d8fce72, fe8309b, 6961d73 (2026-11-09 docs-triage: archive superseded milestone docs/receipts).
**Cleanup:** 2958db5 untrack graphlaw_ex/_build.

## Not verified
npm/Hex publishes after the tag; CI results for any commit; tags/releases after 2026-11-09. Theme assignment of small commits is from subjects only.
