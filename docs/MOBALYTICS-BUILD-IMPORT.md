# Mobalytics campaign build import (alpha.6)

The app can import a public English Mobalytics PoE2 build page directly,
without PoB, a Mobalytics account, browser automation, or executing page
scripts. Importing stores a local snapshot. Starting the app and playing
with an imported build do not make network requests.

## Windows usage

1. Stop the running development app with Ctrl+C. In the repository folder,
   run `git pull --ff-only`, then `npm.cmd run tauri dev`.
2. Open Settings (Alt+Shift+O). In **Build guide**, paste a public
   `https://mobalytics.gg/poe-2/builds/...` link and click **Preview import**.
3. Check the title, author, campaign stages and excluded endgame stages.
   Click **Save imported build**. This replaces the saved build, clears its
   build checkmarks across all profiles, and preserves campaign progress.
4. Set **Character / progress profile** to your character's name, then click
   **Use character**. Choose an existing name to resume its build checklist;
   profiles are selected manually, not inferred from other players' log lines.
5. Leave **Build stage** on Automatic to follow your current supported
   campaign area, or choose a stage manually. Unknown stage names require
   manual selection. Overlapping/ambiguous act matches are not guessed.
6. Check targets after you equip a skill/support, obtain suitable gear, or
   allocate a passive. Undo a check to restore a reminder. Read **Author's
   stage instructions** for acquisition order and equipment priorities.
7. The normal overlay shows the first four unchecked targets separately
   from Campaign, Optional, and Navigation. Compact mode keeps its existing
   campaign view. Checklist controls are in Settings.

For the requested [DEADRABB1T grenade build](https://mobalytics.gg/poe-2/builds/deadrabbit-grenades-gemling-league-starter),
the importer recognizes Act 1, Act 2, Acts 3/4 and Interludes. Its Early,
Mid and Uber Endgame variants are excluded. Supports are linked to their
active skill; unresolved support IDs are visibly labelled, never guessed.

**Check for guide update** fetches another preview. Nothing changes until
**Save imported build** is clicked. A failed fetch or parse leaves the saved
build intact. Replacing a snapshot clears its checkmarks to avoid retaining
completion against changed targets; the UI states this before saving.
**Remove saved build** removes its snapshot and build checkmarks without
changing campaign guidance, the timer, or Client.txt selection.

## If Mobalytics returns HTTP 403

Access may differ between the app and your browser. A successful development
fetch does not guarantee access from every Windows PC.

1. Open the original build link in your normal browser and let it finish loading.
2. Press Ctrl+S. Choose **Webpage, HTML Only** or **Webpage, Complete**, and
   save the `.html` file. Do not choose PDF or Webpage, Single File (`.mhtml`).
3. In Settings, keep the original build URL in the link field. Click
   **Preview saved page** and select the saved `.html` file.
4. Review the preview, then click **Save imported build**.

The app reads only the chosen local HTML file, capped at 8 MiB. It does not
load its companion folder, execute scripts, reuse browser cookies, or fetch
linked assets. The same URL/document matching and explicit-save rules apply.
A cancelled picker or invalid file leaves the saved build unchanged.
To refresh a blocked build, save a new copy from the browser and repeat.

## If saving the page stalls

A Complete save can wait on remote images, ads or other assets. Cancel it
and try **Webpage, HTML Only**, or use the no-download option in alpha.6:

1. Open the build page in your normal browser and press **Ctrl+U**.
2. In the source tab, press **Ctrl+A**, then **Ctrl+C**.
3. In Settings, expand **Paste page source instead (no download)**.
4. Paste into **Page source HTML**, keeping the original build URL above.
5. Click **Preview pasted source**, review, then **Save imported build**.

Copy the source HTML, not the rendered article's visible text. This sends
text only to the local native parser; there is no external fetch, automatic
clipboard access, or script execution. The same 8 MiB cap and matching-build
checks apply. The text field clears after a successful preview. Nothing is
saved until the explicit Save action.

## Scope and limitations

The provider-neutral plan contains staged skills/supports, equipment targets,
author notes, passive-node IDs and named passive priorities. Each stage joins
to its own description widgets by ID, preserving acquisition and equipment
advice without mixing other variants' instructions. Names with no explicit
act mapping remain manual stages; the app does not invent level ranges.

Client.txt supplies campaign location and observed level, not equipped gear,
socketed gems, allocated passives or proof of rewards. Build completion is
manual. Location selects stages; no level-based unlock inference or gem
availability database is included yet. Gear values are guide examples, not
exact required items. Named passive priorities are targets, not an inferred
point-by-point allocation order. Full passive/weapon-set tree rendering,
PoE2 PoB import, calculations and account synchronization remain deferred.

Build snapshots and named-profile progress are stored in `build.json` beside
the app's `config.json` in its Windows application configuration directory.
The saved build is independent of the campaign journal: changing build
stages or checkmarks never resets routing. Reset progress continues to reset
the campaign; use a new build profile for a new character. Per-character
campaign route resume remains a separate, unimplemented feature.

Mobalytics' embedded schema is not a documented public import contract.
Unsupported pages, access challenges, redirects or format changes produce
an error. The app keeps the existing snapshot for offline use. Imported
instructions are guidance from the author, not independently verified build
advice. Network and importer tests do not establish in-game accuracy.

## Request and data handling

Native Rust fetches only canonical HTTPS `mobalytics.gg/poe-2/builds/<slug>`
URLs. `www` input is normalized, query/fragment tracking is removed, and
credentials, other hosts/ports and additional path components are rejected.
Redirects are disabled. No images, linked sites, script URLs or authenticated
APIs are fetched. The request uses normal TLS certificate verification,
a 30-second timeout and an 8 MiB response cap.

Only JSON following the page's `window.__PRELOADED_STATE__` assignment is
parsed. A matching build document is required. JavaScript, HTML and author
instructions are never executed. Rich text becomes plain React text. Saved
snapshots are capped at 1 MiB, written through a temporary file and replaced
before the live plan is changed. Bad/oversized saved files load as no build.
The TLS provider uses ring to avoid adding a Windows CMake/NASM prerequisite.

## Verification

Automated checks cover URL boundaries, missing/changed schema, stage joins,
support relationships, endgame exclusions, manual/automatic selection,
profile isolation, reversible completion, persistence, and rendering. A
synthetic fixture is committed; the full third-party build page is not.
The actual requested page was separately checked through a local capture
and a live native HTTPS smoke test.

Optional live checks (not in the offline test suite):

```sh
cargo test --locked -p poe-copilot-app live_build_fetch_smoke -- --ignored
POE_MOBALYTICS_HTML=/path/to/saved-page.html cargo test --locked -p poe-copilot-app captured_real_build_smoke -- --ignored
```

## Optional user validation

No in-game execution is automated. When checking on Windows:

1. Import the supplied link. Confirm the four campaign stages and linked
   grenade supports appear; read the Act 1 acquisition instructions.
2. Check a target, restart the app, and confirm that the same named profile
   retains it. Switch to another profile and confirm its checkbox is clear.
3. Choose Act 2 manually and then Automatic; neither operation should reset
   the campaign, town confirmation or timer.
4. In game, enter an Act 2 area and confirm the build stage becomes Act 2.
   Enter an unsupported/endgame area with Automatic selected: campaign build
   guidance should disappear. No boss kill or fresh character is required.
5. Disconnect from the network and restart: the saved build should remain
   usable. A failed update check should show an error and keep that snapshot.
