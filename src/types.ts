// Mirrors the Rust UiModel/OverlayModel JSON payloads (snake_case) exactly.
// See src-tauri/src/pipeline.rs (UiModel, UiImage),
// crates/composer/src/lib.rs (OverlayModel, NoteView, ImageView),
// src-tauri/src/config.rs (AppConfig), and src-tauri/src/main.rs
// (PobSummary).

export type NoteCategory = "layout" | "objective" | "danger";

// Outdated notes are dropped by the composer (never shown / no strike-through),
// so a NoteView is always current — no `stale` flag. Images keep their own
// `stale` for dimming (see UiImage).
export interface NoteView {
  text: string;
  category: NoteCategory;
}

export interface UiImage {
  file: string;
  stale: boolean;
  data_url: string;
}

export interface OverlayModel {
  zone_name: string;
  area_id: string;
  act: number;
  off_route_zone: string | null;
  layout_images: { file: string; stale: boolean }[];
  layout_notes: NoteView[];
  steps_in_zone: string[];
  sub_hints: string[];
  primary: string;
  next_zone: string | null;
  pending_count: number;
  town_reminders: string[];
  build_reminders: string[];
  is_town: boolean;
  route_complete: boolean;
  location_status: "on_track" | "catching_up" | "revisiting";
  groups_behind: number;
}

export interface UiModel {
  overlay: OverlayModel;
  images: UiImage[];
  waiting_for_log: boolean;
  pending_town_return?: boolean;
  chapter_label?: string | null;
  navigation?: string[];
  navigation_sources?: string[];
  navigation_caveat?: string | null;
  build_summary: string | null;
  build_guide?: BuildView | null;
  build_act?: number | null;
}

export type RouteVariant = "league-start" | "standard";

// Mirrors src-tauri/src/hotkeys.rs (HotkeyConfig). Values are canonical
// global-shortcut strings like "alt+shift+s" — see src/hotkeys.ts.
export interface HotkeyConfig {
  zoom: string;
  compact: string;
  hide: string;
  setup: string;
  settings: string;
  timer: string;
}

export interface AppConfig {
  client_log_path: string | null;
  variant: RouteVariant;
  pob_code: string | null;
  /** Overlay window opacity, 0.2–1.0 (see src/opacity.ts). */
  overlay_opacity: number;
  hotkeys: HotkeyConfig;
  /** Whether the campaign run timer chip is shown on the overlay. */
  show_run_timer: boolean;
}

export type Reliability = "explicit" | "structured" | "inferred" | "unsupported";

export interface PobSummary {
  class_name: string;
  ascend_name: string | null;
  milestone_count: number;
  reliability: Reliability;
}

export interface BuildItem { id: string; category: string; text: string }
export interface BuildStage {
  id: string;
  title: string;
  acts: number[];
  items: BuildItem[];
  notes: string[];
  passive_nodes: string[];
}
export interface BuildPlan {
  format_version: number;
  source_url: string;
  title: string;
  author: string;
  source_updated: string;
  imported_at: number;
  stages: BuildStage[];
  skipped_stages: string[];
}
export interface SavedBuild {
  plan: BuildPlan | null;
  profile: string;
  profiles: Record<string, {stage_override: string | null; completed: string[]}>;
}
export interface BuildView {
  title: string;
  stage: string;
  pending: string[];
  completed_count: number;
  total_count: number;
}
