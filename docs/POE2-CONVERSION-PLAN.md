# PoE2 conversion implementation plan

Status: implementation proposal with confirmed release scope and remaining research decisions. No implementation is included.

## Objective and scope decisions

Deliver a PoE2-only, Windows-only, English-only passive campaign companion that reads Client.txt, tracks location and route progress, displays actionable campaign guidance, and retains overlay controls, run timer, settings, persistence and diagnostics. The initial release includes campaign routing and Windows desktop distribution. PoE2 layouts and build imports/reminders are deferred follow-on features, not release blockers.

The existing app provides PoE1 campaign guidance for Acts 1–10, not endgame guidance. PoE2 Atlas progression, mapping objectives and endgame boss-unlock tracking would be new product scope, not existing features to convert. Endgame guidance is explicitly excluded from the initial release. Endgame areas must be handled safely without corrupting campaign progress.

Confirmed decisions and remaining questions:

1. **Game support — confirmed:** PoE2-only conversion. Do not maintain a dual-game selector or require ongoing PoE1 compatibility. Preserve existing user data during migration and test that old progress is never interpreted as PoE2 progress.
2. **Release definition — confirmed:** campaign routing and desktop distribution. Layouts and build features follow separately. Remove or disable incompatible PoE1 diagrams and build controls in the initial release.
3. **Platforms — confirmed:** Windows only. Linux and macOS runtime/package validation are outside this conversion's acceptance criteria.
4. **Language — confirmed:** English only. Document this requirement and fail safely for unrecognized/localized messages.
5. **Campaign coverage — confirmed:** target **0.5.5**, confirmed by the user, and its full available campaign. Record the campaign structure and verify source content against that client. Do not assume PoE1's ten acts or an older PoE2 campaign structure.
6. **Routing goals — confirmed:** a fresh-character general-purpose route with permanent-bonus detours explicitly labeled. Hardcore/SSF-specific variants and speedrun optimization are deferred.
7. **Endgame — confirmed exclusion:** no Atlas, mapping or endgame boss guidance. Entering endgame must be handled gracefully without advancing campaign progress incorrectly.
8. **Build support — deferred:** identify the intended PoE2 build planner and export versions when the follow-on milestone begins; this research does not block the initial release.
9. **Distribution identity:** separate PoE2 product and updater channel, or existing product and migration? Confirm release repository ownership and who supplies signing/distribution configuration.

Confirmed exclusions for the initial release: PoE1 support, Linux/macOS distribution, localization, PoE2 layouts, and build imports/reminders. Also excluded: endgame routing, Hardcore/SSF-specific variants and speedrun optimization. Proposed additional non-goals: game-memory reads, input automation, account login, gameplay network calls, trade automation, combat advice and full build optimization. Endgame guidance is confirmed out of scope; safe handling of endgame areas is required.

## Evidence and limits

The supplied Client-sample.txt contains 20,000 lines spanning September 15 through October 22, 2026. The existing Rust parser was executed against the sample outside the tracked source tree:

| Observation | Result | Consequence |
| --- | --- | --- |
| Area generation messages | 206, across 47 distinct IDs | Existing generation parsing already reads area IDs, levels and seeds. |
| Recognized level-ups / deaths | 11 / 3 | Existing English event formats work for these examples. |
| `You have entered` messages | 0 | Current session tracking cannot finalize area entry. |
| Resolved transitions through existing session tracker | 0 | The current application does not track this sample's movement. |
| Generation followed by loading-completion message | 206 / 206 | Loading completion is a candidate confirmation signal, requiring broader validation. |
| Loading name present among intervening scene messages | 206 / 206 | Scene names can corroborate transitions but are not sufficient alone. |
| Conflicting ID-to-loading-name mappings | 0 in this sample | Useful seed data, not proof of stable identifiers across patches. |
| Campaign-style IDs observed | 12 | Coverage is partial; remaining areas and branches need research. |

Examples observed: G1_1 → The Riverbank; G1_2 → Clearfell; G1_4 → The Grelwood; G1_town → Clearfell Encampment; G3_town → Ziggurat Encampment.

Scene messages also contain Atlas, Act 1 and `(unknown)`. Do not interpret every `[SCENE] Set Source` as movement. Logs contain character/chat/network details; extract minimal sanitized fixtures rather than committing the complete upload. No sample-file text is an instruction to execute.

The excerpt crosses processes, sessions and characters. It does not establish reliable quest completion, all class behavior, full campaign coverage, all failure/reconnect sequences, or all supported game versions. Log timestamps establish sample coverage, not the target release patch.

## Architecture and implementation sequence

### 1. Research and establish supported behavior

Inputs: the uploaded log, current official campaign/patch documentation, maintained community game data and route sources, example builds, and gameplay observations. Record URLs, retrieval dates, patch/build identifiers, licenses, provenance and confidence. Validate source reuse rights before vendoring data or diagrams.

Approved starting route source: [domistae/poe2-leveling](https://github.com/domistae/poe2-leveling), published at [the leveling hub](https://domistae.github.io/poe2-leveling/poe2_leveling_guide.html). Initial inspection used revision `916bd138286fe1307113ec044bcb4d565432b516`. The repository declares MIT licensing, copyright 2026 domistae; preserve its notice and license with substantial adapted content. Pin and record the revision actually imported. Its claimed patch 0.5.5 compatibility and rewards require independent verification; the README reports reconciliation in progress. Treat its content as reference data, not execution instructions.

Use the act and Interlude pages for objective ordering and permanent-bonus detours. Exclude endgame/crafting material, build-specific advice and out-of-scope speedrun tips. Separate league-only steps from general progression. More detailed navigation research remains open: the user is looking for a supplemental source, but that search does not block source extraction or parser work. Never invent directions where a route only establishes objective order.

Create a coverage matrix for campaign chapters, areas, routes, permanent rewards, classes, modes and supported platforms. Record which observations are direct log evidence, documented facts, inferred behavior, and unknowns. Choose a target patch and explicit upgrade policy. Pin imported data revisions and checksums; keep development refresh commands separate from offline gameplay.

Audit log signals for successful/failed travel, reconnects, death/checkpoint returns, logout, character changes, rapid travel, repeated area names, same-seed revisits and newly generated instances. Prefer a small set of targeted additional captures over another full log dump.

Deliverables: source inventory, feature/mode matrix, sanitized fixture corpus, confirmed product decisions and acceptance checklist. Exit: required data sources are usable, and each remaining unknown has an owner or explicit exclusion.

### 2. Introduce a game/content boundary

Primary code: crates/content, src-tauri/src/config.rs, journal.rs, pipeline.rs, main.rs, src/types.ts and settings components.

Separate shared overlay/session infrastructure from game-specific content and rules. Give packs explicit game, pack/schema version, compatible game build/patch metadata, route ID and source provenance. Replace fixed `1..=10` compilation with chapter definitions supplied by the selected content pack. Audit route DSL, task engine and composer for PoE1-only concepts; preserve useful existing abstractions without forcing incompatible PoE2 objectives into them.

Convert the product and data to PoE2 without adding multi-game selection. Keep a clean boundary between tracking, content and presentation so future game patches do not require rewriting the overlay. Use a distinct PoE2 config/progress namespace and deliberate product/update migration policy. Do not delete or overwrite existing PoE1 settings/journals during setup or upgrade. PoE1 fixtures may remain where they validate shared parsing, but ongoing PoE1 functionality is not a release requirement.

Version configuration and journals. Migration must never reinterpret a PoE1 area/quest/gem identifier as PoE2, merge unrelated characters' progress, or discard existing user data silently. Decide how character identity is selected when logs cannot identify it reliably.

Exit: synthetic and real fixtures load the selected profile; wrong-game and incompatible packs fail with a useful message; old settings are preserved and incompatible PoE1 progress is not loaded.

### 3. Implement PoE2 event normalization and transition tracking

Primary code: crates/event_parser, crates/input_log, crates/session and crates/replay.

Normalize `Generating level ... area ... with seed ...`, scene names and `[LOADING SCREEN] (...) Duration = ...` into typed events. Keep raw evidence and confidence internally where helpful. Evaluate a state machine that stages generation and confirms entry on a matching loading-completion event. Correlate within the current process/session, expire stale pending transitions, and avoid emitting duplicates. Do not advance progress merely because generation started.

Ignore menu/atlas/null scene labels. Preserve authoritative area IDs even when a name is unknown. For unknown content, show a safe location/unmapped state and diagnostics; never guess a matching PoE1 area. Do not infer death from loading or reconnection alone. Reuse the observed level/death parsing after validating more cases.

Define tail-at-end startup, optional developer replay, partial-line handling, truncation/rotation, non-ASCII names and encoding behavior. Audit single-character pinning: the first level-up in a multi-session log is not necessarily the active character. Specify session boundaries and safe recovery before persisting progress.

Tests: all supplied 206 generation/loading pairs produce the expected normalized confirmations; duplicate messages do not double-count; scene-only UI changes do not move the player; fixtures cover failure, reconnect, stale/mismatched generation, log restart and character changes. Annotate expected results independently of the parser. Matching 206 pairs proves sample compatibility, not universal correctness.

Exit: replay produces verified ordered transitions and correct revisit/new-instance behavior with no unintended campaign advancement.

### 4. Build the PoE2 campaign content pipeline

Primary code/data: crates/content, content/, vendor/, tools/ and compiled content-pack output.

Create versioned PoE2 area and objective datasets, including chapter, display name, town/hub status, connections, variant/context identifiers and source attribution. Derive candidate ID/name mappings from logs, then validate against authoritative or maintained data. Do not infer the complete graph from ID prefixes.

Author routes for every agreed chapter and mode, with mandatory progress, permanent-bonus detours, exits, return-to-hub steps and route variants. Separate objective availability from completion. If Client.txt does not prove boss kills, item collection or quest rewards, keep those steps advisory or introduce explicit user confirmation; never label inferred completion as observed fact.

Validate referential integrity, reachable route transitions, variant completeness, text quality and patch compatibility. Add per-chapter golden replays plus cross-chapter continuity tests. Keep route assets available offline. Establish a repeatable patch-update process that reports changed/removed IDs and invalidates unsupported guidance.

Exit: complete agreed campaign coverage, no unresolved required references, and representative route walkthroughs reviewed by a knowledgeable PoE2 player.

### 5. Adapt route progression and task guidance

Primary code: crates/route_engine, crates/task_engine, crates/composer and src-tauri/src/pipeline.rs.

Validate the existing frontier/focus model against PoE2's actual campaign graph. Support branches, revisits, hub returns, deaths, new instances, optional detours, mode/context distinctions and out-of-campaign locations. Preserve progress during revisits without skipping required objectives. Define explicit handling for manual correction, reset/resume and content-version changes.

Replace PoE1 reward, vendor and crafting assumptions with agreed PoE2 objectives. Ensure reminder timing reflects evidence actually available; label availability separately from acquisition. Test chapter completion and campaign completion against the confirmed target structure.

Exit: end-to-end replay from raw log to rendered overlay model agrees with annotated walkthroughs, including off-route recovery and persistence/restart.

### 6. Deferred: layouts and navigation guidance

Primary code/data: content/layouts, crates/content/src/layouts.rs, composer layout output and src/FilmstripBar.

Inventory licensed PoE2 diagrams, obtain permission where required, or author original guidance. Validate against the target patch and multiple generated layouts. Preserve per-asset provenance, audit status and uncertainty labels. Do not reuse PoE1 diagrams for similarly named PoE2 areas.

Define required layout coverage explicitly: supported areas and expected variants, or a documented unavailable state where no reliable layout exists. Avoid implying that a static diagram predicts a random layout. Keep the overlay usable when no diagram is available, and measure asset size and rendering performance.

Exit: the agreed layout coverage matrix is satisfied, assets are attributable, and stale/missing diagrams have safe behavior. If diagrams are required for release, missing coverage blocks full parity unless the scope is explicitly revised.

### 7. Deferred: build imports and reminders

Primary code: crates/pob_import, content gem/reward data, composer reminders and settings preview.

Research the selected PoE2 planner's actual export format, version markers, classes/ascendancies, skill groups, leveling sections and notes. Obtain representative real exports before implementing schema assumptions. Determine which information exists as structured data and which cannot be inferred reliably.

Keep decoding separate from game-specific interpretation. Reject or clearly identify wrong-game and unsupported versions. Preserve graceful handling of malformed/partial builds, and constrain imported text/XML sizes and parser behavior. Replace PoE1 gem/vender/quest enrichment with validated PoE2 acquisition, requirement and skill/support rules where in scope.

Only show leveling reminders justified by explicit or validated structured build data. A final endgame build is not a leveling plan. Avoid promising optimization or automatically generating unsupported advice. Maintain preview/reliability labels and route-only operation without an import.

Exit: approved sample exports yield independently expected build plans and reminders; incompatible/unsupported imports remain safe and understandable.

### 8. Product integration and user migration

Update product names, PoE2 log discovery and help, route options, default shortcuts if needed, diagnostic reports, screenshots and documentation. Remove game-selection UI and hide/remove PoE1 layout/build features for the initial release; do not present nonfunctional placeholders as supported features. Audit Rust/TypeScript payload parity. Preserve setup mode, click-through, opacity, resizing, hide/show, timer, hotkeys and settings behavior on Windows.

Clearly communicate unsupported patch/content, unknown areas, missing layout/build data and unavailable logs. Keep implementation details out of routine user flows. Exclude raw logs and private paths from automatic uploads; bug reporting should remain an explicit user action.

Exit: a fresh user can select the correct log and route, understand the overlay and recover from misconfiguration; existing users have a tested migration path.

### 9. Release validation and distribution

Run Rust workspace tests/clippy and frontend tests/typecheck/build. Build the Windows installer and launch the installed application, rather than validating only a development binary. Test updater manifest identity, compatibility, signatures and upgrade/rollback behavior using a controlled release channel.

Run a manual Windows gameplay checklist: windowed fullscreen overlay, click-through and focus, hotkeys, display scaling/multiple monitors, settings persistence, log access/rotation, restart/reconnect, loading failures, deaths, towns, detours, route completion, and behavior after a game patch. Verify no gameplay network access is introduced. Signing material belongs in secure release configuration, never fixtures or source.

**In-game validation is user-executed.** There is no automated in-game validation path. Every implementation task requiring real PoE2 gameplay must include a concise validation handoff with:

- The exact app build/commit and game patch, prerequisites, and how to launch/configure that build.
- A short numbered sequence of actions, scoped to the changed behavior.
- The expected visible result after each action and a clear pass/fail condition.
- What to report: failed step, actual result, relevant timestamps, and only the minimal sanitized log excerpt or screenshot needed.

Prefer a reusable existing character/checkpoint over requiring a new campaign run unless fresh-character behavior is the subject. Do not require risky or destructive actions merely to test a feature; identify a suitable safe opportunity for death/reconnect cases. Keep automated replay/unit/build results separate from user gameplay results. Mark each manual check as **pending**, **passed**, **failed**, or **blocked**, with the tested build/patch. An implementation can be ready for user testing while its gameplay acceptance gate remains pending; never report an unexecuted manual check as passed. Continue independent work while awaiting results and turn reported failures into sanitized regression fixtures when possible.

Environment limitation: the cloud Linux machine validates shared Rust/frontend logic, but its native WebKit launch is blocked by missing system-path helpers and system installation privileges. Linux runtime is out of scope, so fixing that launch is not a release prerequisite. Establish a Windows CI runner for compilation/installer checks and a Windows desktop with PoE2 for actual overlay/gameplay validation; Linux test results cannot substitute for those checks.

Exit: the Windows installer and installed app work, complete agreed campaign routing has been reviewed and exercised, legacy PoE1 diagrams/build reminders are not exposed as PoE2 features, and release/update ownership is configured. Deferred layouts/build features do not block this release.

## Milestones and dependencies

| Milestone | Scope | Acceptance gate |
| --- | --- | --- |
| M0: agreed specification | Scope, target patch, sources, fixtures and validation matrix | Product decisions resolved; required source/data availability established. |
| M1: PoE2 location tracker | Profile boundary, event/session changes, overlay location, replay | Supplied transitions reproduced; failure/revisit fixtures pass. |
| M2: Act/chapter 1 vertical slice | Content compiler, one full route, progress, settings and persistence | Annotated fresh-character walkthrough matches UI and saved progress. |
| M3: complete campaign | All agreed chapters/modes/variants and objective guidance | Coverage matrix and end-to-end walkthroughs pass. |
| M4: initial Windows release | Final UI/migration, installer, Windows gameplay checks, updates and docs | Campaign, Windows and distribution gates pass; layouts/build features remain deferred. |
| M5: follow-on features | PoE2 layouts and build import/reminders | Separately agreed coverage and reliability gates pass. |

Dependency order for the initial release: M0 → M1 → M2 → M3 → M4. M5 follows independently and does not block M4. Research layouts/builds only when useful resources become available or that work is commissioned. Ship an earlier preview only with explicit campaign-coverage limitations. Do not announce completed campaign conversion at the zone-tracker stage.

Re-estimate after M0 and again after M2. The earlier several-day prototype estimate applies to M1, not full parity. The initial campaign/Windows conversion should be budgeted in weeks; a firm date is premature until full campaign data and Windows gameplay validation availability are known. Estimate deferred diagrams/build work separately. Track implementation effort separately from time waiting for resources and gameplay validation.

## Requested resources, in priority order

1. Confirm distribution identity before packaging: product name, release repository and updater ownership. Scope is settled: current live patch at implementation start, full available campaign, fresh-character route with permanent-bonus detours, no endgame guidance, and deferred Hardcore/SSF-specific variants and speedrun optimization.
2. Starting guide approved: domistae/poe2-leveling (see research section). Additional navigation-specific references and sources for area IDs, quests and permanent bonuses remain useful; their absence does not block initial work.
3. Small targeted log captures from missing campaign chapters and edge cases. Accompany each with the game build, client language, class, and a short description of what happened. Include failed travel/reconnect, checkpoint/death recovery and character switching where possible. Sanitize chat, account/character identifiers and network details while preserving event order and correlation.
4. Deferred until build-feature work: representative PoE2 planner exports: a structured leveling build, an endgame-only build, and a known unsupported/malformed example. Record planner/export version and expected interpretation; do not include private credentials.
5. The user will execute in-game Windows validation using concise task-specific instructions. No automated gameplay path is available. Licensed layout resources or permission to author original diagrams are only needed for the deferred layout milestone.
6. When distribution work begins: confirmed product/release ownership and access to Windows gameplay validation machines/CI. Secure signing configuration is a release prerequisite, not a prerequisite for research or parser implementation.

Most research and parser work can proceed with the existing sample. Request each additional resource when it unblocks its milestone; do not require a complete collection before starting independent work.

## Implementation checkpoint: 0.2.0-alpha.1

- M1 automated implementation: PoE2 loading completion, bounded generation correlation, process/login isolation, raw identity for unknown areas, and sanitized 206-transition replay coverage. Native Windows/gameplay acceptance is pending.
- M2 preview implementation: the shipped application selects PoE2 only and embeds 65 advisory Act 1 objectives across 18 visits from the approved pinned source. No diagrams/build features or PoE1 routes are loaded in production. Full Act 1 walkthrough and source/live-patch verification remain pending. Some guide side areas occur within objective text rather than as standalone route visits and currently show the safe unavailable state.
- User state is isolated with a distinct PoE2 app identifier. No automatic migration or cross-character progress merge is attempted; login/process boundaries start a fresh route session.
- Old upstream update endpoints are disabled. Version 0.2.0-alpha.1 is a development preview, not a published Windows release.
- M3 full campaign, distribution/signing/update ownership and full Windows acceptance remain outstanding. Deferred layouts and builds remain out of scope.
- See [Windows validation instructions](POE2-WINDOWS-VALIDATION.md). All in-game checks start pending; automated replay success is separate evidence.

Current automated evidence: 241 Rust tests and 120 frontend tests passed;
Rust formatting, Clippy with warnings denied, pinned-source reproducibility,
frontend typecheck/build and Linux desktop compilation passed. The CLI also
reproduced 206 confirmed transitions. No Windows CI execution, installed
Windows launch, or in-game validation is claimed; those checks remain pending.

Security follow-up: [0.2.0-alpha.2](SECURITY-UPDATE-alpha2.md) updates the
development toolchain, removes diagnostic URL prefill and unused updater
dependencies/permissions. Fresh npm scan: zero matches. Rust scan: two
remaining matches in the disabled legacy PoB library. Full campaign scope
and pending Windows validation are unchanged.

Target-patch confirmation: the user reports PoE2 **0.5.5**, matching the
starting guide's claimed version. This confirms the target version, not all
guide rewards/directions. The user reports the early-town fix and direct
overlay confirmation working. Detailed side-area content validation remains
pending under the Windows checklist. Supplemental side-area guidance now
covers Mud Burrow, Mausoleum of the Praetor and Tomb of the Consort without
inserting inferred route progress or fabricated game IDs.

## Remaining campaign checkpoint: 0.2.0-alpha.3

Acts 2–3 now have source-imported sequential visits. Act 4 and all three
Interludes have independent local guidance so an alternate branch order
cannot mark other content complete. Original Act 1 behavior, explicit town
confirmation and Campaign/Optional grouping remain. Endgame guide content
is excluded. See [coverage and user validation](POE2-CAMPAIGN-COVERAGE.md).
This advances M3 implementation coverage; it does not close its in-game
accuracy/complete-walkthrough acceptance gate or claim Windows certification.
