//! Pure pipeline: log lines -> UiModel. No Tauri types here.
//! NOTE: images load from the repo's content/layouts/assets at runtime —
//! fine for development; distribution packaging revisits this path.

use std::collections::HashMap;
use std::path::{Component, Path};

use base64::Engine as _;
#[cfg(test)]
use composer::layouts_by_area;
use composer::{BuildContext, OverlayModel, compose};
#[cfg(test)]
use content::compile::{Variant, compile_route_pack};
use content::game_data::AreaMap;
#[cfg(test)]
use content::game_data::load_vendored;
#[cfg(test)]
use content::layouts::load_all_layouts;
use content::layouts::{LayoutEntry, layouts_dir};
use route_engine::RouteEngine;
use serde::Serialize;
use session::{SessionEvent, SessionTracker};
use task_engine::TaskEngine;
use thiserror::Error;

#[derive(Debug, Clone, Serialize)]
pub struct UiImage {
    pub file: String,
    pub stale: bool,
    pub data_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiModel {
    pub overlay: OverlayModel,
    pub images: Vec<UiImage>,
    pub waiting_for_log: bool,
    pub pending_town_return: bool,
    /// Short label for the imported leveling build, e.g. "Ranger (Deadeye)
    /// — 12 milestones". `None` when no build has been imported.
    pub build_summary: Option<String>,
}

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("game data: {0}")]
    GameData(#[from] content::game_data::GameDataError),
    #[error("route compile: {0}")]
    Compile(#[from] content::compile::CompileError),
    #[error("layouts: {0}")]
    Layouts(#[from] content::layouts::LayoutError),
}

struct ObservedLocation {
    area_id: String,
    name: String,
    act: u8,
    is_town: bool,
    route_area_id: Option<String>,
}

pub struct Pipeline {
    poe2_mode: bool,
    side_area_guidance: Vec<content::poe2::SideAreaGuidance>,
    pending_town_return: Option<String>,
    observed_location: Option<ObservedLocation>,
    tracker: SessionTracker,
    engine: RouteEngine,
    tasks: TaskEngine,
    layouts: std::collections::BTreeMap<String, LayoutEntry>,
    areas: AreaMap,
    seen_area_event: bool,
    encoded: HashMap<String, String>,
    build: Option<pob_import::LevelingBuildPlan>,
}

impl Pipeline {
    /// `variant` selects the compiled route pack (league-start vs.
    /// standard); `build` is the imported leveling-build plan, if any —
    /// `None` when the user hasn't imported a Path of Building code.
    #[cfg(test)]
    pub fn new(
        variant: Variant,
        build: Option<pob_import::LevelingBuildPlan>,
    ) -> Result<Self, PipelineError> {
        let (areas, _) = load_vendored()?;
        let pack = compile_route_pack(variant)?;
        let engine = RouteEngine::from_pack(&pack, areas.clone());
        let layouts = layouts_by_area(load_all_layouts()?);
        Ok(Self {
            poe2_mode: false,
            side_area_guidance: Vec::new(),
            pending_town_return: None,
            observed_location: None,
            tracker: SessionTracker::new(areas.clone()),
            engine,
            tasks: TaskEngine::new(areas.clone()),
            layouts,
            areas,
            seen_area_event: false,
            encoded: HashMap::new(),
            build,
        })
    }

    /// PoE2's first vertical slice: actual location tracking, with no legacy
    /// routes, images, gem data or automatic quest completion attached.
    pub fn new_poe2() -> Result<Self, PipelineError> {
        let (areas, steps) = content::poe2::load_act1_preview()?;
        Ok(Self {
            poe2_mode: true,
            side_area_guidance: content::poe2::load_side_area_guidance()?,
            pending_town_return: None,
            observed_location: None,
            tracker: SessionTracker::new(content::poe2::load_observed_areas()?),
            engine: RouteEngine::new(steps, areas.clone()).with_retained_final_group(),
            tasks: TaskEngine::new(areas.clone()),
            layouts: Default::default(),
            areas,
            seen_area_event: false,
            encoded: HashMap::new(),
            build: None,
        })
    }

    /// Drop all live session state back to a fresh start — the manual
    /// "Reset progress" action. The compiled route returns to Act 1
    /// (`engine.restart`), in-progress task reminders and the tracked
    /// player level are cleared (fresh `TaskEngine` / `SessionTracker`),
    /// and the overlay returns to waiting for the new character's first
    /// zone (`seen_area_event = false`). Everything config-derived — the
    /// route steps, imported `build`, `layouts`, and the content-addressed
    /// `encoded` image cache — is kept.
    pub fn reset(&mut self) {
        self.observed_location = None;
        self.pending_town_return = None;
        self.engine.restart();
        self.tasks = TaskEngine::new(self.areas.clone());
        self.tracker = SessionTracker::new(if self.poe2_mode {
            content::poe2::load_observed_areas().expect("embedded PoE2 observations are valid")
        } else {
            self.areas.clone()
        });
        self.seen_area_event = false;
    }

    pub fn on_line(&mut self, line: &str) -> Option<UiModel> {
        let mut area_changed = false;
        for ev in self.tracker.on_line(line) {
            if self.poe2_mode {
                match &ev {
                    SessionEvent::SessionStarted { .. } => {
                        self.observed_location = None;
                        self.pending_town_return = None;
                        self.engine.restart();
                        self.seen_area_event = false;
                        area_changed = true;
                    }
                    SessionEvent::AreaEntered {
                        area_id,
                        display_name,
                        new_instance,
                        ..
                    }
                    | SessionEvent::UnmappedArea {
                        area_id,
                        display_name,
                        new_instance,
                        ..
                    } => {
                        self.observe_poe2(area_id, display_name, *new_instance);
                        area_changed = true;
                    }
                    SessionEvent::UnresolvedArea { display_name, .. } => {
                        self.pending_town_return = None;
                        // A catalog/name mismatch is not permission to advance a route.
                        self.observed_location = Some(ObservedLocation {
                            area_id: String::new(),
                            name: display_name.clone(),
                            act: 0,
                            is_town: false,
                            route_area_id: None,
                        });
                        self.seen_area_event = true;
                        area_changed = true;
                    }
                    _ => {}
                }
                continue;
            }
            if let SessionEvent::AreaEntered {
                area_id,
                new_instance,
                ..
            } = &ev
            {
                let advance = self.engine.on_area_entered(area_id, *new_instance);
                for &i in advance.newly_done.iter().chain(&advance.newly_skipped) {
                    let step = self.engine.steps()[i].clone();
                    let status = self.engine.statuses()[i];
                    self.tasks.on_step_passed(&step, status);
                }
                area_changed = true;
                self.seen_area_event = true;
            }
        }
        area_changed.then(|| self.current_model())
    }

    fn observe_poe2(&mut self, raw_id: &str, name: &str, new_instance: bool) {
        let known = self
            .areas
            .get(raw_id)
            .filter(|area| area.name == name)
            .or_else(|| {
                // The preview only allows name matching within the observed
                // Act 1 ID family. Endgame/hideout names must never select a
                // campaign route just because a label happens to coincide.
                raw_id
                    .starts_with("G1_")
                    .then(|| {
                        self.areas
                            .values()
                            .find(|area| area.act == 1 && area.name == name)
                    })
                    .flatten()
            });
        let route_id = known
            .filter(|area| {
                self.engine
                    .steps()
                    .iter()
                    .any(|step| step.area_context == area.id)
            })
            .map(|area| area.id.clone());
        let side = self.side_guidance_for(raw_id, name);
        let location = ObservedLocation {
            area_id: raw_id.to_string(),
            name: name.to_string(),
            act: known
                .map(|area| area.act)
                .unwrap_or(if side.is_some() { 1 } else { 0 }),
            is_town: known.is_some_and(|area| area.is_town_area),
            route_area_id: route_id.clone(),
        };
        self.pending_town_return = None;
        if let Some(id) = route_id {
            let repeated_town = location.is_town
                && self.engine.steps()[..self.engine.cursor()]
                    .iter()
                    .any(|step| step.area_context == id);
            let planned_return = self.engine.focus_area() == self.engine.active_area()
                && self.engine.next_transition_area() == Some(id.as_str());
            if repeated_town && self.engine.active_area() != Some(id.as_str()) {
                // A shopping trip and a post-boss return have identical log
                // evidence. Preserve the field frontier until explicit input.
                if planned_return {
                    self.pending_town_return = Some(id);
                }
            } else {
                self.engine.on_area_entered(&id, new_instance);
            }
        }
        self.observed_location = Some(location);
        self.seen_area_event = true;
    }

    fn side_guidance_for(&self, id: &str, name: &str) -> Option<&content::poe2::SideAreaGuidance> {
        // Never apply campaign-name hints to an endgame/hideout label.
        if !self.poe2_mode || !id.starts_with("G1_") {
            return None;
        }
        self.side_area_guidance
            .iter()
            .find(|side| side.aliases.iter().any(|alias| alias == name))
    }

    pub fn has_pending_town_return(&self) -> bool {
        self.pending_town_return.is_some()
            && self.observed_location.as_ref().is_some_and(|location| {
                location.is_town
                    && self.tracker.current_area_id() == Some(location.area_id.as_str())
            })
    }

    pub fn confirm_town_return(&mut self) -> Result<UiModel, String> {
        if !self.has_pending_town_return() {
            return Err("No planned town return is awaiting confirmation.".to_string());
        }
        let id = self
            .pending_town_return
            .take()
            .ok_or("No planned town return is awaiting confirmation.")?;
        self.engine.on_area_entered(&id, false);
        Ok(self.current_model())
    }

    pub fn current_model(&mut self) -> UiModel {
        let player_level = self.tracker.player_level();
        let build_ctx = self
            .build
            .as_ref()
            .map(|plan| BuildContext { plan, player_level });
        let mut overlay = compose(
            &self.engine,
            &self.tasks,
            &self.layouts,
            &self.areas,
            build_ctx,
        );
        if self.poe2_mode {
            let location = self.observed_location.as_ref();
            let showing_route = location
                .and_then(|loc| loc.route_area_id.as_deref())
                .is_some_and(|id| self.engine.focus_area() == Some(id));
            // This is an Act 1 preview, not proof of full campaign completion.
            overlay.route_complete = false;
            overlay.zone_name = location.map(|loc| loc.name.clone()).unwrap_or_default();
            overlay.area_id = location.map(|loc| loc.area_id.clone()).unwrap_or_default();
            overlay.act = location.map(|loc| loc.act).unwrap_or(0);
            overlay.is_town = location.is_some_and(|loc| loc.is_town);
            if showing_route {
                overlay.sub_hints.push(
                    "Act 1 preview · objectives are guidance, not confirmed completion."
                        .to_string(),
                );
            } else {
                overlay.primary = if self.pending_town_return.is_some() {
                    "Town visit — campaign progress unchanged. Complete the previous zone's objectives before confirming this return.".to_string()
                } else if location.is_some_and(|loc| loc.is_town) {
                    "Town visit — campaign progress unchanged.".to_string()
                } else {
                    "Campaign guidance is not available for this area.".to_string()
                };
                overlay.steps_in_zone.clear();
                overlay.sub_hints.clear();
                overlay.next_zone = None;
                overlay.off_route_zone = None;
                overlay.location_status = "on_track".to_string();
                overlay.groups_behind = 0;
                if let Some(side) =
                    location.and_then(|loc| self.side_guidance_for(&loc.area_id, &loc.name))
                {
                    overlay.primary = side.objectives[0].clone();
                    overlay.steps_in_zone = side.objectives.clone();
                    overlay.sub_hints = vec![format!(
                        "Act 1 side area · parent route: {}. Objectives are guidance, not confirmed completion.",
                        side.parent
                    )];
                    overlay.next_zone = Some(side.parent.clone());
                }
            }
        }
        let images = overlay
            .layout_images
            .iter()
            .map(|iv| UiImage {
                file: iv.file.clone(),
                stale: iv.stale,
                data_url: self.data_url_for(&iv.file),
            })
            .collect();
        UiModel {
            overlay,
            images,
            waiting_for_log: !self.seen_area_event,
            pending_town_return: self.has_pending_town_return(),
            build_summary: self.build.as_ref().map(build_summary),
        }
    }

    fn data_url_for(&mut self, file: &str) -> String {
        if let Some(hit) = self.encoded.get(file) {
            return hit.clone();
        }
        let url = if !is_safe_asset_name(file) {
            String::new()
        } else {
            let path = layouts_dir().join("assets").join(file);
            match std::fs::read(&path) {
                Ok(bytes) => format!(
                    "data:image/png;base64,{}",
                    base64::engine::general_purpose::STANDARD.encode(bytes)
                ),
                Err(_) => String::new(),
            }
        };
        self.encoded.insert(file.to_string(), url.clone());
        url
    }

    #[cfg(test)]
    pub(crate) fn encoded_cache_len(&self) -> usize {
        self.encoded.len()
    }
}

/// True when `file` is exactly one normal path component — no `..`, no
/// root/prefix (absolute paths, UNC/`\\host\share`), no separators splitting
/// it into multiple components, and not empty. Guards `data_url_for` against
/// a content-pack-supplied `file` value escaping the layouts `assets/`
/// directory: `PathBuf::join` with an absolute path REPLACES the base
/// instead of nesting under it, so an unvalidated `file` could otherwise
/// read arbitrary local files.
///
/// The explicit `contains('\\')` check matters beyond what `Component`
/// parsing alone gives us: `\` is only a path separator on Windows (the
/// actual shipping target, via the NSIS installer), so on Unix
/// `Path::new("\\\\host\\share").components()` yields a single
/// `Normal(..)` component — `Component` parsing alone would treat a UNC-
/// style string as "safe" on non-Windows build/test hosts. Rejecting any
/// backslash outright keeps this check's result platform-independent.
fn is_safe_asset_name(file: &str) -> bool {
    if file.contains('\\') {
        return false;
    }
    let mut comps = Path::new(file).components();
    matches!(
        (comps.next(), comps.next()),
        (Some(Component::Normal(_)), None)
    )
}

/// "Ranger (Deadeye) — 12 milestones", or "Ranger — 12 milestones" when the
/// build has no ascendancy recorded yet.
fn build_summary(plan: &pob_import::LevelingBuildPlan) -> String {
    let count = plan.milestones.len();
    match &plan.ascend_name {
        Some(ascend) => format!("{} ({ascend}) — {count} milestones", plan.class_name),
        None => format!("{} — {count} milestones", plan.class_name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GEN_STRAND: &str = r#"2026/07/21 19:00:01 1000 1186a0e0 [DEBUG Client 900] Generating level 1 area "1_1_1" with seed 101"#;
    const ENTER_STRAND: &str = "2026/07/21 19:00:03 3000 f22b6b26 [INFO Client 900] : You have entered The Twilight Strand.";
    const GEN_COAST: &str = r#"2026/07/21 19:04:00 240000 1186a0e0 [DEBUG Client 900] Generating level 2 area "1_1_2" with seed 8801"#;
    const ENTER_COAST: &str =
        "2026/07/21 19:04:02 242000 f22b6b26 [INFO Client 900] : You have entered The Coast.";
    const GEN_TOWN: &str = r#"2026/07/21 19:03:20 200000 1186a0e0 [DEBUG Client 900] Generating level 1 area "1_1_town" with seed 555"#;
    const ENTER_TOWN: &str =
        "2026/07/21 19:03:22 202000 f22b6b26 [INFO Client 900] : You have entered Lioneye's Watch.";

    #[test]
    fn initial_model_is_waiting() {
        let mut p = Pipeline::new(Variant::LeagueStart, None).unwrap();
        let m = p.current_model();
        assert!(m.waiting_for_log);
    }

    #[test]
    fn area_entry_produces_model_with_images() {
        let mut p = Pipeline::new(Variant::LeagueStart, None).unwrap();
        assert!(
            p.on_line(GEN_STRAND).is_none(),
            "generating alone changes nothing"
        );
        let m = p.on_line(ENTER_STRAND).expect("entered -> model");
        assert!(!m.waiting_for_log);
        assert_eq!(m.overlay.zone_name, "The Twilight Strand");

        p.on_line(GEN_TOWN);
        p.on_line(ENTER_TOWN);
        p.on_line(GEN_COAST);
        let m = p.on_line(ENTER_COAST).unwrap();
        assert_eq!(m.overlay.zone_name, "The Coast");
        assert!(!m.images.is_empty(), "The Coast has layout images");
        assert!(m.images[0].data_url.starts_with("data:image/png;base64,"));
        assert!(m.images[0].data_url.len() > 100);
    }

    #[test]
    fn non_area_lines_produce_none() {
        let mut p = Pipeline::new(Variant::LeagueStart, None).unwrap();
        assert!(p.on_line("garbage").is_none());
        assert!(p
            .on_line("2026/07/21 19:02:10 130000 f22b6b26 [INFO Client 900] : Wanderer (Ranger) is now level 2")
            .is_none());
    }

    #[test]
    fn data_urls_are_memoized() {
        let mut p = Pipeline::new(Variant::LeagueStart, None).unwrap();
        p.on_line(GEN_COAST);
        let m1 = p.on_line(ENTER_COAST).unwrap();
        // Re-enter: same encoded strings (pointer equality is not observable;
        // assert cache hit via identical output and cache length).
        p.on_line(GEN_COAST);
        let m2 = p.on_line(ENTER_COAST).unwrap();
        assert_eq!(m1.images[0].data_url, m2.images[0].data_url);
        assert_eq!(p.encoded_cache_len(), m1.images.len());
    }

    fn hand_built_plan() -> pob_import::LevelingBuildPlan {
        pob_import::LevelingBuildPlan {
            class_name: "Ranger".into(),
            ascend_name: Some("Deadeye".into()),
            skill_sets: vec![],
            passive_spec_titles: vec![],
            notes: None,
            milestones: vec![pob_import::Milestone {
                level: 2,
                label: "Gem available: Frostblink".into(),
                reliability: pob_import::Reliability::Structured,
            }],
            reliability: pob_import::Reliability::Structured,
        }
    }

    #[test]
    fn pipeline_with_build_surfaces_summary_and_town_reminders() {
        let mut p = Pipeline::new(Variant::LeagueStart, Some(hand_built_plan())).unwrap();

        // No area entered yet: still waiting, but the build summary is
        // already available (it doesn't depend on session state).
        let m = p.current_model();
        assert!(m.waiting_for_log);
        assert_eq!(
            m.build_summary.as_deref(),
            Some("Ranger (Deadeye) — 1 milestones")
        );

        // Pin the player's level (LevelUp lines alone never yield a model).
        assert!(
            p.on_line(
                "2026/07/21 19:02:10 130000 f22b6b26 [INFO Client 900] : Wanderer (Ranger) is now level 2"
            )
            .is_none()
        );

        // Enter town: the milestone (level 2) is within the reminder window
        // for a level-2 player.
        p.on_line(GEN_TOWN);
        let m = p.on_line(ENTER_TOWN).expect("entered town -> model");
        assert_eq!(
            m.build_summary.as_deref(),
            Some("Ranger (Deadeye) — 1 milestones")
        );
        assert_eq!(
            m.overlay.build_reminders,
            vec!["Gem available: Frostblink".to_string()]
        );
    }

    #[test]
    fn pipeline_without_build_has_no_summary() {
        let mut p = Pipeline::new(Variant::LeagueStart, None).unwrap();
        let m = p.current_model();
        assert_eq!(m.build_summary, None);
    }

    #[test]
    fn is_safe_asset_name_rejects_path_traversal_and_absolute_paths() {
        assert!(!is_safe_asset_name("../../etc/passwd"));
        assert!(!is_safe_asset_name("/etc/passwd"));
        assert!(!is_safe_asset_name("a/b.png"));
        assert!(!is_safe_asset_name("\\\\host\\share"));
        assert!(!is_safe_asset_name(""));
    }

    #[test]
    fn is_safe_asset_name_accepts_a_plain_filename() {
        assert!(is_safe_asset_name("image10.png"));
    }

    #[test]
    fn data_url_for_rejects_traversal_and_yields_empty_string() {
        let mut p = Pipeline::new(Variant::LeagueStart, None).unwrap();
        for unsafe_name in ["../../etc/passwd", "/etc/passwd", "a/b.png", ""] {
            assert_eq!(p.data_url_for(unsafe_name), String::new());
        }
    }
}

#[cfg(test)]
mod poe2_tests {
    use super::*;

    fn line(tick: u64, message: &str) -> String {
        format!("2026/10/24 12:00:00 {tick} abc [INFO Client 100] {message}")
    }
    fn travel(p: &mut Pipeline, tick: u64, id: &str, name: &str) -> UiModel {
        p.on_line(&line(
            tick,
            &format!("Generating level 4 area \"{id}\" with seed 7"),
        ));
        p.on_line(&line(tick, &format!("[SCENE] Set Source [{name}]")));
        p.on_line(&line(
            tick + 1,
            &format!("[LOADING SCREEN] ({name}) Duration = 1 seconds"),
        ))
        .unwrap()
    }

    #[test]
    fn poe2_shows_act1_objectives_only_after_confirmed_loading() {
        let mut p = Pipeline::new_poe2().unwrap();
        assert!(p.current_model().waiting_for_log);
        p.on_line(&line(1, "Generating level 4 area \"G1_4\" with seed 7"));
        assert!(p.current_model().waiting_for_log);
        let m = p
            .on_line(&line(
                2,
                "[LOADING SCREEN] (The Grelwood) Duration = 1 seconds",
            ))
            .unwrap();
        assert!(!m.waiting_for_log);
        assert_eq!(m.overlay.zone_name, "The Grelwood");
        assert_eq!(m.overlay.area_id, "G1_4");
        assert_eq!(m.overlay.act, 1);
        assert!(
            m.overlay
                .steps_in_zone
                .iter()
                .any(|step| step.contains("Una"))
        );
        assert!(m.images.is_empty());
        assert!(m.overlay.build_reminders.is_empty());
        assert!(!m.overlay.route_complete);
    }

    #[test]
    fn endgame_never_advances_campaign_or_shows_stale_campaign_objectives() {
        let mut p = Pipeline::new_poe2().unwrap();
        travel(&mut p, 1, "G1_4", "The Grelwood");
        let frontier = p.engine.cursor();
        let m = travel(&mut p, 10, "MapFuture", "The Grelwood");
        assert_eq!(p.engine.cursor(), frontier);
        assert_eq!(m.overlay.area_id, "MapFuture");
        assert_eq!(m.overlay.act, 0);
        assert!(m.overlay.steps_in_zone.is_empty());
        assert!(m.overlay.next_zone.is_none());
        assert!(!m.overlay.route_complete);
        assert!(!m.waiting_for_log);
    }

    #[test]
    fn log_session_changes_do_not_merge_character_route_progress() {
        let mut p = Pipeline::new_poe2().unwrap();
        travel(&mut p, 1, "G1_4", "The Grelwood");
        assert!(p.engine.cursor() > 0);
        p.on_line(&line(10, "Async connecting to example.invalid:21269"));
        let m = travel(&mut p, 20, "G1_1", "The Riverbank");
        assert_eq!(p.engine.cursor(), 0);
        assert_eq!(m.overlay.location_status, "on_track");
        assert!(m.overlay.primary.contains("Wounded Man"));
    }

    #[test]
    fn reset_and_journal_replay_preserve_poe2_semantics() {
        let lines = [
            line(1, "Generating level 4 area \"G1_4\" with seed 7"),
            line(2, "[LOADING SCREEN] (The Grelwood) Duration = 1 seconds"),
        ];
        let mut p = Pipeline::new_poe2().unwrap();
        for input in &lines {
            p.on_line(input);
        }
        let before = p.current_model().overlay;
        p.reset();
        assert!(p.current_model().waiting_for_log);
        for input in &lines {
            p.on_line(input);
        }
        assert_eq!(p.current_model().overlay, before);
    }
}

#[cfg(test)]
mod town_return_tests {
    use super::*;

    fn input(tick: u64, message: &str) -> String {
        format!("2026/10/24 12:00:00 {tick} abc [INFO Client 100] {message}")
    }
    fn travel(p: &mut Pipeline, tick: u64, id: &str, name: &str) -> UiModel {
        for message in [
            format!("Generating level 15 area \"{id}\" with seed 7"),
            format!("[SCENE] Set Source [{name}]"),
        ] {
            p.on_line(&input(tick, &message));
        }
        p.on_line(&input(
            tick + 1,
            &format!("[LOADING SCREEN] ({name}) Duration = 1 seconds"),
        ))
        .unwrap()
    }
    // The log sample has no Manor generation. This synthetic fixture uses an
    // explicit test ID, resolved to the guide name only after scene+load evidence.
    const MANOR: &str = "G1_test_manor";

    #[test]
    fn early_manor_town_return_does_not_advance_or_complete_and_return_restores_geonor() {
        let mut p = Pipeline::new_poe2().unwrap();
        travel(&mut p, 1, MANOR, "Ogham Manor");
        let frontier = p.engine.cursor();
        let town = travel(&mut p, 10, "G1_town", "Clearfell Encampment");
        assert_eq!(town.overlay.zone_name, "Clearfell Encampment");
        assert!(town.pending_town_return);
        assert_eq!(p.engine.cursor(), frontier);
        assert!(!p.engine.is_complete());
        assert!(town.overlay.primary.contains("progress unchanged"));
        let manor = travel(&mut p, 20, MANOR, "Ogham Manor");
        assert_eq!(p.engine.cursor(), frontier);
        assert!(!manor.pending_town_return);
        assert!(
            manor
                .overlay
                .steps_in_zone
                .iter()
                .any(|step| step.contains("Geonor"))
        );
        assert!(!manor.overlay.route_complete);
        assert!(p.confirm_town_return().is_err());
    }

    #[test]
    fn explicit_confirmation_retains_final_town_instructions_and_is_one_time() {
        let mut p = Pipeline::new_poe2().unwrap();
        travel(&mut p, 1, MANOR, "Ogham Manor");
        travel(&mut p, 10, "G1_town", "Clearfell Encampment");
        let model = p.confirm_town_return().unwrap();
        assert!(!model.pending_town_return);
        assert!(
            model
                .overlay
                .steps_in_zone
                .iter()
                .any(|step| step.contains("Follow the Beast"))
        );
        assert!(
            !p.engine.is_complete(),
            "arrival must not automatically finish the final objectives"
        );
        assert!(p.confirm_town_return().is_err());
    }

    #[test]
    fn initial_town_entry_remains_automatic_but_repeated_planned_returns_require_input() {
        let mut p = Pipeline::new_poe2().unwrap();
        travel(&mut p, 1, "G1_1", "The Riverbank");
        let initial = travel(&mut p, 10, "G1_town", "Clearfell Encampment");
        assert!(!initial.pending_town_return);
        assert!(initial.overlay.primary.contains("Renly"));
        travel(&mut p, 20, "G1_5", "The Red Vale");
        let frontier = p.engine.cursor();
        let town = travel(&mut p, 30, "G1_town", "Clearfell Encampment");
        assert!(town.pending_town_return);
        assert_eq!(p.engine.cursor(), frontier);
        let model = p.confirm_town_return().unwrap();
        assert!(model.overlay.primary.contains("Runed Spikes"));
    }

    #[test]
    fn confirmation_restores_only_from_the_app_journal_not_raw_game_lines() {
        let mut p = Pipeline::new_poe2().unwrap();
        let messages = [
            input(
                1,
                &format!("Generating level 15 area \"{MANOR}\" with seed 7"),
            ),
            input(1, "[SCENE] Set Source [Ogham Manor]"),
            input(2, "[LOADING SCREEN] (Ogham Manor) Duration = 1 seconds"),
            input(10, "Generating level 15 area \"G1_town\" with seed 7"),
            input(
                11,
                "[LOADING SCREEN] (Clearfell Encampment) Duration = 1 seconds",
            ),
        ];
        for line in &messages {
            p.on_line(line);
        }
        assert!(p.has_pending_town_return());
        p.on_line(crate::journal::CONFIRM_TOWN_RETURN);
        assert!(
            p.has_pending_town_return(),
            "game logs cannot manufacture user acknowledgment"
        );
        let mut journal = messages.to_vec();
        journal.push(crate::journal::CONFIRM_TOWN_RETURN.to_string());
        let mut restored = Pipeline::new_poe2().unwrap();
        crate::journal::replay_into(&mut restored, &journal);
        assert!(!restored.has_pending_town_return());
        assert!(
            restored
                .current_model()
                .overlay
                .primary
                .contains("Talk to all NPCs")
        );
    }
}

#[cfg(test)]
mod incidental_town_tests {
    use super::*;
    #[test]
    fn shopping_trip_before_a_field_step_never_skips_ahead_to_a_future_town_visit() {
        let mut p = Pipeline::new_poe2().unwrap();
        let mut tick = 0;
        for (id, name) in [
            ("G1_4", "The Grelwood"),
            ("G1_town", "Clearfell Encampment"),
            ("G1_4", "The Grelwood"),
        ] {
            for msg in [
                format!("Generating level 4 area \"{id}\" with seed 7"),
                format!("[SCENE] Set Source [{name}]"),
                format!("[LOADING SCREEN] ({name}) Duration = 1 seconds"),
            ] {
                tick += 1;
                p.on_line(&format!(
                    "2026/10/24 12:00:00 {tick} abc [INFO Client 100] {msg}"
                ));
            }
            assert_eq!(p.engine.active_area(), Some("G1_4"));
            assert!(!p.has_pending_town_return());
        }
        assert!(p.current_model().overlay.primary.contains("Una"));
    }
}

#[cfg(test)]
mod side_area_tests {
    use super::*;
    fn travel(p: &mut Pipeline, tick: u64, id: &str, name: &str) -> UiModel {
        for message in [
            format!("Generating level 8 area \"{id}\" with seed 7"),
            format!("[SCENE] Set Source [{name}]"),
        ] {
            p.on_line(&format!(
                "2026/10/24 12:00:00 {tick} abc [INFO Client 100] {message}"
            ));
        }
        p.on_line(&format!("2026/10/24 12:00:00 {} abc [INFO Client 100] [LOADING SCREEN] ({name}) Duration = 1 seconds",tick+1)).unwrap()
    }
    #[test]
    fn cemetery_side_areas_display_their_objectives_without_consuming_parent_progress() {
        for (id, name, boss) in [
            ("G1_8", "Mausoleum of the Praetor", "Draven"),
            ("G1_9", "Tomb of the Consort", "Asinia"),
        ] {
            let mut p = Pipeline::new_poe2().unwrap();
            let parent = travel(&mut p, 1, "G1_7", "Cemetery of the Eternals");
            let cursor = p.engine.cursor();
            let side = travel(&mut p, 10, id, name);
            assert_eq!(side.overlay.area_id, id);
            assert_eq!(side.overlay.zone_name, name);
            assert_eq!(side.overlay.act, 1);
            assert!(side.overlay.primary.contains(boss));
            assert_eq!(
                side.overlay.next_zone.as_deref(),
                Some("Cemetery of the Eternals")
            );
            assert_eq!(p.engine.cursor(), cursor);
            assert!(!side.pending_town_return);
            assert!(!side.overlay.route_complete);
            let returned = travel(&mut p, 20, "G1_7", "Cemetery of the Eternals");
            assert_eq!(returned.overlay.steps_in_zone, parent.overlay.steps_in_zone);
            assert_eq!(p.engine.cursor(), cursor);
        }
    }
    #[test]
    fn mud_burrow_uses_name_evidence_without_inventing_a_game_id() {
        for name in ["Mud Burrow", "The Mud Burrow"] {
            let mut p = Pipeline::new_poe2().unwrap();
            travel(&mut p, 1, "G1_2", "Clearfell");
            let cursor = p.engine.cursor();
            // Synthetic ID: the supplied log did not capture Mud Burrow.
            let side = travel(&mut p, 10, "G1_test_mud", name);
            assert_eq!(side.overlay.area_id, "G1_test_mud");
            assert!(side.overlay.primary.contains("Devourer"));
            assert!(side.overlay.primary.contains("(Opt)"));
            assert_eq!(side.overlay.next_zone.as_deref(), Some("Clearfell"));
            assert_eq!(p.engine.cursor(), cursor);
        }
    }
    #[test]
    fn a_matching_endgame_label_cannot_select_campaign_side_area_guidance() {
        let mut p = Pipeline::new_poe2().unwrap();
        let model = travel(&mut p, 1, "MapFuture", "Tomb of the Consort");
        assert_eq!(model.overlay.act, 0);
        assert!(!model.overlay.primary.contains("Asinia"));
        assert!(model.overlay.next_zone.is_none());
        assert_eq!(p.engine.cursor(), 0);
    }
}
