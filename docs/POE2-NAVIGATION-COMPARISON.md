# Act 1 layout comparison

Compared on 2026-10-25 UTC with [Lolcohol's Mobalytics Act 1 guide](https://mobalytics.gg/poe-2/guides/campaign-layout-act-1),
whose visible update date is May 23, 2026. Reviewed the article, its layout
legend, and sixteen layout diagrams (both Clearfell seeds and all three
Manor floors included). The source credits Campaign Codex, Haddoq,
Guitaraholic and other contributors.

**Patch limitation:** the article explicitly warns that 0.5 reworks campaign
areas and that its layouts need updating after launch. It does not certify
0.5.5. Its confidence ratings describe the author's sample, not our gameplay
validation. Seed rotation, mirrored points of interest and swapped entrances
make fixed compass instructions unreliable. Where used, north/south/east/west
follow the guide's screen convention: top/bottom/right/left. The summaries below preserve
patterns and fallback searches rather than asserting universal directions.

## Result

| Area | Comparison and change | Source confidence |
| --- | --- | --- |
| Clearfell | Existing Mud Burrow/waypoint clue retained. Add north-side Beira search and interchangeable sides for Campsite/Mud Burrow; source notes newer layouts. | Very high, with new-layout caveat |
| Grelwood | Roots and river corroborated. Remove the extra upstream requirement. Add mushrooms leading to Grim Tangle here, resolving the other source's ambiguous heading. Add diamond search pattern as a heuristic. | High |
| Red Vale | Add broad loop for three altars; no fixed altar sequence. | Low |
| Grim Tangle | Add opposite-side/opening search pattern; do not show Grelwood's mushroom clue inside this zone. | Average |
| Cemetery | Add tentative side-wall search for tombs. Existing optional cache clue remains from domistae and is not corroborated by this guide. Do not promote the source's small-sample percentage to a guarantee. | Low |
| Mausoleum | Add central wheel-room alignment and alternate long-wall search for Draven; ribbed arena edge as an identifying clue. | High |
| Tomb of the Consort | Add qualified upper-area search with central-arena exception. | Low |
| Hunting Grounds | Signposts, bloody tracks and locusts corroborated. Add edge/corner search and narrow Crowbell approach. Ritual is conditional, not a required detour. | High |
| Freythorn | Replace guaranteed smoke navigation. Newer locust advice follows completed rituals; older boss-arena smoke/swirl advice is marked potentially obsolete by the guide. | Low |
| Farmlands | Add hut/cart landmark for Una's Lute and opposite-boundary search for Village, allowing rotation. | Very high |
| Village | Replace uncorroborated cobble-road certainty with boundary/checkpoint search for boss and Tools. | Very high |
| Ramparts | Add U-shaped mirrored pattern; recover from an early dead end rather than force a compass direction. | Very high |
| Manor | Add explicitly floor-labelled bridge/stair/elevator hints with entrance-swap and loop caveats. All floors share the existing area match; no floor detection is claimed. | High; floor 2 high/medium |

The pack now contains 20 cues over 13 distinct areas. There is no new map
image distribution: the Mobalytics diagrams were inspected locally only.
The new summaries are independently worded factual directions with source
headings and confidence retained in `content/poe2/navigation-review.json`.
The fetched HTML checksum records which article response was reviewed; its
full text and image files are not vendored.

## Data and display

`navigation-review.json` stores the curated comparison source, replacement
areas and new cues. `tools/import_poe2_navigation.py` combines it with the
pinned MIT source to generate `navigation.json`; `--check` verifies the
combined result offline. Regenerating the pack preserves the reviewed
corrections. Neither source HTML nor article instructions are executed.

Attribution in the overlay comes from the currently displayed cues; it is
no longer hardcoded to domistae. Mobalytics cues also show the 0.5/0.5.5
patch caveat. No navigation cue changes campaign progress or confirms a
reward. Known Act 1 side-area names use the existing side-area resolver,
without inventing game IDs or accepting matching endgame names.

## Optional user check

In-game checking remains deferred at the user's request. When convenient:

1. In Grelwood, check that Navigation mentions orange mushrooms toward Grim
   Tangle. Inside Grim Tangle it should switch to the opposite-side pattern.
2. In Freythorn, check for the next ritual's locust trail after a ritual;
   report any mismatch with the displayed text and current patch.
3. Enter either tomb and confirm its own navigation replaces Cemetery's
   hints. Return to town and confirm all navigation disappears.
4. In Manor, read only the hint labelled for your current floor. The app
   cannot determine that floor from the current transition data.

No new character or repeated boss kill is needed for the display checks.
