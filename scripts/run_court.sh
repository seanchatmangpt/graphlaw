#!/usr/bin/env bash
# run_court.sh — capture law for court runs (P17 G1).
# The real exit code lands in the log tail (EXIT=<rc>), so a receipt citing
# this log can be audited mechanically by tests/receipt_integrity_court.rs.
# Lawful pattern made easier than the unlawful one:
#   bash scripts/run_court.sh <name> <command> [args...]
# Log: receipts/logs/<UTC timestamp>-<name>.log (colon-free, chain-attested:
# receipt_chain_sync.sh enumerates receipts/ recursively, so logs fold into
# CHAIN.blake3 like any other receipt).
set -u
NAME="${1:?usage: run_court.sh <name> <command> [args...]}"
shift
mkdir -p receipts/logs
LOG="receipts/logs/$(date -u +%Y-%m-%dT%H%M%SZ)-${NAME}.log"
"$@" >"$LOG" 2>&1
RC=$?
echo "EXIT=$RC" >>"$LOG"
echo "run_court: log=$LOG exit=$RC" >&2
exit "$RC"
