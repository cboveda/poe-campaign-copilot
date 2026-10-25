import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { BuildSettings } from "./BuildSettings";
import { SettingsPage } from "./SettingsPage";
import type { AppConfig, PobSummary, UiModel } from "./types";

/// Thin, untested wiring layer (same split as useOverlay/FilmstripBar): all
/// `invoke` calls and settings-window-local state live here, so the
/// presentational `SettingsPage` stays a pure function of props.
///
/// `config` starts `null` and `SettingsPage` isn't mounted until the initial
/// `get_config` resolves — NOT seeded with an empty/default `AppConfig` up
/// front. `SettingsPage` seeds its local `variant`/`pobText` state from
/// `config` exactly once, on mount (see its own doc comment), so mounting it
/// early against a placeholder config would let those fields go stale and
/// silently clobber the real persisted values on Save. Gating the mount on
/// the loaded config sidesteps that resync problem entirely rather than
/// papering over it with an effect that re-seeds local state later.
export function SettingsContainer() {
  const [pendingTownReturn, setPendingTownReturn] = useState(false);
  const [confirmingTownReturn, setConfirmingTownReturn] = useState(false);
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [preview, setPreview] = useState<PobSummary | null>(null);
  const [previewError, setPreviewError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [savedAt, setSavedAt] = useState<number | null>(null);
  // No update checks until the PoE2 distribution channel is configured.

  useEffect(() => {
    let disposed = false;
    invoke<AppConfig>("get_config")
      .then((cfg) => {
        if (!disposed) setConfig(cfg);
      })
      .catch((e) => console.error("get_config failed:", e));
    return () => {
      disposed = true;
    };
  }, []);

  useEffect(() => {
    let disposed = false;
    let eventArrived = false;
    let unlisten: UnlistenFn | undefined;
    listen<UiModel>("overlay-model", ({ payload }) => {
      eventArrived = true;
      if (!disposed) setPendingTownReturn(Boolean(payload.pending_town_return));
    }).then(async (stop) => {
      if (disposed) { stop(); return; }
      unlisten = stop;
      const model = await invoke<UiModel>("get_model");
      if (!disposed && !eventArrived) setPendingTownReturn(Boolean(model.pending_town_return));
    }).catch((error) => { if (!disposed) setPreviewError(String(error)); });
    return () => { disposed = true; unlisten?.(); };
  }, []);

  async function handleConfirmTownReturn() {
    setConfirmingTownReturn(true);
    setPreviewError(null);
    try {
      await invoke("confirm_town_return");
      setPendingTownReturn(false);
    } catch (error) {
      setPreviewError(String(error));
    } finally {
      setConfirmingTownReturn(false);
    }
  }

  // Runs the native file dialog and returns the chosen path. SettingsPage
  // autosaves it (committing the full config), which flows back through
  // handleSave and updates `config` — so this no longer sets config itself.
  async function handlePick(): Promise<string | null> {
    try {
      return (await invoke<string | null>("pick_log_file")) ?? null;
    } catch (e) {
      console.error("pick_log_file failed:", e);
      return null;
    }
  }

  async function handleReset() {
    try {
      await invoke("reset_progress");
    } catch (e) {
      console.error("reset_progress failed:", e);
    }
  }

  async function handleResetTimer() {
    try {
      await invoke("reset_run_timer");
    } catch (e) {
      console.error("reset_run_timer failed:", e);
    }
  }

  async function handleImportPreview(code: string) {
    setPreviewError(null);
    setPreview(null);
    if (code.trim() === "") {
      return;
    }
    try {
      const summary = await invoke<PobSummary>("import_pob", { code });
      setPreview(summary);
    } catch (e) {
      setPreviewError(String(e));
    }
  }

  function handleOpacityPreview(opacity: number) {
    // Fire-and-forget live preview: the backend clamps and re-emits to the
    // overlay window. Not persisted until Save.
    invoke("set_overlay_opacity", { opacity }).catch((e) =>
      console.error("set_overlay_opacity failed:", e),
    );
  }

  async function handleSave(cfg: AppConfig) {
    setSaving(true);
    try {
      await invoke("apply_settings", { cfg });
      setConfig(cfg);
      // A previous failed save (e.g. "log file not found") may have left
      // previewError set; a successful save supersedes it.
      setPreviewError(null);
      setSavedAt(Date.now());
    } catch (e) {
      // SettingsPage's props (fixed by the brief) only have one error slot —
      // previewError — so a failed save (e.g. "log file not found", a bad
      // PoB code re-validated server-side) surfaces there too rather than
      // via a second, unwired prop.
      setPreviewError(String(e));
    } finally {
      setSaving(false);
    }
  }

  if (config === null) {
    // Deliberately not rendering SettingsPage with a placeholder AppConfig
    // here — see the module doc comment above. Inline-styled (rather than
    // pulling in SettingsPage.css) since this placeholder is the one bit of
    // the settings window SettingsPage itself never renders.
    return (
      <div
        style={{
          minHeight: "100vh",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          background: "#16110c",
          color: "#8a8072",
          fontFamily: "-apple-system, 'Segoe UI', system-ui, sans-serif",
          fontSize: 13,
        }}
      >
        Loading&hellip;
      </div>
    );
  }

  return (
    <SettingsPage
      poe2Only
      buildPanel={<BuildSettings />}
      pendingTownReturn={pendingTownReturn}
      confirmingTownReturn={confirmingTownReturn}
      onConfirmTownReturn={handleConfirmTownReturn}
      config={config}
      onPick={handlePick}
      onImportPreview={handleImportPreview}
      preview={preview}
      previewError={previewError}
      onSave={handleSave}
      onReset={handleReset}
      onResetTimer={handleResetTimer}
      saving={saving}
      savedAt={savedAt}
      onOpacityPreview={handleOpacityPreview}
    />
  );
}
