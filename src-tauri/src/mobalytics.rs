//! Reads public provider data without evaluating JavaScript or fetching linked assets.
use crate::build_plan::{BuildItem, BuildPlan, BuildStage};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_HTML: usize = 8 * 1024 * 1024;

pub fn canonical_url(input: &str) -> Result<String, String> {
    let url =
        reqwest::Url::parse(input.trim()).map_err(|_| "Enter a Mobalytics PoE2 build URL.")?;
    if url.scheme() != "https"
        || !matches!(url.host_str(), Some("mobalytics.gg" | "www.mobalytics.gg"))
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
    {
        return Err("Only HTTPS Mobalytics build links are supported.".into());
    }
    let slug = url
        .path()
        .strip_prefix("/poe-2/builds/")
        .unwrap_or("")
        .trim_end_matches('/');
    if slug.is_empty()
        || slug.len() > 200
        || !slug
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
    {
        return Err("Use a link to one PoE2 build, not a profile or planner page.".into());
    }
    Ok(format!("https://mobalytics.gg/poe-2/builds/{slug}"))
}

pub async fn fetch(input: &str) -> Result<BuildPlan, String> {
    let url = canonical_url(input)?;
    // reqwest's no-provider feature requires explicit initialization. A prior
    // successful install is fine; install_default never replaces a provider.
    let _ = rustls::crypto::ring::default_provider().install_default();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent(concat!("PoE2-Campaign-Copilot/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| format!("Could not initialize HTTPS: {e}"))?;
    let mut response = client
        .get(&url)
        .header(reqwest::header::ACCEPT, "text/html,application/xhtml+xml")
        .header(reqwest::header::ACCEPT_LANGUAGE, "en-US,en;q=0.9")
        .send()
        .await
        .map_err(|e| format!("Could not fetch Mobalytics: {e}"))?;
    if response.status().as_u16() == 403 {
        return Err("Mobalytics denied the direct request (HTTP 403). Open the build in your browser, save it as an HTML page, then choose Preview saved page. Your saved build was not changed.".into());
    }
    if !response.status().is_success() {
        return Err(format!(
            "Mobalytics returned HTTP {}. Your saved build was not changed.",
            response.status().as_u16()
        ));
    }
    if response
        .content_length()
        .is_some_and(|n| n > MAX_HTML as u64)
    {
        return Err("Build page is too large.".into());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| format!("Could not read build page: {e}"))?
    {
        if bytes.len().saturating_add(chunk.len()) > MAX_HTML {
            return Err("Build page is too large.".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    let html = std::str::from_utf8(&bytes).map_err(|_| "Build page is not UTF-8.")?;
    parse_page(html, &url)
}

/// Read only the file explicitly chosen in the native dialog; no linked assets.
pub fn parse_saved_page(path: &std::path::Path, url: &str) -> Result<BuildPlan, String> {
    use std::io::Read;
    canonical_url(url)?;
    let file = std::fs::File::open(path).map_err(|e| format!("Could not open saved page: {e}"))?;
    let metadata = file.metadata().map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.len() > MAX_HTML as u64 {
        return Err("Choose an HTML file smaller than 8 MiB.".into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_HTML as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > MAX_HTML {
        return Err("Saved page is too large.".into());
    }
    let html = std::str::from_utf8(&bytes)
        .map_err(|_| "Save the page as UTF-8 HTML, not PDF or an archive.")?;
    parse_page(html.trim_start_matches('\u{feff}'), url)
}

fn text(value: &Value) -> String {
    fn visit(v: &Value, out: &mut String) {
        if let Some(s) = v.get("text").and_then(Value::as_str) {
            out.push_str(s);
        } else if v.get("type").and_then(Value::as_str) == Some("static-data-widget") {
            if let Some(label) = v.get("label").and_then(Value::as_str) {
                out.push_str(label);
            }
        } else if let Some(children) = v.get("children").and_then(Value::as_array) {
            for child in children {
                visit(child, out);
            }
            if matches!(
                v.get("type").and_then(Value::as_str),
                Some("paragraph" | "listitem" | "heading")
            ) {
                out.push('\n');
            }
        } else if let Some(root) = v.get("root") {
            visit(root, out);
        } else if let Some(value) = v.get("value") {
            visit(value, out);
        }
    }
    let mut out = String::new();
    visit(value, &mut out);
    out.lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
        .chars()
        .take(20_000)
        .collect()
}
fn array<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}
fn string(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .chars()
        .take(500)
        .collect()
}
fn walk<'a>(v: &'a Value, visit: &mut impl FnMut(&'a Value)) {
    visit(v);
    match v {
        Value::Object(map) => {
            for child in map.values() {
                walk(child, visit);
            }
        }
        Value::Array(list) => {
            for child in list {
                walk(child, visit);
            }
        }
        _ => {}
    }
}
fn campaign_acts(title: &str) -> Vec<u8> {
    match title.trim().to_ascii_lowercase().replace(' ', "").as_str() {
        "act1" => vec![1],
        "act2" => vec![2],
        "act3" => vec![3],
        "act4" => vec![4],
        "act3/4" | "acts3/4" => vec![3, 4],
        "interlude" | "interludes" => vec![5],
        _ => vec![],
    }
}

pub fn parse_page(html: &str, url: &str) -> Result<BuildPlan, String> {
    if html.len() > MAX_HTML {
        return Err("Build page is too large.".into());
    }
    let url = canonical_url(url)?;
    // Parse only the JSON following the known assignment inside a script.
    // The stream parser stops at JSON's end; trailing JavaScript is never run.
    let mut state = None;
    for script in html.split("<script").skip(1) {
        let Some((_, body)) = script.split_once('>') else {
            continue;
        };
        let body = body.split("</script>").next().unwrap_or("").trim();
        if let Some(data) = body.strip_prefix("window.__PRELOADED_STATE__=") {
            state = serde_json::Deserializer::from_str(data)
                .into_iter::<Value>()
                .next()
                .transpose()
                .map_err(|_| "Mobalytics returned invalid build data.")?;
            break;
        }
    }
    let state = state
        .ok_or("This page has no supported build data. Mobalytics may have changed its format.")?;
    let slug = url.rsplit('/').next().unwrap_or("");
    let mut documents = Vec::new();
    walk(&state, &mut |v| {
        if v.get("type").and_then(Value::as_str) == Some("builds")
            && (v.pointer("/featured/slug").and_then(Value::as_str) == Some(slug)
                || v.get("slugifiedName").and_then(Value::as_str) == Some(slug))
            && v.pointer("/data/buildVariants/values").is_some()
        {
            documents.push(v);
        }
    });
    if documents.len() != 1 {
        return Err("Could not identify this build unambiguously in the page data.".into());
    }
    parse_document(documents[0], url)
}

fn parse_document(doc: &Value, url: String) -> Result<BuildPlan, String> {
    let content = array(doc, "content");
    let mut labels = BTreeMap::new();
    walk(doc, &mut |v| {
        if v.get("type").and_then(Value::as_str) == Some("static-data-widget") {
            let id = string(v, "id");
            let label = string(v, "label");
            if !id.is_empty() && !label.is_empty() {
                labels.insert(id, label);
            }
        }
    });
    let variants = content
        .iter()
        .find_map(|v| {
            v.pointer("/data/childrenVariants")
                .and_then(Value::as_array)
        })
        .ok_or("Build has no supported variant descriptions.")?;
    if variants.len() > 32 {
        return Err("Build has too many variants.".into());
    }
    let values = doc
        .pointer("/data/buildVariants/values")
        .and_then(Value::as_array)
        .ok_or("Build has no structured variants.")?;
    let mut stages = Vec::new();
    let mut skipped = Vec::new();
    let mut ids = BTreeSet::new();
    for variant in variants {
        let title = string(variant, "title");
        let id = string(variant, "id");
        if id.is_empty() || !ids.insert(id.clone()) {
            return Err("Build contains missing or duplicate stage IDs.".into());
        }
        if title
            .to_ascii_lowercase()
            .replace(' ', "")
            .contains("endgame")
        {
            skipped.push(title);
            continue;
        }
        let candidates: Vec<_> = values.iter().filter(|v| string(v, "id") == id).collect();
        if candidates.len() != 1 {
            return Err(format!(
                "Missing or ambiguous structured data for stage {title}."
            ));
        }
        let data = candidates[0];
        let mut items = Vec::new();
        for (index, skill) in data
            .pointer("/skillGems/gems")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .enumerate()
        {
            let name = skill
                .pointer("/activeSkill/name")
                .and_then(Value::as_str)
                .unwrap_or("");
            if name.is_empty() {
                continue;
            }
            let supports: Vec<_> = array(skill, "subSkills")
                .iter()
                .filter_map(|s| {
                    let slug = s.get("gemSlug")?.as_str()?;
                    Some(
                        labels
                            .get(slug)
                            .cloned()
                            .unwrap_or_else(|| format!("{slug} (unresolved support ID)")),
                    )
                })
                .collect();
            items.push(BuildItem {
                id: format!("skill-{index}"),
                category: "Skill".into(),
                text: if supports.is_empty() {
                    name.into()
                } else {
                    format!("{name} + {}", supports.join(" + "))
                },
            });
        }
        if let Some(equipment) = data.get("equipment").and_then(Value::as_object) {
            for (slot, value) in equipment {
                if let Some(item) = value.get("commonItem") {
                    let name = string(item, "name");
                    if name.is_empty() {
                        continue;
                    }
                    let mods: Vec<_> = array(item, "explicitDescriptions")
                        .iter()
                        .map(|v| string(v, "description"))
                        .filter(|v| !v.is_empty())
                        .collect();
                    items.push(BuildItem {
                        id: format!("gear-{slot}"),
                        category: "Gear target".into(),
                        text: format!(
                            "{slot}: {name}{}",
                            if mods.is_empty() {
                                String::new()
                            } else {
                                format!(" — {}", mods.join("; "))
                            }
                        ),
                    });
                }
            }
        }
        let tree = data
            .pointer("/passiveTree/mainTree")
            .unwrap_or(&Value::Null);
        for (index, priority) in array(tree, "priorityList").iter().enumerate() {
            let name = string(priority, "name");
            if !name.is_empty() {
                items.push(BuildItem {
                    id: format!("passive-{index}"),
                    category: "Passive target".into(),
                    text: name,
                });
            }
        }
        let mut notes = Vec::new();
        let overview = text(&variant["description"]);
        if !overview.is_empty() {
            notes.push(overview);
        }
        // Join each stage to its own widgets, never descriptions from another stage.
        let child_ids: BTreeSet<_> = array(variant, "childrenIds")
            .iter()
            .filter_map(Value::as_str)
            .collect();
        for widget in content
            .iter()
            .filter(|w| child_ids.contains(string(w, "id").as_str()))
        {
            for field in [
                "descriptionPoeEquipment",
                "descriptionPoeSkillGems",
                "descriptionPoe2PassiveTree",
            ] {
                let note = text(&widget["data"][field]);
                if !note.is_empty() {
                    notes.push(note);
                }
            }
        }
        if items.is_empty() && notes.is_empty() {
            return Err(format!("Stage {title} contains no usable guidance."));
        }
        if items.len() > 256 || notes.len() > 20 {
            return Err("Build stage is too large.".into());
        }
        stages.push(BuildStage {
            id,
            acts: campaign_acts(&title),
            title,
            items,
            notes,
            passive_nodes: array(tree, "selectedSlugs")
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .take(500)
                .collect(),
        });
    }
    if stages.is_empty() {
        return Err(
            "This build has no campaign stages; endgame guidance is outside this release.".into(),
        );
    }
    Ok(BuildPlan {
        format_version: 1,
        source_url: url,
        title: string(&doc["data"], "name"),
        author: string(&doc["author"], "name"),
        source_updated: string(doc, "updatedAt"),
        imported_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        stages,
        skipped_stages: skipped,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    const URL: &str = "https://mobalytics.gg/poe-2/builds/example-starter";
    const PAGE: &str = include_str!("../fixtures/mobalytics-build.html");
    #[test]
    fn imports_matching_stages_links_supports_and_keeps_notes_scoped() {
        let plan = parse_page(PAGE, URL).unwrap();
        assert_eq!(plan.stages.len(), 4);
        assert_eq!(plan.stages[2].acts, [3, 4]);
        assert_eq!(plan.stages[3].acts, [5]);
        assert_eq!(plan.skipped_stages, ["Early Endgame"]);
        assert!(
            plan.stages[0]
                .items
                .iter()
                .any(|i| i.text == "Example Grenade + Example Support")
        );
        assert!(
            plan.stages[0]
                .items
                .iter()
                .any(|i| i.text.contains("movement speed"))
        );
        assert!(
            plan.stages[0]
                .items
                .iter()
                .any(|i| i.text == "Example Notable")
        );
        assert_eq!(plan.stages[0].passive_nodes, ["node-1"]);
        assert!(
            plan.stages[0]
                .notes
                .iter()
                .any(|n| n.contains("Only stage 0 instructions"))
        );
        assert!(plan.stages[0].notes.iter().all(|n| !n.contains("stage 1")));
        assert!(!plan.title.is_empty());
    }
    #[test]
    fn url_validation_blocks_other_hosts_schemes_paths_and_credentials() {
        for url in [
            "http://mobalytics.gg/poe-2/builds/a",
            "https://mobalytics.gg.evil.test/poe-2/builds/a",
            "https://user@mobalytics.gg/poe-2/builds/a",
            "https://localhost/poe-2/builds/a",
            "file:///tmp/a",
            "https://mobalytics.gg/poe-2/builds/a/admin",
            "https://mobalytics.gg:8443/poe-2/builds/a",
            "https://mobalytics.gg/poe-2/builds/a%2fb",
        ] {
            assert!(canonical_url(url).is_err(), "{url}");
        }
        assert_eq!(
            canonical_url(
                "https://www.mobalytics.gg/poe-2/builds/example-starter/?tracking=yes#act2"
            )
            .unwrap(),
            URL
        );
    }
    #[test]
    fn missing_schema_wrong_document_and_oversized_inputs_fail_cleanly() {
        assert!(parse_page("<html>Challenge page</html>", URL).is_err());
        assert!(parse_page(PAGE, "https://mobalytics.gg/poe-2/builds/different").is_err());
        assert!(parse_page(&"x".repeat(MAX_HTML + 1), URL).is_err());
        let invalid = PAGE.replace("window.__PRELOADED_STATE__=", "window.SOMETHING_ELSE=");
        assert!(parse_page(&invalid, URL).is_err());
    }
    #[test]
    fn saved_html_is_bounded_validated_and_can_include_a_utf8_bom() {
        let dir = std::env::temp_dir().join(format!(
            "poe-saved-html-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("build.html");
        std::fs::write(&path, format!("\u{feff}{PAGE}")).unwrap();
        assert_eq!(parse_saved_page(&path, URL).unwrap().stages.len(), 4);
        assert!(parse_saved_page(&path, "https://mobalytics.gg/poe-2/builds/wrong-build").is_err());
        assert!(parse_saved_page(&dir, URL).is_err());
        std::fs::write(&path, b"not a saved build").unwrap();
        assert!(parse_saved_page(&path, URL).is_err());
        std::fs::write(&path, vec![0xff, 0xfe]).unwrap();
        assert!(parse_saved_page(&path, URL).is_err());
        std::fs::write(&path, vec![b'x'; MAX_HTML + 1]).unwrap();
        assert!(parse_saved_page(&path, URL).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    #[ignore = "optional local capture supplied through POE_MOBALYTICS_HTML; no network"]
    fn captured_real_build_smoke() {
        let path = std::env::var("POE_MOBALYTICS_HTML").unwrap();
        let plan = parse_saved_page(
            std::path::Path::new(&path),
            "https://mobalytics.gg/poe-2/builds/deadrabbit-grenades-gemling-league-starter",
        )
        .unwrap();
        assert_eq!(plan.stages.len(), 4);
        assert_eq!(plan.skipped_stages.len(), 3);
        assert!(
            plan.stages[0]
                .items
                .iter()
                .any(|i| i.text.contains("Explosive Grenade") && i.text.contains("Multishot I"))
        );
        assert!(
            plan.stages[0]
                .notes
                .iter()
                .any(|n| n.contains("Skill Gem Order"))
        );
        assert!(
            plan.stages
                .iter()
                .all(|s| !s.items.is_empty() && !s.notes.is_empty())
        );
        assert!(serde_json::to_vec(&plan).unwrap().len() < 1024 * 1024);
    }
}

#[cfg(test)]
#[test]
#[ignore = "optional live HTTPS smoke test; not part of offline test suite"]
fn live_build_fetch_smoke() {
    let plan = tauri::async_runtime::block_on(fetch(
        "https://mobalytics.gg/poe-2/builds/deadrabbit-grenades-gemling-league-starter",
    ))
    .unwrap();
    assert_eq!(plan.stages.len(), 4);
    assert_eq!(plan.author, "DEADRABB1T");
}
