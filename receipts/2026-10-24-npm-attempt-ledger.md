# npm publish attempt ledger — run 40084544193

Ledger of every npm publish attempt against `@seanchatmangpt/graphlaw@26.10.22`
on Actions run **40084544193** (`publish-npm`, push event on tag `v26.10.22`,
head `29c6d98`), plus the tarball-identity witness. Every output block below
is verbatim command output captured live on 2026-10-24 by this lane; each
command re-runs read-only. (This file supersedes, same path, a corrupt
first draft written seconds earlier that embedded hand-typed pseudo-output;
the draft was never cited and is fully replaced by this version.)

- **Identity**: network-read lane over run 40084544193 of
  `seanchatmangpt/graphlaw`, tag `v26.10.22` (head `29c6d98`); subject =
  npm publish attempts for `@seanchatmangpt/graphlaw@26.10.22` and the
  CI-packed tarball identity.
- **Authority**: write lease = this file only; no rerun, no publish, no
  workflow dispatch, no git transitions executed by this lane.
- **Consequence**: one file created (this one); zero code edits. Network
  reads only: `gh` run/log/artifact reads + one artifact download.
- **Replay**: every command shown above its output block re-runs read-only;
  artifact download re-witnesses `shasum`/`b3sum` against the values below.
- **Standing**: A2 is the latest attempt (failure, E404, Trusted Publisher
  still unconfigured as of 2026-10-24 18:26Z); A3 is PENDING-OPERATOR.
  Re-read run state at use time — any newer attempt supersedes A2's rows.

## Attempt register

| id | attempt | When (UTC) | Trigger / context | Outcome | Load-bearing evidence |
|---|---|---|---|---|---|
| A0-1 | 1 | created 09:41:33Z | workflow-fix push (stage WASI SDK + duckdb wasm32 archive) | failure — superseded pre-fix | A0 embeds (context-only) |
| A0-2 | 1 | created 09:53:04Z | workflow-fix push (byte-authority gate) | failure — superseded pre-fix | A0 embeds (context-only) |
| A0-3 | 1 | created 09:59:10Z | workflow-fix push (tarball-name resolution) | failure — superseded pre-fix | A0 embeds (context-only) |
| A1 | 1 | publish step 10:07:23Z | tag `v26.10.22` push | **E404** at the registry `PUT` — Trusted Publisher unconfigured | PUBLISH-STATUS "The blocker" (attempt-1 log tail embedded there); superseded by A2 as latest |
| A2 | 2 | job 18:25:14–18:25:54Z | `gh run rerun --failed` after the first TP-config attempt | **E404 again** — TP still unconfigured | A2 embeds below (latest, load-bearing) |
| A3 | — | pending operator | post-TP-config rerun | PENDING-OPERATOR | protocol: `docs/jira/v26.10.23/PUBLISH-COMPLETION-RUNBOOK.md` §A.2 |

A0 note: the three A0 runs are closeout-classified superseded pre-fix
attempts of the tag-era workflow (each `attempt: 1`, `conclusion: failure`,
`status: completed` — outputs embedded verbatim below). A1's conclusion is
carried by the run's attempt history; the attempt-2 rerun (`gh run rerun
--failed`) re-ran only the failed publish job, so attempt 1's validate job
result and the PUBLISH-STATUS-embedded attempt-1 log tail (10:07:23Z E404)
remain the A1 evidence of record.

## A0 — superseded pre-fix runs (verbatim)

```text
$ gh run view 40082955181 -R seanchatmangpt/graphlaw --json conclusion,status,attempt,displayTitle,createdAt --jq '{conclusion,status,attempt,displayTitle,createdAt}'
{"attempt":1,"conclusion":"failure","createdAt":"2026-10-24T09:41:33Z","displayTitle":"fix(ci): publish-npm — stage WASI SDK + duckdb wasm32 archive (CI had…","status":"completed"}

$ gh run view 40083716865 -R seanchatmangpt/graphlaw --json conclusion,status,attempt,displayTitle,createdAt --jq '{conclusion,status,attempt,displayTitle,createdAt}'
{"attempt":1,"conclusion":"failure","createdAt":"2026-10-24T09:53:04Z","displayTitle":"fix(ci): publish-npm — byte-authority gate on the committed artifact …","status":"completed"}

$ gh run view 40084097811 -R seanchatmangpt/graphlaw --json conclusion,status,attempt,displayTitle,createdAt --jq '{conclusion,status,attempt,displayTitle,createdAt}'
{"attempt":1,"conclusion":"failure","createdAt":"2026-10-24T09:59:10Z","displayTitle":"fix(ci): resolve tarball name after npm pack (pack produces it; resol…","status":"completed"}
```

## A2 — run state (verbatim)

```text
$ gh run view 40084544193 -R seanchatmangpt/graphlaw --json conclusion,status,attempt,jobs --jq '{attempt,status,conclusion,jobs:[.jobs[]|{name,status,conclusion,completedAt}]}'
{"attempt":2,"conclusion":"failure","jobs":[{"completedAt":"2026-10-24T18:25:54Z","conclusion":"failure","name":"publish to npm (OIDC trusted publishing, no token)","status":"completed"},{"completedAt":"2026-10-24T10:06:48Z","conclusion":"success","name":"validate and pack (version + blake3 digest gates)","status":"completed"}],"status":"completed"}
```

Field-level projection of the same run captured earlier in this session
(verbatim JSON, `gh run view 40084544193 -R seanchatmangpt/graphlaw --json
conclusion,status,attempt,jobs`): publish job `120949032543` — started
18:25:14Z; step 8 "Publish to npm" failure at 18:25:52Z; job completed
18:25:54Z, conclusion failure. Validate job `120949033879` —
10:05:45–10:06:48Z, success (untouched by the `--failed` rerun; steps 1-10
all success, including step 7 "Re-verify tarball blake3" and step 9 "Pack
(real, not dry-run) with tarball drift check").

## A2 — failed-step log tail (verbatim)

```text
$ gh run view 40084544193 -R seanchatmangpt/graphlaw --log-failed | tail -8
publish to npm (OIDC trusted publishing, no token)	Publish to npm	2026-10-24T18:25:52.2407354Z npm error 404 Not Found - PUT https://registry.npmjs.org/@seanchatmangpt%2fgraphlaw - Not found
publish to npm (OIDC trusted publishing, no token)	Publish to npm	2026-10-24T18:25:52.2408499Z npm error 404
publish to npm (OIDC trusted publishing, no token)	Publish to npm	2026-10-24T18:25:52.2413741Z npm error 404  The requested resource '@seanchatmangpt/graphlaw@26.10.22' could not be found or you do not have permission to access it.
publish to npm (OIDC trusted publishing, no token)	Publish to npm	2026-10-24T18:25:52.2414921Z npm error 404
publish to npm (OIDC trusted publishing, no token)	Publish to npm	2026-10-24T18:25:52.2415790Z npm error 404 Note that you can also install from a
publish to npm (OIDC trusted publishing, no token)	Publish to npm	2026-10-24T18:25:52.2418561Z npm error 404 tarball, folder, http url, or git url.
publish to npm (OIDC trusted publishing, no token)	Publish to npm	2026-10-24T18:25:52.2433142Z npm error A complete log of this run can be found in: /home/runner/.npm/_logs/2026-10-24T18_25_47_942Z-debug-0.log
publish to npm (OIDC trusted publishing, no token)	Publish to npm	2026-10-24T18:25:52.2911331Z ##[error]Process completed with exit code 1.
```

## Tarball identity — witnessed from the CI artifact (verbatim)

```text
$ gh api repos/seanchatmangpt/graphlaw/actions/runs/40084544193/artifacts --jq '.artifacts[] | {name, size_in_bytes, id, expired}'
{"expired":false,"id":12555938743,"name":"npm-tarball-40084544193","size_in_bytes":12582011}

$ gh run download 40084544193 -R seanchatmangpt/graphlaw -n npm-tarball-40084544193 -D /tmp/lane-npmtar.wNOLfw
$ ls -la /tmp/lane-npmtar.wNOLfw
-rw-r--r--@  1 sac  wheel  12625740 Oct 24 13:21 seanchatmangpt-graphlaw-26.10.22.tgz

$ shasum -a 1 /tmp/lane-npmtar.wNOLfw/seanchatmangpt-graphlaw-26.10.22.tgz
115e5024840955e741c788edcacf3358d2ff277a  /tmp/lane-npmtar.wNOLfw/seanchatmangpt-graphlaw-26.10.22.tgz
```

- Filename `seanchatmangpt-graphlaw-26.10.22.tgz`, 12,625,740 B (12.6 MB)
  — matches the run-log claim (137 files, 12.6 MB packed).
- shasum **`115e5024840955e741c788edcacf3358d2ff277a`** — matches the
  PUBLISH-STATUS row for the run log byte-identity claim.

File count and the wasm pin check inside the same tarball (verbatim):

```text
$ tar -tzf /tmp/lane-npmtar.wNOLfw/seanchatmangpt-graphlaw-26.10.22.tgz | grep -c 'package/'
137

$ tar -xzf /tmp/lane-npmtar.wNOLfw/seanchatmangpt-graphlaw-26.10.22.tgz -C /tmp/lane-npmtar.wNOLfw package/wasm/graphlaw.wasm package/package.json
$ b3sum /tmp/lane-npmtar.wNOLfw/package/wasm/graphlaw.wasm
58a4409505f32b5d9aebc8c0ce2e2adeaba05c8c4b55f951a244ab4ad7d2280a  /tmp/lane-npmtar.wNOLfw/package/wasm/graphlaw.wasm
$ cat /Users/sac/graphlaw/priv/graphlaw.wasm.blake3
58a4409505f32b5d9aebc8c0ce2e2adeaba05c8c4b55f951a244ab4ad7d2280a  priv/graphlaw.wasm
$ grep -m1 '"version"' /tmp/lane-npmtar.wNOLfw/package/package.json
  "version": "26.10.22",
```

The wasm shipped inside the CI-packed tarball is byte-identical to the
4-surface pin `58a44095…d2280a` (58,124,612 B) — the A.3 post-publish
`b3sum(node_modules/.../graphlaw.wasm) == pin` check is pre-witnessed here
on the exact bytes npm will receive.

## A3 — pending operator (typed)

A3 = the post-Trusted-Publisher rerun of run 40084544193. Machine-side it
is fully specified (`gh run rerun 40084544193 -R seanchatmangpt/graphlaw`
after the TP config is saved, retry-loop + classification per the runbook
§A.2); it is operator-gated because the TP configuration is an npmjs.com
console action requiring registry auth this lane must not hold. Until A3
reaches `conclusion: success`, PUBLISH-STATUS row 6 stays RED and row 2
stays `26.10.14`.

## Falsifiers

- `gh run view 40084544193 --json conclusion` returning `success` (or a
  newer successful run) flips A3 to SHIPPED and discharges the npm half of
  the completion runbook.
- The downloaded artifact re-download producing a different shasum
  invalidates this receipt's identity section (re-derive from a fresh
  artifact listing first — the artifact id is pinned above).
- An npm registry read showing `26.10.22` published (via any path)
  supersedes the attempt standing entirely.

## See Also

- `/Users/sac/graphlaw/docs/jira/v26.10.23/PUBLISH-STATUS.md` (rows 2/5/6)
- `/Users/sac/graphlaw/docs/jira/v26.10.23/PUBLISH-COMPLETION-RUNBOOK.md` (§A.2, §A.3)
- `https://github.com/seanchatmangpt/graphlaw/actions/runs/40084544193`
