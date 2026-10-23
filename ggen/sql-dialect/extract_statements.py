#!/usr/bin/env python3
"""Read DuckDB's PEG `Statement` rule and write the ggen ontology.

Source pin: frozen-duckdb 2.0.0-dev, duckdb commit b6013f7aa2.
"""

import os
from pathlib import Path

# DUCKDB_SRC is the unpacked frozen-duckdb tree (the directory that contains
# src/parser/peg/grammar). There is no default checkout path: a missing tree
# exits 1.
ROOT = Path(os.environ["DUCKDB_SRC"]) if os.environ.get("DUCKDB_SRC") else None
GRAM = (
    ROOT / "src/parser/peg/grammar/statements/common.gram" if ROOT else Path()
)
EXT_DIR = ROOT / "extension" if ROOT else Path()

# setup SQL, then a SELECT whose first column is an integer.
# Directories under extension/ that are not loadable extensions are omitted.
EXTENSIONS = {
    "autocomplete": ("LOAD autocomplete", "SELECT 1", 1),
    "core_functions": ("SELECT 1", "SELECT list_sum([1, 2, 3])", 6),
    "delta": ("LOAD delta", "SELECT 1", 1),
    "icu": ("SELECT 1", "SELECT CASE WHEN count(*) > 0 THEN 1 ELSE 0 END FROM icu_calendar_names()", 1),
    "json": ("SELECT 1", "SELECT json_extract('{\"a\": 1}', '$.a')::INTEGER", 1),
    "parquet": (
        "COPY (SELECT 1 AS id) TO '/tmp/graphlaw-sql-dialect.parquet' (FORMAT PARQUET)",
        "SELECT count(*) FROM read_parquet('/tmp/graphlaw-sql-dialect.parquet')",
        1,
    ),
    "tpcds": (
        "LOAD tpcds; CALL dsdgen(sf = 0)",
        "SELECT CASE WHEN count(*) > 0 THEN 1 ELSE 0 END FROM call_center",
        1,
    ),
    "tpch": ("CALL dbgen(sf = 0.01)", "SELECT count(*) FROM nation", 25),
}

# One statement the installed engine can parse, keyed by the PEG alternative name.
EXAMPLES = {
    "ExternalResourceStatement": "SHOW EXTERNAL RESOURCES",
    "ExtensionRepositoryStatement": "SET allow_extension_repositories='allowed'; CREATE OR REPLACE EXTENSION REPOSITORY repo WITH PREFIX '/tmp/graphlaw-ext-repo' USING PUBLIC KEY 'MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAt9+KeDkWJa7SiBb7AMdXAPV2qpRunztoC2c7DtRmN9+Lq2YQm4rMQmmAQcwyxTxcK8vwPKOiE4mLMwmQMvTty9tL+buREfvg5KJ+FY3Bv05cZYsf7VuYgUDcw41zSEE5W1thh5PiafcrMn7CPF0jN0oFfVvAHB/fAM0yaZGPKs3WYqnHAHa6v/cVMeiWZ/lQORCBqqL2hii1fEiuIqw6AOEoZVsN0/lqm7GfnD899zrzLOPRuP4YFg46drL1UyNdg+4TquJ2OV9kR4CAfdGRexqHFWLmoxkTfnUxJ4yL+gkV57K1H+smEdKrDhDX68pEFM/34E2eVKZoRU/0paxDZQIDAQAB'",
    "CreateStatement": "CREATE TABLE t2(id INTEGER)",
    "SelectStatement": "SELECT 1",
    "SetStatement": "SET threads = 1",
    "PragmaStatement": "PRAGMA version",
    "CallStatement": "CALL pragma_database_size()",
    "InsertStatement": "INSERT INTO t VALUES (2, 'b')",
    "DropStatement": "DROP TABLE IF EXISTS t2",
    "CopyStatement": "COPY t TO '/tmp/graphlaw-sql-dialect-t.csv' (FORMAT CSV)",
    "ExplainStatement": "EXPLAIN SELECT 1",
    "UpdateStatement": "UPDATE t SET name = 'c' WHERE id = 1",
    "PrepareStatement": "PREPARE q AS SELECT 1",
    "ExecuteStatement": "PREPARE q AS SELECT 1; EXECUTE q",
    "AlterStatement": "ALTER TABLE t ADD COLUMN extra INTEGER",
    "TransactionStatement": "BEGIN",
    "DeleteStatement": "DELETE FROM t WHERE id = 2",
    "AttachStatement": "ATTACH ':memory:' AS mem",
    "UseStatement": "ATTACH ':memory:' AS mem2; USE mem2",
    "DetachStatement": "ATTACH ':memory:' AS mem3; DETACH mem3",
    "CheckpointStatement": "CHECKPOINT",
    "VacuumStatement": "VACUUM",
    "ResetStatement": "RESET threads",
    "ExportStatement": "EXPORT DATABASE '/tmp/graphlaw-sql-dialect-export'",
    "ImportStatement": "ATTACH '/tmp/graphlaw-import-src.duckdb' AS src; USE src; CREATE OR REPLACE TABLE only_src(id INTEGER); EXPORT DATABASE '/tmp/graphlaw-sql-dialect-import'; USE memory; DETACH src; IMPORT DATABASE '/tmp/graphlaw-sql-dialect-import'",
    "CommentStatement": "COMMENT ON TABLE main.t IS 'graphlaw'",
    "DeallocateStatement": "PREPARE q AS SELECT 1; DEALLOCATE q",
    "TruncateStatement": "TRUNCATE main.t",
    "LoadStatement": "LOAD icu",
    "InstallStatement": "INSTALL tpch FROM '/Users/sac/.duckdb/extensions'",
    "UpdateExtensionsStatement": "UPDATE EXTENSIONS",
    "AnalyzeStatement": "ANALYZE main.t",
    "MergeIntoStatement": "MERGE INTO main.t USING (SELECT 1 AS id, 'z' AS name) s ON main.t.id = s.id WHEN MATCHED THEN UPDATE SET name = s.name",
    "ConnectStatement": "CONNECT LOCAL",
    "DisconnectStatement": "DISCONNECT",
}


def statement_names(text: str) -> list[str]:
    start = text.index("Statement <-")
    rest = text[start:].split("\n", 1)[1]
    names = []
    for line in rest.splitlines():
        stripped = line.strip()
        if not stripped:
            if names:
                break
            continue
        if stripped.startswith("#"):
            continue
        token = stripped.strip("/").strip()
        if not token.endswith("Statement") and token not in names:
            # alternatives are *Statement names; a non-name ends the rule
            if names and not token.endswith("Statement"):
                break
        if token.endswith("Statement"):
            names.append(token)
        elif names:
            break
    return names


def main() -> None:
    if ROOT is None or not GRAM.is_file():
        raise SystemExit(
            "DUCKDB_SRC is unset or has no PEG grammar at "
            "src/parser/peg/grammar/statements/common.gram; "
            "the committed contract is ggen/sql-dialect/ontology/statements.ttl"
        )
    names = statement_names(GRAM.read_text())
    missing = [n for n in names if n not in EXAMPLES]
    extra = [n for n in EXAMPLES if n not in names]
    if missing or extra:
        raise SystemExit(f"example map drifted: missing={missing} extra={extra}")
    out = Path(__file__).resolve().parent / "ontology" / "statements.ttl"
    lines = [
        "@prefix sql: <http://graphlaw.org/sql#> .",
        "@prefix dcterms: <http://purl.org/dc/terms/> .",
        "",
        "sql: a dcterms:source .",
        'sql:grammarSource dcterms:source "src/parser/peg/grammar/statements/common.gram" .',
        "",
    ]
    for i, name in enumerate(names, start=1):
        example = EXAMPLES[name].replace("\\", "\\\\").replace('"', '\\"')
        lines.append(f"sql:{name} a sql:StatementKind ;")
        lines.append(f'    sql:name "{name}" ;')
        lines.append(f'    sql:example "{example}" ;')
        lines.append(f"    sql:order {i} .")
        lines.append("")
    skip = {"demo_capi", "loader"}
    dirs = sorted(p.name for p in EXT_DIR.iterdir() if p.is_dir() and p.name not in skip)
    missing_ext = [n for n in dirs if n not in EXTENSIONS]
    extra_ext = [n for n in EXTENSIONS if n not in dirs]
    if missing_ext or extra_ext:
        raise SystemExit(f"extension map drifted: missing={missing_ext} extra={extra_ext}")
    lines.append('sql:extensionRoot dcterms:source "extension" .')
    lines.append("")
    for i, name in enumerate(dirs, start=1):
        setup, check, expect = EXTENSIONS[name]
        setup = setup.replace("\\", "\\\\").replace('"', '\\"')
        check = check.replace("\\", "\\\\").replace('"', '\\"')
        lines.append(f"sql:ext_{name} a sql:Extension ;")
        lines.append(f'    sql:name "{name}" ;')
        lines.append(f'    sql:setup "{setup}" ;')
        lines.append(f'    sql:check "{check}" ;')
        lines.append(f"    sql:expect {expect} ;")
        lines.append(f"    sql:order {i} .")
        lines.append("")
    out.write_text("\n".join(lines))
    print(f"wrote {len(names)} statements and {len(dirs)} extensions to {out}")


if __name__ == "__main__":
    main()
