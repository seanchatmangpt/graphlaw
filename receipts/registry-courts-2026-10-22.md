# registry_artifacts + registry_dispatch re-run receipt (lane L167, staleness-gate R4 remediation)

- Date: 2026-10-22 (local) / 2026-10-23T02:40:18Z
- Subject: /Users/sac/graphlaw @ main `615dbfaf94d407cffe174af02804d0dbfb3f8f1d`, working tree dirty (lane-isolated build `CARGO_TARGET_DIR=target-lane167`).
- Backs: `docs/jira/v26.10.22/RECEIPTS.md:175` and `:451`.
- Commands:

```
$ CARGO_TARGET_DIR=target-lane167 cargo test --test registry_artifacts
     Running tests/registry_artifacts.rs (target-lane167/debug/deps/registry_artifacts-39d34e674dbef1ae)
test result: FAILED. 17 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 18.41s
$ CARGO_TARGET_DIR=target-lane167 cargo test --test registry_dispatch
     Running tests/registry_dispatch.rs (target-lane167/debug/deps/registry_dispatch-08dad6c1d4e3e78c)
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
```

Standing: registry_dispatch PASS 14/0 (matches legacy 14/0). registry_artifacts
**FAILED 17/5** — the digest-drift failure mode recorded at :175 persists on the
current tree (surface edits postdate the pin; needs a registry regeneration,
not a hand patch). The failure is witnessed, not laundered.
