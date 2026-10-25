//! Separate from campaign config/journal: build edits never reset route progress.
use crate::build_plan::SavedBuild;
use std::io::Read;
use tauri::Manager;
const MAX_SAVED: usize = 1024 * 1024;
fn path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    Ok(app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?
        .join("build.json"))
}
pub fn load(app: &tauri::AppHandle) -> SavedBuild {
    load_path(&match path(app) {
        Ok(path) => path,
        Err(_) => return SavedBuild::default(),
    })
    .unwrap_or_default()
}
pub fn load_path(path: &std::path::Path) -> Result<SavedBuild, String> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(MAX_SAVED as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > MAX_SAVED {
        return Err("saved build too large".into());
    }
    let saved: SavedBuild = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if saved.profile.trim().is_empty() || saved.profile.len() > 100 || saved.profiles.len() > 100 {
        return Err("invalid saved build profile".into());
    }
    if let Some(plan) = &saved.plan
        && (plan.format_version != 1
            || plan.stages.is_empty()
            || plan.stages.len() > 32
            || crate::mobalytics::canonical_url(&plan.source_url).is_err())
    {
        return Err("unsupported saved build".into());
    }
    Ok(saved)
}
pub fn store(app: &tauri::AppHandle, saved: &SavedBuild) -> Result<(), String> {
    store_path(&path(app)?, saved)
}
pub fn store_path(target: &std::path::Path, saved: &SavedBuild) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(saved).map_err(|e| e.to_string())?;
    if bytes.len() > MAX_SAVED {
        return Err("Saved build exceeds the 1 MiB limit.".into());
    }
    std::fs::create_dir_all(target.parent().ok_or("Invalid build storage path.")?)
        .map_err(|e| e.to_string())?;
    let temp = target.with_extension("json.tmp");
    std::fs::write(&temp, bytes).map_err(|e| format!("Could not save build: {e}"))?;
    std::fs::rename(&temp, target).map_err(|e| format!("Could not replace saved build: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn store_replaces_existing_snapshot_and_bad_input_does_not_overwrite_it() {
        let dir = std::env::temp_dir().join(format!(
            "poe-build-store-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = dir.join("build.json");
        let mut saved = SavedBuild::default();
        saved.replace(
            crate::mobalytics::parse_page(
                include_str!("../fixtures/mobalytics-build.html"),
                "https://mobalytics.gg/poe-2/builds/example-starter",
            )
            .unwrap(),
        );
        saved.set_profile("Alice", Some("0".into())).unwrap();
        saved.complete("0", "skill-0", true).unwrap();
        store_path(&path, &saved).unwrap();
        let restored = load_path(&path).unwrap();
        assert_eq!(restored.profile, "Alice");
        assert_eq!(restored.view(None).unwrap().completed_count, 1);
        assert_eq!(restored.plan.unwrap().stages.len(), 4);
        saved.set_profile("Bob", None).unwrap();
        store_path(&path, &saved).unwrap();
        assert_eq!(load_path(&path).unwrap().profile, "Bob");
        saved.profile = "x".repeat(MAX_SAVED + 1);
        assert!(store_path(&path, &saved).is_err());
        assert_eq!(load_path(&path).unwrap().profile, "Bob");
        std::fs::write(&path, b"broken JSON").unwrap();
        assert!(load_path(&path).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
