# PoE2 data

`observed-areas.json` records 12 campaign areas observed in the supplied
English log, matched to campaign chapters using the approved guide. It is
not a complete game database. IDs/names and example levels come from the
log; loading uses the generated instance level, not this example level.
Only observed towns are marked as towns. `has_waypoint: false` means not
established by this dataset, not an assertion that the game lacks one.
No connections, quests, rewards or waypoint ownership are inferred.

Approved route reference:
https://github.com/domistae/poe2-leveling at
916bd138286fe1307113ec044bcb4d565432b516 (MIT, copyright 2026 domistae).
Its claimed 0.5.5 patch compatibility still requires verification against
the live game. No guide text or diagrams are included in this observation
catalog. `act1-route.json` separately adapts 65 objective texts across 18
visits from the pinned Act 1 guide, generated with
`python tools/import_poe2_act1.py` (verify with `--check`). Its coverage is
explicitly Act 1 preview, and objectives are advisory. The JSON records
source revision, SHA-256 and claimed patch. The embedded route uses internal
`poe2-guide:` keys for names whose game IDs have not been observed; these
keys must never be represented as real game IDs. Full campaign coverage and
live-patch/gameplay verification are outstanding.
