# code-alignment — graphlaw v26.11.17

Standing: PASS

## Identity
- Path: `/Users/sac/graphlaw`
- Branch at generation: main
- HEAD: 5facaab4a3c953d2dc4b91b16fc6a3ef24266d46
- Tracked files: 6550
- Tracked docs markdown: 111

## Build manifests (tracked at the commit above)
- `Cargo.toml`
- `CLAUDE.md`
- `README.md`

## Top-level layout (tracked file counts)
- `.`: 18
- `.cargo`: 7
- `.claude`: 19
- `.github`: 10
- `.qme`: 1
- `benches`: 13
- `conformance`: 132
- `contracts`: 64
- `courts`: 7
- `designs`: 1
- `docs`: 135
- `examples`: 19
- `ggen`: 12
- `graphlaw_ex`: 71
- `innovation`: 8
- `js`: 45
- `ontologies`: 453
- `packs`: 4
- `priv`: 2
- `python`: 12
- `qualification`: 26
- `queries`: 54
- `receipts`: 3699
- `registry`: 8
- `scripts`: 22
- `src`: 61
- `tests`: 1639
- `tests_py`: 3
- `wasm`: 5

## Prior doc waves
- `docs/jira/v26.10.22`
- `docs/jira/v26.10.28`
- `docs/jira/v26.11.16`

## Falsifiers
- Any tracked path above absent at HEAD, or a count differing from `git ls-files`.
- HEAD sha not equal to or an ancestor of the live HEAD.

## Acceptance
```bash
python3 ~/zcode-cli/scripts/docs_v261117.py validate /Users/sac/graphlaw
```
Expected: exit 0 and `PASS`.

## See Also
- `~/zcode-cli/scripts/docs_v261117.py`
