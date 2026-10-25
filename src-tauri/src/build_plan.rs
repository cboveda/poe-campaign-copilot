//! Provider-neutral saved build stages. Imported text is data, never executable.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildItem {
    pub id: String,
    pub category: String,
    pub text: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildStage {
    pub id: String,
    pub title: String,
    pub acts: Vec<u8>,
    pub items: Vec<BuildItem>,
    pub notes: Vec<String>,
    pub passive_nodes: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildPlan {
    pub format_version: u32,
    pub source_url: String,
    pub title: String,
    pub author: String,
    pub source_updated: String,
    pub imported_at: u64,
    pub stages: Vec<BuildStage>,
    pub skipped_stages: Vec<String>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildProgress {
    #[serde(default)]
    pub stage_override: Option<String>,
    #[serde(default)]
    pub completed: BTreeSet<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SavedBuild {
    pub plan: Option<BuildPlan>,
    pub profile: String,
    pub profiles: BTreeMap<String, BuildProgress>,
}
impl Default for SavedBuild {
    fn default() -> Self {
        Self {
            plan: None,
            profile: "Current character".into(),
            profiles: BTreeMap::new(),
        }
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct BuildView {
    pub title: String,
    pub stage: String,
    pub pending: Vec<String>,
    pub completed_count: usize,
    pub total_count: usize,
}
impl SavedBuild {
    pub fn selected_stage(&self, act: Option<u8>) -> Option<&BuildStage> {
        let plan = self.plan.as_ref()?;
        let manual = self
            .profiles
            .get(&self.profile)
            .and_then(|p| p.stage_override.as_deref());
        if let Some(id) = manual {
            return plan.stages.iter().find(|s| s.id == id);
        }
        let act = act?;
        let mut candidates = plan.stages.iter().filter(|s| s.acts.contains(&act));
        let first = candidates.next()?;
        candidates.next().is_none().then_some(first)
    }
    pub fn key(stage: &str, item: &str) -> String {
        format!("{stage}/{item}")
    }
    pub fn view(&self, act: Option<u8>) -> Option<BuildView> {
        let stage = self.selected_stage(act)?;
        let done = self.profiles.get(&self.profile).map(|p| &p.completed);
        let pending: Vec<_> = stage
            .items
            .iter()
            .filter(|item| !done.is_some_and(|d| d.contains(&Self::key(&stage.id, &item.id))))
            .map(|item| format!("{}: {}", item.category, item.text))
            .collect();
        Some(BuildView {
            title: self.plan.as_ref()?.title.clone(),
            stage: stage.title.clone(),
            completed_count: stage.items.len() - pending.len(),
            total_count: stage.items.len(),
            pending: pending.into_iter().take(4).collect(),
        })
    }
    pub fn replace(&mut self, plan: BuildPlan) {
        // New snapshots must never silently inherit completion against changed targets.
        self.plan = Some(plan);
        self.profiles.clear();
    }
    pub fn set_profile(&mut self, profile: &str, stage: Option<String>) -> Result<(), String> {
        let profile = profile.trim();
        if profile.is_empty() || profile.len() > 100 {
            return Err("Enter a character/profile name (1–100 bytes).".into());
        }
        if let Some(id) = &stage
            && !self
                .plan
                .as_ref()
                .is_some_and(|p| p.stages.iter().any(|s| &s.id == id))
        {
            return Err("Unknown build stage.".into());
        }
        if !self.profiles.contains_key(profile) && self.profiles.len() >= 100 {
            return Err("Maximum of 100 character profiles reached.".into());
        }
        self.profile = profile.into();
        self.profiles
            .entry(self.profile.clone())
            .or_default()
            .stage_override = stage;
        Ok(())
    }
    pub fn complete(&mut self, stage: &str, item: &str, done: bool) -> Result<(), String> {
        if !self.plan.as_ref().is_some_and(|p| {
            p.stages
                .iter()
                .any(|s| s.id == stage && s.items.iter().any(|i| i.id == item))
        }) {
            return Err("Unknown build checklist item.".into());
        }
        let progress = self.profiles.entry(self.profile.clone()).or_default();
        let key = Self::key(stage, item);
        if done {
            progress.completed.insert(key);
        } else {
            progress.completed.remove(&key);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn plan() -> BuildPlan {
        BuildPlan {
            format_version: 1,
            source_url: String::new(),
            title: "Example".into(),
            author: String::new(),
            source_updated: String::new(),
            imported_at: 0,
            skipped_stages: vec![],
            stages: vec![BuildStage {
                id: "a".into(),
                title: "ACT 1".into(),
                acts: vec![1],
                items: vec![BuildItem {
                    id: "skill".into(),
                    category: "Skill".into(),
                    text: "Grenade".into(),
                }],
                notes: vec![],
                passive_nodes: vec![],
            }],
        }
    }
    #[test]
    fn selection_is_explicit_and_completion_is_per_character_and_reversible() {
        let mut saved = SavedBuild::default();
        saved.replace(plan());
        assert!(saved.view(None).is_none());
        assert!(saved.view(Some(2)).is_none());
        saved.set_profile("Alice", None).unwrap();
        saved.complete("a", "skill", true).unwrap();
        assert_eq!(saved.view(Some(1)).unwrap().completed_count, 1);
        saved.set_profile("Bob", Some("a".into())).unwrap();
        assert_eq!(saved.view(None).unwrap().completed_count, 0);
        saved.set_profile("Alice", None).unwrap();
        saved.complete("a", "skill", false).unwrap();
        assert_eq!(saved.view(Some(1)).unwrap().completed_count, 0);
        assert!(saved.complete("a", "forged", true).is_err());
        saved.complete("a", "skill", true).unwrap();
        saved.replace(plan());
        assert_eq!(saved.view(Some(1)).unwrap().completed_count, 0);
    }
}
