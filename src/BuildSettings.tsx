import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { BuildPlan, SavedBuild, UiModel } from "./types";
import "./BuildSettings.css";

export function BuildSettings() {
  const [saved, setSaved] = useState<SavedBuild | null>(null);
  const [preview, setPreview] = useState<BuildPlan | null>(null);
  const [url, setUrl] = useState("");
  const [profile, setProfile] = useState("");
  const [act, setAct] = useState<number | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    let disposed = false;
    let stop: UnlistenFn | undefined;
    let eventArrived = false;
    invoke<SavedBuild>("get_build").then(value => {
      if (!disposed) { setSaved(value); setProfile(value.profile); setUrl(value.plan?.source_url ?? ""); }
    }).catch(e => { if (!disposed) setError(String(e)); });
    listen<UiModel>("overlay-model", ({payload}) => {
      eventArrived = true;
      if (!disposed) setAct(payload.build_act ?? null);
    }).then(async unlisten => {
      if (disposed) { unlisten(); return; }
      stop = unlisten;
      const model = await invoke<UiModel>("get_model");
      if (!disposed && !eventArrived) setAct(model.build_act ?? null);
    }).catch(e => { if (!disposed) setError(String(e)); });
    return () => { disposed = true; stop?.(); };
  }, []);

  async function mutate(command: string, args?: Record<string, unknown>) {
    setBusy(true); setError(null);
    try {
      const value = await invoke<SavedBuild>(command, args);
      setSaved(value); setProfile(value.profile);
      if (command === "save_imported_build" || command === "remove_build") setPreview(null);
    } catch (e) { setError(String(e)); } finally { setBusy(false); }
  }
  async function fetchPreview(target: string) {
    setBusy(true); setError(null); setPreview(null);
    try { setPreview(await invoke<BuildPlan>("preview_mobalytics", {url: target})); }
    catch (e) { setError(String(e)); } finally { setBusy(false); }
  }
  async function previewSavedPage() {
    setBusy(true); setError(null); setPreview(null);
    try { setPreview(await invoke<BuildPlan | null>("preview_mobalytics_file", {url})); }
    catch (e) { setError(String(e)); } finally { setBusy(false); }
  }
  const plan = saved?.plan;
  const progress = saved?.profiles[saved.profile];
  const matches = plan?.stages.filter(stage => act !== null && stage.acts.includes(act)) ?? [];
  const stage = progress?.stage_override
    ? plan?.stages.find(stage => stage.id === progress.stage_override)
    : matches.length === 1 ? matches[0] : undefined;

  return <section className="settings-row build-settings" aria-label="Mobalytics build">
    <h2>Build guide</h2>
    <label className="settings-label" htmlFor="build-url">Mobalytics PoE2 build link</label>
    <input id="build-url" className="build-input" type="url" value={url} disabled={busy}
      onChange={e => { setUrl(e.target.value); setPreview(null); }} placeholder="https://mobalytics.gg/poe-2/builds/…" />
    <div className="build-actions">
      <button type="button" className="btn btn-secondary" disabled={busy || !saved || !url.trim()}
        onClick={() => fetchPreview(url)}>Preview import</button>
      <button type="button" className="btn btn-secondary" disabled={busy || !saved || !url.trim()}
        onClick={previewSavedPage}>Preview saved page</button>
      {plan && <button type="button" className="btn btn-secondary" disabled={busy}
        onClick={() => { setUrl(plan.source_url); fetchPreview(plan.source_url); }}>Check for guide update</button>}
    </div>
    <p>If direct import is blocked, open this build in your browser, press Ctrl+S,
      save as Webpage, HTML Only (or Webpage, Complete), then choose Preview saved page.
      Keep the original build link above. PDF and MHTML files are not supported.</p>
    {busy && <p role="status">Working…</p>}
    {error && <p role="alert" className="preview-error">{error}</p>}
    {!saved && !error && <p>Loading saved build…</p>}
    {preview && <section className="build-preview" aria-label="Import preview">
      <strong>{preview.title}</strong><p>By {preview.author} · {preview.stages.length} campaign stages</p>
      <ul>{preview.stages.map(s => <li key={s.id}>{s.title}: {s.items.length} targets{s.acts.length === 0 ? " (manual stage selection)" : ""}</li>)}</ul>
      {preview.skipped_stages.length > 0 && <p>Endgame stages excluded: {preview.skipped_stages.join(", ")}</p>}
      <p>Saving replaces the current build and clears its completion checkmarks for all profiles. Campaign progress is kept.</p>
      <button type="button" className="btn btn-secondary" disabled={busy} onClick={() => mutate("save_imported_build")}>Save imported build</button>
      <button type="button" className="btn btn-secondary" disabled={busy} onClick={() => setPreview(null)}>Cancel preview</button>
    </section>}
    {plan && <>
      <h3>{plan.title}</h3>
      <p className="build-source">By {plan.author} · source updated {plan.source_updated || "unknown"}<br />
        Saved {new Date(plan.imported_at * 1000).toLocaleString()}<br />{plan.source_url}</p>
      <p>Targets are advice. Check them manually after equipping skills, changing gear, or allocating passives.</p>
      <label className="settings-label" htmlFor="build-profile">Character / progress profile</label>
      <input id="build-profile" className="build-input" value={profile} disabled={busy} maxLength={100} onChange={e => setProfile(e.target.value)} />
      <button type="button" className="btn btn-secondary" disabled={busy || !profile.trim()}
        onClick={() => mutate("set_build_profile", {profile, stage: saved?.profiles[profile.trim()]?.stage_override ?? null})}>Use character</button>
      <p>Active profile: {saved?.profile}. Switch this when changing characters.</p>
      <label className="settings-label" htmlFor="build-stage">Build stage</label>
      <select id="build-stage" value={progress?.stage_override ?? ""} disabled={busy}
        onChange={e => mutate("set_build_profile", {profile: saved?.profile, stage: e.target.value || null})}>
        <option value="">Automatic — current campaign area</option>
        {plan.stages.map(s => <option value={s.id} key={s.id}>{s.title}</option>)}
      </select>
      {!stage && <p>Select a stage manually or enter a supported campaign area. Ambiguous stages are not selected automatically.</p>}
      {stage && <div aria-label="Build checklist">
        <h3>{stage.title}</h3>
        {stage.items.map(item => <label className="build-check" key={item.id}>
          <input type="checkbox" disabled={busy} checked={progress?.completed.includes(`${stage.id}/${item.id}`) ?? false}
            onChange={e => mutate("set_build_item", {stage: stage.id, item: item.id, done: e.target.checked})} />
          <span><strong>{item.category}:</strong> {item.text}</span>
        </label>)}
        <p>{stage.passive_nodes.length} passive nodes in this stage's target tree. Named priorities above are targets, not an inferred point-by-point allocation order.</p>
        {stage.notes.length > 0 && <details><summary>Author's stage instructions</summary>
          {stage.notes.map((note, index) => <p className="build-notes" key={index}>{note}</p>)}
        </details>}
      </div>}
      <button type="button" className="btn btn-secondary" disabled={busy} onClick={() => mutate("remove_build")}>Remove saved build</button>
    </>}
  </section>;
}
