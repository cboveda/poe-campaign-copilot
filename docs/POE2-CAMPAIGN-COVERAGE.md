# Campaign coverage — 0.2.0-alpha.3 / target patch 0.5.5

Configured source coverage below is **not gameplay certification**. The user
confirmed the target patch and tested the early-town guard/overlay button.
Remaining-act guidance has automated synthetic replay coverage, not captured
full-playthrough evidence. No gameplay is automated.

| Chapter | Imported content | Behavior |
| --- | --- | --- |
| Act 1 | 18 visits, 65 objectives; 3 supplemental side areas | Existing route and explicit repeated-town confirmation retained. |
| Act 2 | 25 visits, 78 objectives; Venom Pit/Galleria supplements | Sequential guide, including towns and permanent-bonus detours. |
| Act 3 | 25 visits, 79 objectives | Sequential guide, including temple/trial aliases and return visits. Two explicit league-mechanic objectives excluded. |
| Act 4 | 19 source headings merged by area; 10 named supplements | Local island/area guidance; selecting another island never completes a branch. |
| Interlude 5.1 | Refuge, Farmlands, Stones of Serle, Blackwood, Holten, Wolvenhold, Holten Estate | Location-based guidance, independent of other Interludes. |
| Interlude 5.2 | Khari Bazaar/Crossing, Pools of Kathal, Sel Khari Sanctuary, Molten Shrine/Skullmaw Stairway, Qimah/Reservoir | Location guidance plus chapter-wide final-objective reminders where arena attribution is unclear. |
| Interlude 5.3 | Glade, Ashen Forest, Kriar Village, Glacial Tarn, Howling Caves, Kriar Peaks, Etched Ravine, Cuachic Vault | Location guidance plus the Hilda turn-in reminder; order-independent. |
| Campaign hand-in | Source's conditional +2-point hand-in in Kingsmarch after all three Interludes | Advisory condition only; neither log travel nor entering Kingsmarch confirms it. |
| Endgame | Not included | Maps/hideouts remain outside campaign guidance; source steps 34–36 excluded. |

## Provenance and limits

Source: MIT-licensed domistae/poe2-leveling revision
`916bd138286fe1307113ec044bcb4d565432b516`. HTML is vendored unmodified;
only text is extracted, never scripts. JSON records per-file normalized
UTF-8/LF checksums. `python tools/import_poe2_campaign.py --check` checks
reproducibility and coverage. Act 3 source steps 28/81 are explicit league
mechanics and are omitted; step counts are actual pinned content rather than
older README estimates.

The original real log covers 47 area IDs but only 12 campaign IDs. Most new
areas therefore match known guide names/aliases after generation, scene and
successful loading evidence. `poe2-guide:` keys are internal; actual raw IDs
remain visible in the model/CLI. No game IDs were invented for new areas.
Candidates use a numeric `G..._` namespace and exact normalized names (leading
The/apostrophe style and explicit source aliases); endgame Map/Hideout
names do not select campaign content. Ambiguous names remain unmapped.

Act 4's “Fragment?” labels and other uncertainties in the original source
remain uncertain. Some Interlude text covers several locations: contextual
mapping is provisional and chapter-wide reminders deliberately avoid
assigning the final Faridun boss or Hilda turn-in to an unsupported arena.
Source navigation cues/rewards and exact live loading names need review.
Do not treat configured guide coverage as proof every live subzone is mapped.

The existing per-login session reset and lack of individual quest-completion
tracking remain. Act 4/Interlude branches are not automatically checked off;
there is no automatic campaign-complete claim. These limits preserve the
passive Client.txt-only design.

## User-executed checks (existing characters are fine)

Stop the app, `git pull --ff-only`, then `npm.cmd run tauri dev`. Confirm
startup says 0.2.0-alpha.3. Do not reset your game log or create a new run
merely to test guidance. Mark inaccessible content **unrun**.

1. **Act 2:** visit Vastiri Outskirts, Caravan, and an accessible field
   (Mawdun Mine is a useful case). Expect Act 2 objectives (Rathbreaker,
   caravan NPCs, Rudja as applicable), separate Campaign/Optional groups,
   and no Act 1 text. An early planned return to Caravan must preserve
   field progress; only confirm after legitimately finishing its objectives.
2. **Act 3:** visit Ziggurat Encampment and a field such as Jungle Ruins or
   Black Chambers. Expect Act 3 NPC/boss guidance for the actual location.
   A name/prefix difference such as “The ...” must not show another area.
   Revisit an earlier zone and confirm its own instructions return.
3. **Act 4:** visit two accessible islands in a different order than the
   source guide (for example Kedge Bay then Whakapanu). Expect each area's
   own map/boss objectives, with no “other branch completed” implication.
   Kingsmarch includes the final Interlude hand-in only as an “after all
   three” condition; it must not announce completion on arrival.
4. **Interludes:** visit one accessible area per Interlude, in any order
   (for example Kriar Village, Wolvenhold, then Qimah). Expect Interlude
   5.3/5.1/5.2 labels and corresponding objectives. Chapter reminders may
   refer to another area; they are not claims about your current arena.
5. **Endgame boundary:** if available, enter a hideout/map. Expect actual
   location with campaign guidance unavailable and no false completion.
6. **Persistence:** restart only the overlay in a mapped new area; expect
   restored location/guidance and correct updates after the next load.

For content correctness, check rewards/quest order during normal play—no
boss re-kills or item use are required just for this smoke check. Report:
app commit/version, patch, chapter/zone, expected versus actual display and
approximate timestamp. For an unmapped area, provide only nearby sanitized
generation/scene/loading lines. Remove chat/names/addresses; preserve order,
timestamps, IDs, names and seeds. These results remain **pending** until
reported; no Windows execution is claimed by the cloud checks.

Automated checkpoint: **252 Rust tests**, **137 frontend tests**, formatting,
Clippy with warnings denied, frontend build and Linux desktop compilation
passed. The importer reproduced identical JSON/checksums from LF and CRLF
copies of all pinned HTML. These checks do not replace Windows/gameplay tests.
