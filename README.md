# PoE2 Campaign Copilot

A passive Windows overlay for the English Path of Exile 2 client. It reads
`Client.txt`; it does not read game memory or automate inputs.

**0.2.0-alpha.2 — development preview, not a complete campaign release.**

The preview tracks completed zone loads, level-up/death events and instance
revisits. It displays Act 1 objectives adapted from a pinned, licensed
[community guide](https://domistae.github.io/poe2-leveling/poe2_act1_guide.html).
Objectives are suggestions, not proof that a boss, quest or reward is done.
The guide's patch compatibility and rewards still need gameplay review.

Later campaign chapters, diagrams, build imports/reminders and endgame
guidance are not provided. Unknown/endgame locations show their observed
name without moving the campaign route. PoE1 settings and progress use a
separate app namespace and are not imported. Automatic updates are disabled
until a PoE2 release channel is configured. No Windows release is published
by these changes.

See the [alpha.2 security update](docs/SECURITY-UPDATE-alpha2.md).

Start with [Run on Windows](docs/RUN-ON-WINDOWS.md). Also see
[development](docs/DEVELOPMENT.md), the
[conversion plan](docs/POE2-CONVERSION-PLAN.md), and the
[Windows validation checklist](docs/POE2-WINDOWS-VALIDATION.md).

## Using the preview

1. Build from the current preview source as described in the development guide.
2. Open tray → Settings and browse to the PoE2 installation's `logs/Client.txt`.
3. Run the game in windowed fullscreen and enter another area. Existing log
   history is skipped during normal use.

Settings retain opacity, hotkeys and timer controls. The default hotkeys are
Alt+Shift+S (move/resize setup), Z (zoom), C (compact), H (hide), O (settings)
and T (timer). Use Reset progress when intentionally starting a new run. Planned returns to
a revisited town preserve progress until you confirm the previous zone
objectives in Settings; early shopping trips do not advance the route.
Login/process changes start a fresh route session to avoid mixing characters;
per-character resume across game sessions is not implemented yet.

## Credits

Code is MIT. See [CREDITS.md](CREDITS.md) and
[vendor/poe2-leveling/LICENSE](vendor/poe2-leveling/LICENSE) for route
attribution. This is an unofficial fan project, unaffiliated with Grinding
Gear Games.
