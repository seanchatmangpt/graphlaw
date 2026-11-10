# RECEIPTS-INDEX

renamed from RECEIPTS.md 2026-10-24; per-version ledgers live under
`docs/jira/<version>/`.

Pointer index for durable witness receipts. Newest first. As of 2026-11-09 the older
receipt files (`receipts/*.md` other than `receipts/2026-10-24-log-retention.md`) and the
superseded v26.10.22–v26.10.26 milestone docs were archived out of the tree; recover any
of them from git history (`git log --diff-filter=D --name-only -- receipts/ docs/jira/`).
Machine receipts (JSON) are append-only; rows point to their witnesses.

| date | receipt | subject | verdict | machine receipt (sha256) |
|---|---|---|---|---|
| 2026-10-28 | `docs/jira/v26.10.28/RECEIPTS.md` | v26.10.28 release: purrdf fork swap @ 2223f894 (38 fork-sourced packages, `vendor/`+`patch/`+`wasi-patched-deps` deleted), named-court ladder + closing wave (census 13/13, cliff 10/10, chicago 12/12, dialect 36/36, statement-kinds 8/8, registry 22/22, differential 2/2, size 4/4 at 80MiB SCAR, SPC Cpk 6.098/14.41, BEAM 2/2 packet-4, install-from-scratch + downstream-consumer rehearsals PASS, GitHub release v26.10.28 live with 8 blake3-sidecarred assets) | PARTIAL_ALIVE (mutation court = the named open gate; npm/hex publish coordinator-pressed) | — (md receipt, b3sum `ebd43a7bc936b350f195779123809f46585ab10e685b6877f0163f56ca5a0d24`) |
| 2026-10-24 | `receipts/2026-10-24-log-retention.md` | log retention: volatile battery + mutation evidence copied into `receipts/logs/` (chain-attested location) | n/a (retention index, no verdict) | — (md receipt) |
