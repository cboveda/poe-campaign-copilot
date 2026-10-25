# Credits & Attribution

PoE Campaign Copilot is an unaffiliated fan tool. Path of Exile is a
registered trademark of Grinding Gear Games.

- **Route data** — vendored from
  [HeartofPhos/exile-leveling](https://github.com/HeartofPhos/exile-leveling)
  (MIT). See `vendor/exile-leveling/LICENSE` and `VENDOR.md` for the pinned
  commit.
- **Zone layout images** — created by **Engineering Eternity**
  (https://www.youtube.com/@EngineeringEternity), included with credit from the
  community-shared ["Cheat sheets based on Engineering Eternity" compilation](https://docs.google.com/document/d/1sExA-AnTbroJ-HN2neZiij5G4X9u2ENlC7m_zf1tqP8/edit);
  committed under `content/layouts/assets/`. The legend image in the README
  (`docs/images/layout-legend.png`) is from the same compilation.
  Exception: the Act 10 Ravaged Square composite image was created by the
  compilation's author. If you are a rights holder and want anything removed,
  open an issue and it will be taken down promptly.
- **Zone layout notes** — adapted from the same community
  ["Cheat sheets based on Engineering Eternity"](https://docs.google.com/document/d/1sExA-AnTbroJ-HN2neZiij5G4X9u2ENlC7m_zf1tqP8/edit)
  compilation (author's notes based on Engineering Eternity's videos);
  committed as structured JSON under `content/layouts/act-*/`. The notes
  date from an older patch and are being audited zone-by-zone: most still
  carry `audit.status: "unaudited"`, while reviewed ones are marked
  `verified`, `outdated`, or `corrected` (with replacement text).

- **PoE2 Act 1 route objectives** — adapted from
  [domistae/poe2-leveling](https://github.com/domistae/poe2-leveling),
  revision `916bd138286fe1307113ec044bcb4d565432b516`,
  copyright (c) 2026 domistae, MIT. Pinned source and license are in
  `vendor/poe2-leveling/`; `tools/import_poe2_act1.py` generates the preview.
  Source patch/reward claims await live-game verification. Existing PoE1
  content above remains only as legacy code/test references; the PoE2 app
  does not display its diagrams or build reminders.

- **Remaining PoE2 campaign guidance** — Acts 2–4 and Interludes adapted
  from the same domistae/poe2-leveling revision and MIT license above.
  `tools/import_poe2_campaign.py` records per-file source checksums and
  step references. Endgame content is excluded. Source claims remain
  advisory pending patch 0.5.5 gameplay review.

PoE2 within-zone navigation cues are extracted from the same pinned MIT
Act 1 guide by domistae. Source revision, checksum and section anchors are
recorded in `content/poe2/navigation.json`.
