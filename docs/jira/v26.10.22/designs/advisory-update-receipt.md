# Advisory Update Receipt — Lane L103 (L63 residual)

Date: 2026-10-22 · Subject: /Users/sac/graphlaw @ main (615dbfa) · Target dir: target-lane103

## Baseline: `CARGO_TARGET_DIR=target-lane103 cargo deny check advisories`

FAILED, 8 errors (exit 1):

| RUSTSEC | crate | version | class |
|---|---|---|---|
| RUSTSEC-2026-0258 | h2 | 0.3.27 | vulnerability |
| RUSTSEC-2026-0346 | rustls | 0.23.45 | vulnerability |
| RUSTSEC-2026-0347 | rustls | 0.23.45 | vulnerability |
| RUSTSEC-2026-0348 | rustls | 0.23.45 | vulnerability |
| RUSTSEC-2026-0349 | rustls | 0.23.45 | vulnerability |
| RUSTSEC-2025-0134 | rustls-pemfile | 1.0.4 | unmaintained |
| RUSTSEC-2026-0382 | rustls-webpki | 0.103.15 | vulnerability |
| RUSTSEC-2024-0449 | unsafe-libyaml | 0.2.11 | unmaintained |

## Targeted updates performed

| command (each with CARGO_TARGET_DIR=target-lane103) | exit | result |
|---|---|---|
| `cargo update -p h2` | 0 | "Locking 0 packages" — h2 stays 0.3.27 (latest compatible); fix requires >=0.4.16 |
| `cargo update -p rustls` | 0 | rustls 0.23.45 -> 0.23.46 |
| `cargo update -p rustls-webpki` | 0 | rustls-webpki 0.103.15 -> 0.103.16 |
| `cargo update -p rustls-pemfile` | 0 | "Locking 0 packages" — stays 1.0.4 (latest 1.x) |

## Post-update gates

- `CARGO_TARGET_DIR=target-lane103 cargo check` — exit 0 (`Finished dev profile ... in 1m 08s`)
- `CARGO_TARGET_DIR=target-lane103 cargo test --test chicago_validation --test sql_rdf_dialect --test trust_attest --test registry_dispatch --test query_cache_court` — exit 0, 66 passed / 0 failed (9+8+14+25+10)
- `CARGO_TARGET_DIR=target-lane103 cargo deny check advisories` — exit 1, **3 errors remaining**

## Delta: 8 -> 3

CLOSED (patch-level, minimal update):
- RUSTSEC-2026-0346/0347/0348/0349 (rustls 0.23.45 -> 0.23.46)
- RUSTSEC-2026-0382 (rustls-webpki 0.103.15 -> 0.103.16)

REMAINING / REQUIREMENTS-CHANGE (not forced, per task rules):
- RUSTSEC-2026-0258 (h2): fix is `>=0.4.16`; in-tree h2 is 0.3.27 (latest 0.3.x). Closure requires
  a semver-incompatible 0.3.x -> 0.4.x migration — REQUIREMENTS-CHANGE, skipped, not forced.
- RUSTSEC-2025-0134 (rustls-pemfile 1.0.4): "No safe upgrade is available!" (unmaintained).
  Closure requires upstream replacement in the dependency providing it — REQUIREMENTS-CHANGE.
- RUSTSEC-2024-0449 (unsafe-libyaml 0.2.11): "No safe upgrade is available!" (unmaintained),
  via serde_yaml_ng 0.10.0 <- purrdf family. Closure requires purrdf/serde_yaml_ng upstream
  transition — REQUIREMENTS-CHANGE.

## Reverts

None. No update broke a gate; git untouched (`git checkout -- Cargo.lock` never used).
