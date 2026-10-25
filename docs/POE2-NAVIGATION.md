# Within-zone navigation

Navigation is a separate overlay section for landmarks, entrances, and
within-zone directions. It does not complete campaign objectives, claim
rewards, change the frontier, or infer your position inside a zone.
Client.txt provides zone transitions, not coordinates or landmark detection.
The normal overlay shows all cues for the current supported area; compact
mode continues to show the campaign objective and next zone.

## Current content

Twenty cues cover thirteen Act 1 areas: Clearfell, Grelwood, Red Vale,
Grim Tangle, Cemetery, both tombs, Hunting Grounds, Freythorn, Farmlands,
Village, Ramparts and Manor. The Mobalytics comparison resolved the mushroom
clue to Grelwood and replaced Freythorn's guaranteed smoke directions.

See [the comparison](POE2-NAVIGATION-COMPARISON.md) for each change, confidence
limits, source date and optional checks. Mobalytics warns that 0.5 reworks
layouts; these directions are not independently verified for 0.5.5. The
overlay displays that caveat and the applicable attribution. Patterns may
rotate or mirror. Manor hints are labelled by floor; the app does not infer
a floor, coordinates or discovered landmarks from the current log data.
There are no diagrams or automatic cue completion.

The Cemetery cache cue remains from the MIT guide; Mobalytics does not
corroborate it. Optional cache/area directions are navigation advice, not
required campaign objectives. Unsupported areas have no Navigation section.

## Adding coverage

`content/poe2/navigation.json` is generated with:

```sh
python tools/import_poe2_navigation.py
python tools/import_poe2_navigation.py --check
```

The importer combines the existing pinned Act 1 HTML with curated original
summaries in `content/poe2/navigation-review.json`. Revision, normalized
UTF-8/LF checksum, source anchors and comparison metadata are recorded in
the pack. Edit the review file to maintain the comparison; do not hand-edit
the generated `navigation.json`.
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
