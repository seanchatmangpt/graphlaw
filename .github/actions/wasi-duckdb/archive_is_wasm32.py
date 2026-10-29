#!/usr/bin/env python3
"""archive_is_wasm32 — fail-closed ar + wasm32 magic admission.

Exact port of frozen-duckdb-builder archive_is_wasm32, used by the composite
WASI action for cache admission and built-archive admission: the file must be
an ar archive (magic `!<arch>\n`) whose first 64 MiB contains the `\0asm` wasm
magic and none of the Mach-O/ELF magics (a host archive or binary is the
historical false-pass). Exit 0 = admit; exit 1 = refuse.
"""
import sys

b = open(sys.argv[1], "rb").read(64 * 1024 * 1024)
ok = (b.startswith(b"!<arch>\n") and b"\0asm" in b
      and not any(m in b for m in (b"\xcf\xfa\xed\xfe", b"\xfe\xed\xfa\xcf", b"\xca\xfe\xba\xbe", b"\x7fELF")))
sys.exit(0 if ok else 1)
