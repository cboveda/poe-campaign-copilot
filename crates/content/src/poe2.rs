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
