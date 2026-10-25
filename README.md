# PoE2 Campaign Copilot

A passive Windows overlay for the English Path of Exile 2 client. It reads
`Client.txt`; it does not read game memory or automate inputs.

**0.2.0-alpha.4 — campaign guide preview; Windows gameplay validation pending.**

The preview tracks completed zone loads and instance revisits on the English
**0.5.5** client. Acts 1–3 use sequential source-guided route visits; Act 4
islands and Interludes 5.1–5.3 use area guidance so choosing a different
order cannot complete unrelated branches. Campaign and Optional objectives
are separate; compact mode prioritizes campaign guidance.

Content is adapted from a pinned, licensed
[community guide](https://domistae.github.io/poe2-leveling/poe2_leveling_guide.html).
Objectives/rewards/directions are advisory and still need in-game review.
Entering an area does not confirm a boss kill, quest hand-in or reward.
Unknown/endgame areas do not select campaign guidance. No endgame Atlas/map
guide or diagrams are included. Mobalytics campaign build links can be
imported as saved snapshots with manual target checklists and act-based
stages; PoE2 PoB import remains deferred. PoE1 state is
separate; automatic updates are disabled. No installer is published here.

Start with [Run on Windows](docs/RUN-ON-WINDOWS.md). Existing testers can
stay on `codex/poe2-preview-alpha2`, stop the app, run `git pull --ff-only`,
then `npm.cmd run tauri dev`.

See [chapter coverage and validation](docs/POE2-CAMPAIGN-COVERAGE.md),
[development](docs/DEVELOPMENT.md), the
[conversion plan](docs/POE2-CONVERSION-PLAN.md), and the
[Windows checklist](docs/POE2-WINDOWS-VALIDATION.md). The earlier
[security update](docs/SECURITY-UPDATE-alpha2.md) remains applicable;
remaining audit findings have not been silently marked fixed.

## Using the preview

1. Build from this branch using the Windows instructions.
2. Open tray → Settings and browse to the PoE2 installation's `logs/Client.txt`.
3. Use windowed fullscreen and enter another area after startup; normal use
   skips existing game-log history.

Settings retain opacity, hotkeys and timer controls. The default hotkeys are
Alt+Shift+S (move/resize setup), Z (zoom), C (compact), H (hide), O (settings)
and T (timer). Use Reset progress when intentionally starting a new run. Planned returns to
a revisited town preserve progress until you confirm the previous zone
objectives with **Continue town step** on the overlay (or in Settings);
early shopping trips do not advance the route.
Login/process changes start a fresh route session to avoid mixing characters;
per-character resume across game sessions is not implemented yet.

Import a build from Settings → **Build guide** → **Preview import**, then
**Save imported build**. Use a named character profile and check targets
manually. See [Mobalytics build import](docs/MOBALYTICS-BUILD-IMPORT.md).

## Credits

Code is MIT. See [CREDITS.md](CREDITS.md) and
[vendor/poe2-leveling/LICENSE](vendor/poe2-leveling/LICENSE) for route
attribution. This is an unofficial fan project, unaffiliated with Grinding
Gear Games.
