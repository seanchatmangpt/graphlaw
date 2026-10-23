# SPC Court mktemp Robustness Fix — L57 Receipt

Date: 2026-10-22. Lane: L57 on /Users/sac/graphlaw (canonical checkout).

## Finding (L50, cited reproducer)

`scripts/spc_court.sh:37` used `mktemp "${TMPDIR:-/tmp}/spc-court.XXXXXX.json"`.
Under a polluted TMPDIR (6178 entries with stale `spc-court.*` files) the gate
died PRE-GATE with `mkstemp failed ... File exists` — gate plumbing failure,
not a measurement verdict. L50 witnessed; pre-fix repro not re-run (fix already
in; L50's reproducer stands).

## Fix

`scripts/spc_court.sh` (temp-file block only, measurement untouched):
- fresh `mktemp -d` scratch dir (`spc-court.XXXXXXXX`), atomic creation, no
  name collision with stale entries;
- fallback loop over `"${TMPDIR:-/tmp}"` then `/tmp`;
- explicit exit 2 with a message if no scratch dir can be created;
- `trap 'rm -rf "$SPC_SCRATCH"' EXIT` cleans the whole dir.

## Run (ambient TMPDIR, no env massaging)

Command: `bash scripts/spc_court.sh` — exit **1**.

Ambient TMPDIR resolved to `/Users/sac/.cache/tmp`; export path shows the new
scratch behavior:

```
SPC_EXPORT path=/Users/sac/.cache/tmp/spc-court.j1w7X7g5/spc-court.json samples=999999
```

Verdict (from the gate's own receipt, real sample n=999,999):

```
"cpk": 0.2948,
"p99_ns": 250,
"verdict": "FAIL",
"reasons": [ "cpk 0.2948 < 1.33" ]
spc_court: FAIL — process out of control (see receipt above)
EXIT=1
```

Also observed in the bench's internal gates (informational, owned by
tc_hot_01, not the SPC court): clone-path p99 = 250ns at budget, variance
self-check VARIANCE-EXCEEDED (spread 22.69% vs 5% budget).

## Verdict

PASS for this lane: the gate ran its full measurement path, printed a typed
verdict, and exited 1 on the honest red (Cpk 0.2948 < 1.33). No plumbing
crash. The Cpk FAIL itself is the standing process state, not a lane defect.

Standing receipt written by the gate:
`receipts/spc-2026-10-22.json` (SMOKE, inject_outlier=false, samples=999999).
