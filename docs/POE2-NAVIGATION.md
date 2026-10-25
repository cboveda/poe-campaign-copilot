# Within-zone navigation

Navigation is a separate overlay section for landmarks, entrances, and
within-zone directions. It does not complete campaign objectives, claim
rewards, change the frontier, or infer your position inside a zone.
Client.txt provides zone transitions, not coordinates or landmark detection.
The normal overlay shows all cues for the current supported area; compact
mode continues to show the campaign objective and next zone.

## Initial content

Seven source cues cover six Act 1 areas for the 0.5.5 guide:

| Area | Navigation content |
| --- | --- |
| Clearfell | Mud Burrow waypoint and bridge toward Grelwood |
| The Grelwood | Roots toward Tree of Souls; upstream toward Red Vale |
| Cemetery of the Eternals | Optional Forgotten Riches cache and pillar switch |
| Hunting Grounds | Locust trail toward Freythorn; blood trails and signpost |
| Freythorn | Smoke toward smaller Ritual Altars |
| Ogham Village | Cobble road toward the smithing house |

These directions preserve the pinned MIT guide's claims; they have not been
independently checked in game. They are matched to the resolved current
campaign area, including revisits, and cleared on unsupported locations.
There are no diagrams, fixed map coordinates, or automatic cue completion.
The optional cache and optional-area directions remain navigation advice,
not required campaign objectives.

The source's Grim Tangle cue is deliberately omitted: its heading is Grim
Tangle but its text describes finding the Grim Tangle entrance. We need a
clearer source establishing the origin area before attaching that cue.
Areas without source-backed navigation have no Navigation section.

## Adding coverage

`content/poe2/navigation.json` is generated with:

```sh
python tools/import_poe2_navigation.py
python tools/import_poe2_navigation.py --check
```

The importer uses the existing pinned Act 1 HTML, with revision, normalized
UTF-8/LF checksum, source anchors, patch and license recorded in the pack.
It executes no source scripts and makes no network requests. Existing source
attribution is in CREDITS.md and vendor/poe2-leveling/VENDOR.md.

For more coverage, supply a guide URL, timestamped video, or authored notes
that establish the origin zone, landmark, destination and any layout
conditions. Check permission before copying third-party text or diagrams.
Avoid presenting one random layout's direction as universal. Unclear origin
zones or patch-dependent claims should remain recorded gaps until resolved.

## Optional manual check

No in-game validation is required to continue development. If checking this
feature later:

1. Enter Grelwood and confirm Navigation shows roots and the river, separately
   from Campaign and Optional objectives.
2. Return to town: the Navigation section should disappear. Return to
   Grelwood: its cues should reappear without advancing campaign progress.
3. Compare a cue against the current layout. Report the area, cue text, and
   observed mismatch; no fresh character or repeated boss kill is needed.
4. Toggle compact mode: navigation is hidden. Restore normal mode to read it.
