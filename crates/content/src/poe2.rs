//! PoE2 observations, kept separate from the legacy PoE1 vendor data.
//! This is a partial catalog from a supplied log, not full campaign data.
use crate::game_data::{AreaMap, GameDataError, load_areas};

pub fn load_observed_areas() -> Result<AreaMap, GameDataError> {
    Ok(load_areas(include_str!(
        "../../../content/poe2/observed-areas.json"
    ))?)
}

use crate::game_data::Area;
use crate::route_dsl::Fragment;
use crate::walk::CompiledStep;
use serde::Deserialize;

#[derive(Deserialize)]
struct Act1Route {
    format_version: u32,
    game: String,
    coverage: String,
    visits: Vec<Visit>,
    #[serde(default)]
    side_areas: Vec<SideAreaGuidance>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SideAreaGuidance {
    pub name: String,
    pub aliases: Vec<String>,
    pub parent: String,
    pub source_steps: Vec<u16>,
    pub objectives: Vec<String>,
}

/// Supplemental objectives do not become route steps or inferred game IDs.
pub fn load_side_area_guidance() -> Result<Vec<SideAreaGuidance>, GameDataError> {
    let route: Act1Route =
        serde_json::from_str(include_str!("../../../content/poe2/act1-route.json"))?;
    let mut names = std::collections::BTreeSet::new();
    for side in &route.side_areas {
        if side.name.trim().is_empty()
            || side.aliases.is_empty()
            || side.objectives.is_empty()
            || side.source_steps.len() != side.objectives.len()
            || !route.visits.iter().any(|visit| visit.name == side.parent)
            || side
                .aliases
                .iter()
                .any(|name| name.trim().is_empty() || !names.insert(name.clone()))
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid PoE2 side-area guidance",
            )
            .into());
        }
    }
    Ok(route.side_areas)
}

#[derive(Deserialize)]
struct Visit {
    name: String,
    source_heading: String,
    objectives: Vec<String>,
}

/// The preview uses observed game IDs where established; otherwise internal
/// `poe2-guide:` keys identify guide names, never purported game identifiers.
/// Matching these entries in the application is by exact loading-screen name.
pub fn load_act1_preview() -> Result<(AreaMap, Vec<CompiledStep>), GameDataError> {
    let route: Act1Route =
        serde_json::from_str(include_str!("../../../content/poe2/act1-route.json"))?;
    if route.format_version != 1 || route.game != "poe2" || route.coverage != "act1-preview" {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "unsupported PoE2 route pack",
        )
        .into());
    }
    let mut areas = load_observed_areas()?;
    let mut steps = Vec::new();
    for (visit_index, visit) in route.visits.into_iter().enumerate() {
        if visit.name.trim().is_empty() || visit.objectives.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "empty PoE2 route visit",
            )
            .into());
        }
        let id = areas
            .values()
            .find(|area| area.name == visit.name)
            .map(|area| area.id.clone())
            .unwrap_or_else(|| {
                format!(
                    "poe2-guide:act1:{}",
                    visit.name.to_lowercase().replace(' ', "-")
                )
            });
        areas.entry(id.clone()).or_insert_with(|| Area {
            id: id.clone(),
            name: visit.name,
            act: 1,
            level: None,
            has_waypoint: false,
            is_town_area: false,
            parent_town_area_id: None,
            connection_ids: Vec::new(),
            crafting_recipes: Vec::new(),
        });
        for (index, text) in visit.objectives.into_iter().enumerate() {
            steps.push(CompiledStep {
                id: format!("poe2-a1-v{visit_index}-s{index}"),
                act: 1,
                section: visit.source_heading.clone(),
                area_context: id.clone(),
                fragments: vec![Fragment::Text { value: text }],
                sub_steps: Vec::new(),
            });
        }
    }
    Ok((areas, steps))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn act1_preview_has_stable_references_and_permanent_bonus_objectives() {
        let (areas, steps) = load_act1_preview().unwrap();
        assert_eq!(steps.len(), 65);
        assert_eq!(steps[0].area_context, "G1_1");
        assert_eq!(steps[0].act, 1);
        assert!(
            steps
                .iter()
                .all(|step| areas.contains_key(&step.area_context))
        );
        let ids: std::collections::BTreeSet<_> = steps.iter().map(|step| &step.id).collect();
        assert_eq!(ids.len(), steps.len());
        let text = format!("{steps:?}");
        for objective in ["Beira", "Crowbell", "Una's Lute", "Candlemass", "Geonor"] {
            assert!(
                text.contains(objective),
                "missing permanent reward/story objective: {objective}"
            );
        }
        assert!(
            !text.contains("Frost bomb"),
            "build-specific notes must not become route objectives"
        );
    }
}

#[cfg(test)]
mod side_area_tests {
    use super::*;
    #[test]
    fn supplemental_objectives_are_sourced_without_inserting_route_visits() {
        let sides = load_side_area_guidance().unwrap();
        assert_eq!(sides.len(), 3);
        assert_eq!(load_act1_preview().unwrap().1.len(), 65);
        assert_eq!(sides[0].source_steps, [9]);
        assert!(sides[0].objectives[0].contains("Devourer"));
        assert_eq!(sides[1].source_steps, [33, 34]);
        assert!(sides[1].objectives[0].contains("Draven"));
        assert_eq!(sides[2].source_steps, [35]);
        assert!(sides[2].objectives[0].contains("Asinia"));
    }
}

#[derive(Deserialize)]
struct RemainingCampaign {
    format_version: u32,
    game: String,
    target_patch: String,
    chapters: Vec<Chapter>,
}
#[derive(Deserialize)]
struct Chapter {
    id: String,
    label: String,
    act: u8,
    mode: String,
    visits: Vec<CampaignVisit>,
    #[serde(default)]
    context_objectives: Vec<String>,
}
#[derive(Deserialize)]
struct CampaignVisit {
    #[serde(default)]
    parent: Option<String>,
    name: String,
    aliases: Vec<String>,
    source_heading: String,
    is_town: bool,
    objectives: Vec<String>,
}
#[derive(Debug, Clone)]
pub struct LocalGuidance {
    pub area_id: String,
    pub return_to: Option<String>,
    pub label: String,
    pub objectives: Vec<String>,
    pub context_objectives: Vec<String>,
}
pub struct CampaignPreview {
    pub areas: AreaMap,
    pub steps: Vec<CompiledStep>,
    pub aliases: std::collections::BTreeMap<String, Vec<String>>,
    pub local_guidance: Vec<LocalGuidance>,
}

/// Names are compared exactly except for leading The and apostrophe style.
/// No substring/fuzzy match and no fabricated game identifier is involved.
pub fn normalize_area_name(name: &str) -> String {
    let name = name.trim().replace('\u{2019}', "'").to_lowercase();
    name.strip_prefix("the ").unwrap_or(&name).to_string()
}

/// Candidate campaign-style namespace; a known guide name must also match.
/// The number is only an ambiguity hint;
/// chapter metadata comes from the matched source area, not an ID guess.
pub fn campaign_namespace(id: &str) -> Option<u8> {
    let (number, rest) = id.strip_prefix('G')?.split_once('_')?;
    let number = number.parse::<u8>().ok()?;
    (number > 0 && !rest.is_empty()).then_some(number)
}

pub fn load_campaign_preview() -> Result<CampaignPreview, GameDataError> {
    let remaining: RemainingCampaign = serde_json::from_str(include_str!(
        "../../../content/poe2/remaining-campaign.json"
    ))?;
    if remaining.format_version != 1
        || remaining.game != "poe2"
        || remaining.target_patch != "0.5.5"
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "unsupported remaining campaign pack",
        )
        .into());
    }
    let (mut areas, mut steps) = load_act1_preview()?;
    let mut aliases: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    for area in areas.values() {
        aliases
            .entry(normalize_area_name(&area.name))
            .or_default()
            .push(area.id.clone());
    }
    let mut local_guidance: Vec<LocalGuidance> = Vec::new();
    for chapter in remaining.chapters {
        if !(2..=5).contains(&chapter.act)
            || !["sequential", "area_guidance"].contains(&chapter.mode.as_str())
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid campaign chapter",
            )
            .into());
        }
        for (visit_index, visit) in chapter.visits.into_iter().enumerate() {
            if visit.name.trim().is_empty()
                || visit.objectives.is_empty()
                || visit.aliases.is_empty()
            {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "empty campaign visit",
                )
                .into());
            }
            let area_id = areas
                .values()
                .find(|area| {
                    area.act == chapter.act
                        && normalize_area_name(&area.name) == normalize_area_name(&visit.name)
                })
                .map(|area| area.id.clone())
                .unwrap_or_else(|| {
                    format!(
                        "poe2-guide:{}:{}",
                        chapter.id,
                        normalize_area_name(&visit.name).replace(' ', "-")
                    )
                });
            let area = areas.entry(area_id.clone()).or_insert_with(|| Area {
                id: area_id.clone(),
                name: visit.name.clone(),
                act: chapter.act,
                level: None,
                has_waypoint: false,
                is_town_area: visit.is_town,
                parent_town_area_id: None,
                connection_ids: Vec::new(),
                crafting_recipes: Vec::new(),
            });
            area.is_town_area |= visit.is_town;
            for alias in visit.aliases.iter().chain(std::iter::once(&visit.name)) {
                let ids = aliases.entry(normalize_area_name(alias)).or_default();
                if !ids.contains(&area_id) {
                    ids.push(area_id.clone());
                }
            }
            if chapter.mode == "sequential" {
                for (index, text) in visit.objectives.into_iter().enumerate() {
                    steps.push(CompiledStep {
                        id: format!("poe2-{}-v{visit_index}-s{index}", chapter.id),
                        act: chapter.act,
                        section: visit.source_heading.clone(),
                        area_context: area_id.clone(),
                        fragments: vec![Fragment::Text { value: text }],
                        sub_steps: Vec::new(),
                    });
                }
            } else if let Some(existing) = local_guidance
                .iter_mut()
                .find(|guide| guide.area_id == area_id)
            {
                existing.objectives.extend(visit.objectives);
            } else {
                local_guidance.push(LocalGuidance {
                    area_id,
                    return_to: visit.parent,
                    label: chapter.label.clone(),
                    objectives: visit.objectives,
                    context_objectives: chapter.context_objectives.clone(),
                });
            }
        }
    }
    Ok(CampaignPreview {
        areas,
        steps,
        aliases,
        local_guidance,
    })
}

#[cfg(test)]
mod campaign_tests {
    use super::*;
    #[test]
    fn remaining_campaign_has_referenced_unique_sequential_steps_and_branch_guidance() {
        let pack = load_campaign_preview().unwrap();
        let counts: Vec<_> = (1..=3)
            .map(|act| pack.steps.iter().filter(|step| step.act == act).count())
            .collect();
        assert_eq!(counts, [65, 78, 79]);
        let ids: std::collections::BTreeSet<_> = pack.steps.iter().map(|step| &step.id).collect();
        assert_eq!(ids.len(), pack.steps.len());
        assert!(
            pack.steps
                .iter()
                .all(|step| pack.areas.contains_key(&step.area_context))
        );
        for label in ["Act 4", "Interlude 5.1", "Interlude 5.2", "Interlude 5.3"] {
            assert!(
                pack.local_guidance.iter().any(|guide| guide.label == label),
                "missing {label}"
            );
        }
        let text = format!("{:?} {:?}", pack.steps, pack.local_guidance);
        for objective in [
            "Jamanra",
            "Doryani",
            "Tavakai",
            "Warden's Ledger",
            "Molten One's Gift",
            "Gemcrust Skull",
        ] {
            assert!(text.contains(objective), "missing {objective}");
        }
        assert!(!text.contains("start your first map"));
        assert!(!text.contains("Endgame Atlas / Maps unlock"));
    }
    #[test]
    fn only_explicit_campaign_namespaces_are_accepted_and_aliases_do_not_fuzz() {
        for id in [
            "MapG4_test",
            "HideoutShip",
            "IncursionTemple",
            "G0_1",
            "G1_",
        ] {
            assert_eq!(campaign_namespace(id), None);
        }
        assert_eq!(campaign_namespace("G4_example"), Some(4));
        assert_eq!(campaign_namespace("G7_interlude"), Some(7));
        assert_eq!(
            normalize_area_name("The Ardura Caravan"),
            normalize_area_name("Ardura Caravan")
        );
        assert_ne!(
            normalize_area_name("Lost City"),
            normalize_area_name("Lost City Map")
        );
    }
}

/// Source-backed directions are independent of objectives and route progress.
#[derive(Debug, Clone, Deserialize)]
pub struct NavigationCue {
    pub act: u8,
    pub area: String,
    pub text: String,
    pub source_label: String,
    pub patch_caveat: String,
}

pub fn load_navigation() -> Result<Vec<NavigationCue>, GameDataError> {
    #[derive(Deserialize)]
    struct Pack {
        format_version: u32,
        game: String,
        target_patch: String,
        entries: Vec<NavigationCue>,
    }
    let pack: Pack = serde_json::from_str(include_str!("../../../content/poe2/navigation.json"))?;
    if pack.format_version != 1
        || pack.game != "poe2"
        || pack.target_patch != "0.5.5"
        || pack.entries.iter().any(|cue| {
            cue.act == 0
                || cue.area.trim().is_empty()
                || cue.text.trim().is_empty()
                || cue.source_label.trim().is_empty()
        })
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid PoE2 navigation pack",
        )
        .into());
    }
    Ok(pack.entries)
}
