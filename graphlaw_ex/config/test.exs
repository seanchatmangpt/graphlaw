# SPDX-FileCopyrightText: 2026 Sean Chatman <https://github.com/seanchatmangpt/graphlaw/graphs/contributors>
#
# SPDX-License-Identifier: MIT
import Config

config :graphlaw_ex, :kill_switch_test_mode, true
# Existing rdfs/chicago example suites run lease-less; production default is :required.
config :graphlaw_ex, :lease_policy, :none_allowed
