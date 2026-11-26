# abi-bin-v2

Archived experimental fixed-header binary ABI v2 (magic `\x89GLB`, 64-byte header, BLAKE3
trailer). Echo-only, zero consumers; removed at the v26.11.26 delete criterion.

- `abi_bin.rs.txt` was `src/abi_bin.rs`.
- `abi_bin_court.rs.txt` was `tests/abi_bin_court.rs`.

Restore by renaming back, re-adding `pub mod abi_bin;` to `src/lib.rs`. Recover history from git.
