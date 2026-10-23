#!/usr/bin/env python3
"""SessionStart card. Stdout is the lithography the next session reads.

Does not call the hooks ABI, does not write a receipt, and does not treat
the partition-court ratio as a pass.
"""
import sys

CARD = """\
graphlaw lithography (this session):
- Partitions: a pinned axiom is re-pinned only with a receipt; a generated projection is regenerated, never patched; a leaf edit names one owned path and one court.
- Every dispatch carries the absolute repo path `/Users/sac/graphlaw` and a disjoint file lease.
- Refusal codes that stop the tool: GENERIC_AGENT_REFUSED, REFUSED(topology), MANIFEST-REFUSED, DISK-REFUSED, LITHO-REFUSED(generated|registry-artifact|pin-file|staging|unreadable).
- Cheapest court rung: cargo check, then one court, then the suite. A loud SKIP is not a pass.
- A deny is not a receipt. Do not rephrase a refused tool into a second attempt.
- court-runner stays read-only.
"""


def main() -> None:
    sys.stdout.write(CARD)


if __name__ == "__main__":
    main()
