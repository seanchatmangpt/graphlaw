#!/usr/bin/env bash
# M11: self-test for the Exit-capture law (.claude/agents/court-runner.md).
# Runs `false | true` (tail exits 0, head exits 1) and proves the documented
# zsh pattern detects the real head exit. A gate that cannot exit 1 is the
# failure — the script exits 1 if the pattern FAILS to detect it.
set -u

fail=0

# 1. Baseline defect witness: $? after the pipe is tail's (0) — false green.
false | true
naive=$?
if [ "$naive" -eq 0 ]; then
  echo "WITNESS: \$? after 'false | true' is 0 (tail's exit) — the defect is real"
else
  echo "UNEXPECTED: \$? after pipe was $naive (expected 0 witness)"; fail=1
fi

# 2. Lawful pattern: set -o pipefail + ${pipestatus[1]} under zsh.
zout=$(zsh -c 'set -o pipefail; false | true; print ${pipestatus[1]}')
zrc=$?
echo "LAWFUL: zsh pipestatus[1] for 'false | true' = ${zout} (zsh rc=$zrc)"
if [ "$zout" = "1" ] && [ "$zrc" -eq 0 ]; then
  echo "PASS: exit-capture pattern detects head exit 1"
else
  echo "FAIL: pattern did not detect head exit 1 (got '${zout}')"; fail=1
fi

# 3. Positive control: pattern must NOT flag a healthy pipe.
zout2=$(zsh -c 'set -o pipefail; true | true; print ${pipestatus[1]}')
if [ "$zout2" = "0" ]; then
  echo "PASS: healthy 'true | true' reads 0 (no false positive)"
else
  echo "FAIL: healthy pipe misread as '${zout2}'"; fail=1
fi

if [ "$fail" -eq 0 ]; then
  echo "exit_capture_check: PASS"
  exit 0
else
  echo "exit_capture_check: FAIL"
  exit 1
fi
