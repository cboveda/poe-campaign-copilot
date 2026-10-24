# Windows / in-game validation: 0.2.0-alpha.2

Status: **pending — not executed in this cloud environment**.
Test the source containing these PoE2 changes, not the previous PoE1 main.
Record `git rev-parse HEAD` and whether your checkout has local modifications,
Windows version, PoE2 patch/build, class and client language. The guide claims
0.5.5; the current live patch must be verified before calling it compatible.

## Start the preview

Prerequisites: Windows, English PoE2, Node 24 LTS or Node 22.12+ (22 LTS series), stable Rust, Visual Studio
C++ tools/Windows SDK and WebView2. From the repository root:

```powershell
npm ci
npm run tauri dev
```

Expected: **PoE2 Campaign Copilot** opens with a waiting overlay. Open tray →
Settings, select your PoE2 installation's `logs\Client.txt`, and use windowed
fullscreen in PoE2. Route variant/build import controls should be absent.
This build has separate settings from PoE1 and automatic updates disabled.

## Required short gameplay check (about 5–10 minutes)

Use an existing character and accessible areas; a new character is unnecessary.
Do not reset or clear your real Client.txt. Do not deliberately die or force a
network failure just to test this preview.

1. Enter **Clearfell**, then **The Grelwood** (or two accessible Act 1 areas).
   **Expected:** the zone label changes once after each successful load; the
   Act 1 objective list corresponds to the zone and displays a preview label.
   No PoE1 names/images or build advice appear. Report any repeated flicker.
2. Open/close the world map or Atlas without traveling.
   **Expected:** the zone and route do not change to “Atlas”, “Act 1” or an
   unknown scene label.
3. Portal/waypoint back to town, then return to the same field instance.
   **Expected:** town and field names follow the actual locations. A revisit
   does not falsely finish the campaign or report quest/reward completion.
4. If you already have an endgame character/location available, enter a
   hideout or map. Otherwise mark this step **blocked: no endgame access**.
   **Expected:** the actual location name appears with guidance unavailable;
   Act 1 objectives disappear and campaign completion is not asserted.
5. Use Alt+Shift+H to hide/show, Alt+Shift+S to move/resize, and Alt+Shift+T to
   pause/resume the timer. Click through the overlay with setup mode off.
   **Expected:** controls work, clicks reach PoE2, game focus remains usable,
   and the first successful zone load started the timer.
6. Restart only the overlay while remaining in the same game session.
   **Expected:** the journal restores the last observed location/advisory route
   without mixing PoE1 state. Enter another zone to confirm live tracking resumes.

**Pass:** each executed step matches its expected result with no crash/freeze.
Mark inaccessible tests blocked, not passed. If a load exceeds three minutes,
report it: the prototype expires pending generation after that limit.

## Follow-up checks at a convenient opportunity

- During normal play, observe a natural death/checkpoint return and a normal
  logout/relogin or character switch. The next confirmed zone should be
  correct. Login/process changes intentionally start a fresh route session;
  per-character resume across game sessions is not implemented.
- During an Act 1 run, compare displayed boss/quest/permanent-bonus objectives
  with actual gameplay. Report wrong names, rewards, order, missing areas or
  misleading directions. The app cannot verify that these actions were done.
- Test display scaling/multiple monitors if relevant to your setup. Installer,
  update and full-campaign acceptance are later checks, not covered here.

## Report results

Send this compact record (no full private log needed):

```text
App: 0.2.0-alpha.2; source commit: ...; modified files: yes/no
Windows / PoE2 patch / class / English client: ...
Steps 1–6: passed / failed / blocked (one short reason per exception)
Failed step: expected ...; actually ...; timestamp ...
```

For a tracking failure, attach only the nearby generation, scene and loading
messages from Client.txt plus a screenshot if it clarifies the issue. Remove
chat, player/account names and server addresses; keep event ordering,
timestamps, area names/IDs, seeds and process markers. Use tray → Open logs
for app diagnostics, reviewing paths/names before sharing.
