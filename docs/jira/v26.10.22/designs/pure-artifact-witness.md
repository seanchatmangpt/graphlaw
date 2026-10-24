# PURE Artifact Frame Witness (Lane N15)

Date: 2026-10-22. Lane: N15, `/Users/sac/graphlaw`, read-only on both repos and
`/Users/sac/beam4pm`. Purpose: fresh packet-4 frame witness for the PURE-tier
candidate artifact, feeding D3 of the dual-artifact policy
(`designs/dual-artifact-policy-design.md` §6 D3). No rebuilds, no pin changes.

## 1. Artifact identity

`/Users/sac/beam4pm/native/graphlaw/graphlaw_wasm.wasm`

| field | value |
|---|---|
| byte size | 6,655,651 B |
| BLAKE3 | `8b4f4bc8ef650a85135da4d7b6a13b94be20bd38143a21049c03c274c2bb43b5` | (historical)
| sha256 | `bf2bdd8fd761ff85d876a4af8805d77606d08ca1788e9a89be26039f2150d66c` | (historical)
| mtime | Oct 13 07:15 |

**The BLAKE3 matches the D3 candidate `8b4f4bc8…` exactly.** It does NOT match
the legacy registry pin `332ea218…2c30c` (that pin is 6,793,617 B — a different
byte stream of the same generation, never frame-witnessed; see
`designs/smalt-pairing-receipt.md` finding 2). The two digests differ honestly:
`332ea218…` is the 26.10.21 release artifact; `8b4f4bc8…` is the pre-duckdb
generation whose bytes beam4pm carries.

## 2. Runner

`graphlaw-run` built fresh in this lane (no shared-target contention):

```
CARGO_TARGET_DIR=target-lane-n15 cargo build --features runner --bin graphlaw-run
→ Finished `dev` profile in 1m 41s (exit 0)
```

Binary: `/Users/sac/graphlaw/target-lane-n15/debug/graphlaw-run`.

## 3. Frames witnessed

Each run: fresh `graphlaw-run --wasm <artifact>` process, packet-4 frames on
stdin, replies quoted verbatim (sort_keys JSON), stderr quoted verbatim.

### 3.1 capabilities — exit 0

Request frame: `{"op":"capabilities"}` (21 B)

```json
{"ok": true, "result": {"abi": 1, "abi_version": 1,
  "authorities": [
    {"authority": "purrdf", "capability": "RDF 1.2 / codecs / storage IR", "revision": "2.0.2"},
    {"authority": "purrdf::sparql", "capability": "SPARQL 1.1/1.2", "revision": "2.0.2"},
    {"authority": "purrdf::shapes", "capability": "SHACL", "revision": "2.0.2"},
    {"authority": "purrdf::shex", "capability": "ShEx 2.1", "revision": "2.0.2"},
    {"authority": "purrdf::datalog", "capability": "Datalog", "revision": "2.0.2"},
    {"authority": "purrdf::entail", "capability": "RDF/RDFS/OWL-RL entailment", "revision": "2.0.2"},
    {"authority": "purrdf::sparql", "capability": "Knowledge hooks (kh: orchestration over SPARQL)", "revision": "2.0.2"},
    {"authority": "eyeron", "capability": "Notation3", "revision": "0.7.7 (eyereasoner/eyeron@d6568f65…, vendored)"}],
  "crate": "26.10.13",
  "ops": ["capabilities","sniff","parse","convert","canonical","sparql","shacl","shex","n3","entail","datalog","hooks","law","policy"],
  "rdf_dialects": ["turtle","trig","ntriples","nquads","rdfxml","jsonld","yamlld","trix","hextuples"],
  "other_dialects": ["n3","sparql","shexc","shexj"],
  "registry_schema": "graphlaw.capability-registry/1",
  "registry_sha256": "sha256:1c129dd6…",
  "surface_sha256": "sha256:fcee74ee…"}}
```

(Note: two authority rows above are lightly elided for line length; the frame
on the wire was complete and unmodified. `abi=1`. No `duckdb` op is listed —
the ops list ends at `policy`.)

### 3.2 parse (real Turtle) — exit 0

Request: `{"op":"parse","text":"<http://example/s> <http://example/p> <http://example/o> .","dialect":"turtle"}`

```json
{"ok": true, "result": {"dialect": "Turtle", "id": "sha256:f033b5c657944fc3d258e3698a2ba6c7d149b4213449b2bdf6977ce584e81775", "ok": true, "quads": 1}} # historical
```

### 3.3 governance op (`law` plan step, real admission + receipt) — exit 0

Request: `op=law`, data = the Turtle triple above, one plan step
(`a1`: empty N-Triples `pre`, `add` = `<http://example/g> <http://example/says> <http://example/ok> .`,
same goal).

```json
{"ok": true, "result": {"nquads": "<http://example/g> <http://example/says> <http://example/ok> .\n<http://example/s> <http://example/p> <http://example/o> .\n",
  "ok": true,
  "receipts": [{"added": 1, "authority": "purrdf", "child": "sha256:eebbbd4d5f97d86f59ddad3e7e1dc3a192a5d7b966309295071a86c153ffc7a1", # historical
                "index": 0, "parent": "sha256:f033b5c657944fc3d258e3698a2ba6c7d149b4213449b2bdf6977ce584e81775", # historical
                "plan_sha256": "sha256:8602b5163a07170a4dca3028d6ab0d9828bc76b9ec9f77fcbe4b9cd3a1bd431f", # historical
                "revision": "2.0.2", "step": "plan-action"}],
  "states": ["sha256:f033b5c657944fc3d258e3698a2ba6c7d149b4213449b2bdf6977ce584e81775", # historical
             "sha256:eebbbd4d5f97d86f59ddad3e7e1dc3a192a5d7b966309295071a86c153ffc7a1"]}} # historical
```

A real plan admission ran inside the guest: state id advanced, one receipt
minted (`plan-action`, authority purrdf 2.0.2).

### 3.4 duckdb-mode envelope — tier-defining refusal — exit 0 (typed refusal)

Request: `{"op":"duckdb","sql":"SELECT 1"}`

```json
{"ok": true, "result": {"ok": false, "error": {"dialect": null, "engine": null, "kind": "Unsupported", "message": "unknown op `duckdb`"}}}
```

The pure module has NO `duckdb` op at all — the op dispatch refuses typed
(`Unsupported`, "unknown op `duckdb`"), no trap, no crash, exit 0. This is the
expected PURE-tier behavior: the SQL surface is absent, not faked. (This is a
pre-OP15 generation: it predates the `duckdb` op entirely; it is not
REFUSAL-04 — REFUSAL-04 is the non-document-backed builtin path, a different
mechanism in the duckdb-linked generation.)

### 3.5 digest-mismatch negative — exit 84

`--expect-blake3 000…000`, one capabilities frame supplied:

```
stderr: graphlaw-run: ARTIFACT DIGEST MISMATCH for /Users/sac/beam4pm/native/graphlaw/graphlaw_wasm.wasm: expected 0000000000000000000000000000000000000000000000000000000000000000, got 8b4f4bc8ef650a85135da4d7b6a13b94be20bd38143a21049c03c274c2bb43b5 # historical
exit: 84
```

Note the runner independently computed the same BLAKE3 over the loaded bytes —
a second in-band confirmation of the artifact digest.

### 3.6 EOF-mid-frame negative — exit 90

Sent a 4-byte BE length prefix `00 00 00 0A` followed by only 3 bytes `{"o`,
then stdin EOF.

```
stderr: graphlaw-run: EOF mid-frame
exit: 90
```

### 3.7 digest-pin positive control — exit 0

```
printf '\x00\x00\x00\x15{"op":"capabilities"}' | graphlaw-run --wasm <artifact> \
  --expect-blake3 8b4f4bc8ef650a85135da4d7b6a13b94be20bd38143a21049c03c274c2bb43b5 # historical
exit: 0
```

Pin `8b4f4bc8…` admits and serves a real frame.

## 4. D3 recommendation inputs

- **The beam4pm artifact IS the `8b4f4bc8…` bytes** (BLAKE3 exact match;
  6,655,651 B). The fresh frame witness required by D3 now exists against
  exactly those bytes: capabilities, real-Turtle parse, real `law` plan
  admission with a minted receipt, typed duckdb refusal, and both negative
  fault modes (84/90) all pass.
- **`332ea218…` (6,793,617 B) is a different byte stream** — verified not
  equal to the artifact on disk. It was never frame-witnessed. The witness
  therefore supports D3's default: pin `8b4f4bc8…` as `wasm-pure`
  (pin-source=beam4pm@<sha>, recipe-external), keep `332ea218…` as the
  26.10.21 release pin / historical note, and do NOT bind wasmi courts to
  `332ea218…`.
- Runner independently recomputes the digest on the loaded bytes (exit-84
  diagnostic shows `got 8b4f4bc8…`), so a future court banner can assert
  tier + digest through the same `--expect-blake3` path.

## 5. Verification receipt

All commands run in this session, actual exits:

| run | exit |
|---|---|
| `cargo build --features runner --bin graphlaw-run` (CARGO_TARGET_DIR=target-lane-n15) | 0 |
| capabilities frame | 0 |
| parse Turtle frame | 0 |
| law plan frame | 0 |
| duckdb envelope (typed refusal) | 0 |
| digest mismatch negative | 84 |
| EOF-mid-frame negative | 90 |
| digest-pin positive (`8b4f4bc8…`) | 0 |
