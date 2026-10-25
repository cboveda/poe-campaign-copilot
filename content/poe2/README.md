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

Target client patch **0.5.5** is user-confirmed. This matches the guide's
claimed patch but is not a substitute for gameplay validation. The generator
also exports `side_areas`: objective text from source steps 9, 33–35 for Mud
Burrow, Mausoleum of the Praetor and Tomb of the Consort. These do not add
main-route visits, advance progress or confirm rewards. A parent-zone label
indicates where the main guidance resumes, not an asserted portal mechanic
or guaranteed exit location. Mud Burrow's two supported display-name aliases
still need a real game-log capture; its game ID is deliberately unspecified.

`remaining-campaign.json` adds the pinned Act 2/3 route visits, local Act 4
island guidance, named side areas, and all three Interlude outlines. Generate
and check both route files with `python tools/import_poe2_campaign.py` and
`--check`. Known names/aliases are matched only within a candidate campaign
namespace; the raw observed ID is preserved. Guide-generated keys are never
claimed as actual game IDs. Source chapter numbers come from guide metadata;
a raw ID's number is only an ambiguity hint, allowing Interlude content to
use a different internal number. Ambiguous matches fail closed.

Act 4/Interlude guidance does not move the sequential frontier or infer
other branches' completion. For chapter-wide Interlude objectives whose
precise area is not established by the source, separate chapter reminders
retain the source text without claiming a boss-arena ID. All mappings need
Windows/gameplay validation. See docs/POE2-CAMPAIGN-COVERAGE.md.

`navigation.json` contains twenty landmark/direction cues for thirteen Act 1
areas, combining pinned MIT content with reviewed Mobalytics summaries.
The Mobalytics source warns of 0.5 layout changes; it is not certified for
0.5.5. Curated corrections live in `navigation-review.json`. It is independent of campaign objectives and progress.
Generate/check with `python tools/import_poe2_navigation.py [--check]`.
See docs/POE2-NAVIGATION.md for coverage, source gaps and extension guidance.
