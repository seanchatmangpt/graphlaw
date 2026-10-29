# Log retention — volatile battery + mutation inventory evidence into receipts/logs/ (2026-10-24)

## Header

- Repo: `/Users/sac/graphlaw` (canonical checkout, one-tree fan-out)
- HEAD: `a00770b4998a55ed4ee397e3e2a1f5a880c51db0` (branch `main`)
- Lane: execution lane, log-retention aspect; lease = `receipts/logs/` (new) +
  this one receipt. No git mutations (coordinator owns git).
- Retention stamp: `20261024T204225Z` (single batch stamp, UTC).
- Digest tool: `/opt/homebrew/bin/b3sum` (BLAKE3).

## Retention law (P17 G1)

`receipts/logs/<stamp>-<court>.log` is the chain-attested evidence location.
`receipt_chain_sync` enumerates `receipts/` recursively, so every file placed
here becomes BLAKE3-hash-chained at the coordinator's next
`scripts/receipt_chain_sync.sh` run. Volatile `/tmp` is NOT evidence: battery
logs and mutation inventories live in `/tmp` until purged, so this lane copies
them under `receipts/logs/` BEFORE any `/tmp` purge. Every future court run
cites `receipts/logs/` — never a `/tmp` path — as its evidence location.

## File map: volatile path → retained path (b3sum, bytes)

All copies verified byte-exact at copy time (digest(source) == digest(copy)):
ten distinct files, one table row each.

| # | Volatile source | Retained path (`receipts/logs/`) | b3sum | bytes |
|---|---|---|---|---|
| 1 | `/tmp/bat-L1.log` | `20261024T204225Z-battery-L1.log EXIT=0` | `7dec1d1f598a6f1861824f55927bec94e56a3f6785eb208559700bd0647c29b3` | 29071 |
| 2 | `/tmp/bat-L2.log` | `20261024T204225Z-battery-L2.log EXIT=0` | `5d5337166ad00196cb37ca555f58e116854707121e563dd503e10fb37c221029` | 31396 |
| 3 | `/tmp/bat-L3.log` | `20261024T204225Z-battery-L3.log EXIT=0` | `5f69751d4c360b8e9fa79948e2cea9017550019eaa6b12a5a4b8051f5383192d` | 31241 |
| 4 | `/tmp/bat-L4.log` | `20261024T204225Z-battery-L4.log EXIT=101` | `c2eedcb4d1f09d5c4a171bb5d02036ee9f4a7eee13c01ba6e6b5adc3bc025647` | 28052 |
| 5 | `/tmp/r1-rung1.log` | `20261024T204225Z-r1-rung1.log EXIT=0` | `efd5800647e8dd1133b2e0bf8027b1234b25cb5b29700222bdade8cde4b3ed9b` | 8270 |
| 6 | `/tmp/r1-rung2.log` | `20261024T204225Z-r1-rung2.log EXIT=0` | `70f0f19cc1672bbef4b3ed8ff4f7ad8a5c3f964582bae6e12b727b12df3485c2` | 501 |
| 7 | `var/mutation-court/mc-list-prefix-20261024T131028Z.txt` | `20261024T204225Z-mc-list-prefix-20261024T131028Z.txt` | `45b6e3eb834eebe6327f8b4598b1fbd9f443da992a98d9ef7af64f13192dfb12` | 6110 |
| 8 | `var/mutation-court/mc-list-postfix-20261024T131648Z.txt` | `20261024T204225Z-mc-list-postfix-20261024T131648Z.txt` | `a6eeb76cfd5100722fbc5913f4b37090ef37e53d1af99fd1f82417663ab70cff` | 15142 |
| 9 | `var/mutation-court/mc-case-capdispatch-20261024T131648Z.txt` | `20261024T204225Z-mc-case-capdispatch-20261024T131648Z.txt` | `e463c70377452318ca8bcd69181cf8edf806f92d21acf635c4f7cd68c70d99d0` | 323 |
| 10 | `var/mutation-court/mc-case-rb-20261024T131648Z.txt` | `20261024T204225Z-mc-case-rb-20261024T131648Z.txt` | `4433af0ae8a4f9e5dec92aaf35f42234319d7707dba988a7026ee5b776e7119e` | 1252 |

## Embedded output: copy + digest verification (real transcript)

Captured from the copy run (one `STAMP` line + ten `OK byte-exact` lines,
zero `DIGEST-MISMATCH` firings — the guard compares digest(source) to
digest(copy) and exits 1 on any mismatch):

```
STAMP=20261024T204225Z
OK byte-exact: /tmp/bat-L1.log -> receipts/logs/20261024T204225Z-battery-L1.log EXIT=0 (29071 bytes)
OK byte-exact: /tmp/bat-L2.log -> receipts/logs/20261024T204225Z-battery-L2.log EXIT=0 (31396 bytes)
OK byte-exact: /tmp/bat-L3.log -> receipts/logs/20261024T204225Z-battery-L3.log EXIT=0 (31241 bytes)
OK byte-exact: /tmp/bat-L4.log -> receipts/logs/20261024T204225Z-battery-L4.log EXIT=101 (28052 bytes)
OK byte-exact: /tmp/r1-rung1.log -> receipts/logs/20261024T204225Z-r1-rung1.log EXIT=0 (8270 bytes)
OK byte-exact: /tmp/r1-rung2.log -> receipts/logs/20261024T204225Z-r1-rung2.log EXIT=0 (501 bytes)
OK byte-exact: var/mutation-court/mc-list-prefix-20261024T131028Z.txt -> receipts/logs/20261024T204225Z-mc-list-prefix-20261024T131028Z.txt (6110 bytes)
OK byte-exact: var/mutation-court/mc-list-postfix-20261024T131648Z.txt -> receipts/logs/20261024T204225Z-mc-list-postfix-20261024T131648Z.txt (15142 bytes)
OK byte-exact: var/mutation-court/mc-case-capdispatch-20261024T131648Z.txt -> receipts/logs/20261024T204225Z-mc-case-capdispatch-20261024T131648Z.txt (323 bytes)
OK byte-exact: var/mutation-court/mc-case-rb-20261024T131648Z.txt -> receipts/logs/20261024T204225Z-mc-case-rb-20261024T131648Z.txt (1252 bytes)
```

Re-verification at receipt-write time (independent of the copy run):

```
$ b3sum receipts/logs/20261024T204225Z-*
7dec1d1f598a6f1861824f55927bec94e56a3f6785eb208559700bd0647c29b3  receipts/logs/20261024T204225Z-battery-L1.log EXIT=0
5d5337166ad00196cb37ca555f58e116854707121e563dd503e10fb37c221029  receipts/logs/20261024T204225Z-battery-L2.log EXIT=0
5f69751d4c360b8e9fa79948e2cea9017550019eaa6b12a5a4b8051f5383192d  receipts/logs/20261024T204225Z-battery-L3.log EXIT=0
c2eedcb4d1f09d5c4a171bb5d02036ee9f4a7eee13c01ba6e6b5adc3bc025647  receipts/logs/20261024T204225Z-battery-L4.log EXIT=101
e463c70377452318ca8bcd69181cf8edf806f92d21acf635c4f7cd68c70d99d0  receipts/logs/20261024T204225Z-mc-case-capdispatch-20261024T131648Z.txt
4433af0ae8a4f9e5dec92aaf35f42234319d7707dba988a7026ee5b776e7119e  receipts/logs/20261024T204225Z-mc-case-rb-20261024T131648Z.txt
a6eeb76cfd5100722fbc5913f4b37090ef37e53d1af99fd1f82417663ab70cff  receipts/logs/20261024T204225Z-mc-list-postfix-20261024T131648Z.txt
45b6e3eb834eebe6327f8b4598b1fbd9f443da992a98d9ef7af64f13192dfb12  receipts/logs/20261024T204225Z-mc-list-prefix-20261024T131028Z.txt
efd5800647e8dd1133b2e0bf8027b1234b25cb5b29700222bdade8cde4b3ed9b  receipts/logs/20261024T204225Z-r1-rung1.log EXIT=0
70f0f19cc1672bbef4b3ed8ff4f7ad8a5c3f964582bae6e12b727b12df3485c2  receipts/logs/20261024T204225Z-r1-rung2.log EXIT=0
```

All ten digests match the file map (replay: rerun the `b3sum` command above).

## Provenance

- The four mutation-inventory witnesses live canonically in
  `var/mutation-court/`; the retained copies were taken from there.
  `mc-list-prefix` / `mc-list-postfix` also have `/tmp` mirrors, verified
  digest-equal at retention time to the retained copies:
  - `/tmp/mc-list-prefix-20261024T131028Z.txt` →
    `45b6e3eb834eebe6327f8b4598b1fbd9f443da992a98d9ef7af64f13192dfb12` (equal)
  - `/tmp/mc-list-postfix-20261024T131648Z.txt` →
    `a6eeb76cfd5100722fbc5913f4b37090ef37e53d1af99fd1f82417663ab70cff` (equal)
  `mc-case-capdispatch` / `mc-case-rb` existed ONLY in `var/mutation-court/`.
- `/tmp/bat-L{1..4}.log` are the L1-L4 battery run logs (cargo battery
  transcripts, each opening with the workspace profile warning line);
  `/tmp/r1-rung1.log` is the first rung run, `/tmp/r1-rung2.log` opens
  `== LEG A (baseline admit) ==`.
- The zero-byte `/tmp/mc-list-postfix-20261024T131518Z.txt` (with its `.err`)
  is a failed first postfix listing attempt, superseded by the `131648Z`
  listing; not retained (zero-byte witness carries no inventory bits).

## E09 linkage

E09's battery receipts embed these logs' content verbatim and are
digest-equal: the b3sum values in the file map are the comparison keys for
that equality check. Retention here exists so the digests stay resolvable
after `/tmp` purges the originals.

## Verification: chain state (real output, pre-receipt-write run)

```
=== receipt_chain_sync.sh --check (after logs, before receipt write) ===
TAMPER-DETECTED: receipts/2026-10-24-npm-attempt-ledger.md (chain=b512b628eaf87edf1ef359e1e31bf89a429f4e609baaba2b14202f27e8d370f0 actual=42ff70b7bbdad6ad107abf5fc3edbb04aeff963a7ac7a8cdef887a3b2155e09f)
exit=1
=== chained entries vs on-disk files ===
chained=2205 on_disk=2297 unchained_delta=92
```

Reading:

- The `TAMPER-DETECTED` line is PRE-EXISTING and OUTSIDE this lane's lease:
  `receipts/2026-10-24-npm-attempt-ledger.md` (another lane's untracked
  receipt) was modified at 13:29:02 local, after the 13:28:13 CHAIN.blake3
  sync (`stat` mtimes; git shows it `??` untracked and `CHAIN.blake3` `M`
  from the wave's earlier syncs). Not repaired by this lane — receipts are
  immutable and the file is not leased here; disclosed to the coordinator
  for its sync decision.
- The 10 retained logs were unchained by construction until the coordinator's
  sync — expected and fine (`--check` verifies chain→disk only; new files
  are invisible to it). Disk-count delta 92 → 93 once this receipt lands
  (10 logs + this receipt + 81 other in-flight receipt files from the wave).

## Falsifier

A `b3sum` over any retained file that fails to reproduce its file-map digest,
or an E09 battery receipt whose embedded log content does not digest-match
the corresponding `receipts/logs/` file, falsifies this retention.

## Retro-tail addendum (2026-10-26)

The six 2026-10-24 battery/rung logs predate the EXIT-tail capture law. Each now
carries a `# RETRO-TAIL 2026-10-26` block deriving its exit from the log's own
verdict lines (appended after retention; pre-tail blake3 digests are embedded in
each log's retro-tail comment, so the retention-time digests in the tables above
refer to the pre-tail bytes — byte-exactness of the retention event is unchanged).
battery-L4 is honestly cited at EXIT=101: its log contains real failing result
lines; it was retained as the red leg of that battery, not a green one.
