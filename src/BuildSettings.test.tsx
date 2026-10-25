import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { BuildSettings } from "./BuildSettings";
import type { BuildPlan, SavedBuild } from "./types";

const { invoke, listen } = vi.hoisted(() => ({ invoke: vi.fn(), listen: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen }));
const plan: BuildPlan = {
  format_version: 1, source_url: "https://mobalytics.gg/poe-2/builds/example-starter",
  title: "Example Grenades", author: "Example Author", source_updated: "2026-10-25",
  imported_at: 1792886400, skipped_stages: ["Early Endgame"],
  stages: [{id: "act1", title: "ACT 1", acts: [1], passive_nodes: ["node-1"],
    items: [{id: "gem", category: "Skill", text: "Grenade + Support"}], notes: ["Act 1 instructions <script>literal text</script>"]},
  {id: "act2", title: "ACT 2", acts: [2], passive_nodes: [], items: [], notes: ["Act 2 instructions"]}],
};
let saved: SavedBuild;
let finishResult: BuildPlan | null;
beforeEach(() => {
  saved = {plan: null, profile: "Current character", profiles: {}};
  finishResult = structuredClone(plan);
  invoke.mockReset(); listen.mockReset();
  listen.mockResolvedValue(() => {});
  invoke.mockImplementation(async (command: string, args?: Record<string, unknown>) => {
    switch (command) {
      case "get_build": return structuredClone(saved);
      case "get_model": return {build_act: 1};
      case "preview_mobalytics":
      case "preview_mobalytics_file": return structuredClone(plan);
      case "begin_source_import": return "transfer-1";
      case "append_source_import":
      case "cancel_source_import": return;
      case "finish_source_import": return structuredClone(finishResult);
      case "save_imported_build": saved.plan = structuredClone(plan); return structuredClone(saved);
      case "remove_build": saved.plan = null; return structuredClone(saved);
      case "set_build_profile": {
        const profile = String(args?.profile);
        saved.profile = profile;
        saved.profiles[profile] = {stage_override: args?.stage as string | null, completed: saved.profiles[profile]?.completed ?? []};
        return structuredClone(saved);
      }
      case "set_build_item": {
        saved.profiles[saved.profile] = {stage_override: null, completed: args?.done ? ["act1/gem"] : []};
        return structuredClone(saved);
      }
      default: throw new Error(command);
    }
  });
});
async function imported() {
  render(<BuildSettings />);
  const button = await screen.findByRole("button", {name: "Preview import"});
  await waitFor(() => expect(screen.queryByText("Loading saved build…")).not.toBeInTheDocument());
  fireEvent.change(screen.getByLabelText("Mobalytics PoE2 build link"), {target: {value: plan.source_url}});
  fireEvent.click(button);
  await screen.findByRole("region", {name: "Import preview"});
}
describe("Mobalytics build settings", () => {
  it("previews before saving and renders a stage's linked targets and plain-text notes", async () => {
    await imported();
    expect(screen.getByRole("status")).toHaveTextContent("Preview ready — build has not been saved.");
    await waitFor(() => expect(screen.getByRole("button", {name: "Save imported build"})).toHaveFocus());
    expect(invoke).not.toHaveBeenCalledWith("save_imported_build", undefined);
    expect(screen.getByText(/Endgame stages excluded/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", {name: "Save imported build"}));
    const checklist = await screen.findByLabelText("Build checklist");
    expect(within(checklist).getByText("Grenade + Support")).toBeInTheDocument();
    expect(within(checklist).getByText(/Act 1 instructions <script>/)).toBeInTheDocument();
    expect(within(checklist).queryByText("Act 2 instructions")).not.toBeInTheDocument();
    fireEvent.click(within(checklist).getByRole("checkbox"));
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("set_build_item", {stage: "act1", item: "gem", done: true}));
    await waitFor(() => expect(screen.getByRole("checkbox")).toBeChecked());
  });
  it("previews a saved HTML file after a blocked direct import", async () => {
    render(<BuildSettings />);
    await waitFor(() => expect(screen.queryByText("Loading saved build…")).not.toBeInTheDocument());
    fireEvent.change(screen.getByLabelText("Mobalytics PoE2 build link"), {target: {value: plan.source_url}});
    fireEvent.click(screen.getByRole("button", {name: "Preview saved page"}));
    await screen.findByRole("region", {name: "Import preview"});
    expect(invoke).toHaveBeenCalledWith("preview_mobalytics_file", {url: plan.source_url});
    expect(invoke).not.toHaveBeenCalledWith("save_imported_build", undefined);
    fireEvent.click(screen.getByRole("button", {name: "Save imported build"}));
    await screen.findByLabelText("Build checklist");
  });
  it("leaves the saved build alone when the file picker is cancelled", async () => {
    saved.plan = structuredClone(plan);
    render(<BuildSettings />);
    await screen.findByLabelText("Build checklist");
    invoke.mockImplementationOnce(async () => null);
    fireEvent.click(screen.getByRole("button", {name: "Preview saved page"}));
    await waitFor(() => expect(screen.queryByText("Working…")).not.toBeInTheDocument());
    expect(screen.queryByRole("region", {name: "Import preview"})).not.toBeInTheDocument();
    expect(screen.getByLabelText("Build checklist")).toBeInTheDocument();
  });
  it("previews pasted source without fetching the link or automatically saving", async () => {
    render(<BuildSettings />);
    await waitFor(() => expect(screen.queryByText("Loading saved build…")).not.toBeInTheDocument());
    fireEvent.change(screen.getByLabelText("Mobalytics PoE2 build link"), {target: {value: plan.source_url}});
    fireEvent.click(screen.getByText("Paste page source instead (no download)"));
    const html = '<html><script>window.__PRELOADED_STATE__={}</script></html>';
    fireEvent.change(screen.getByLabelText("Page source HTML"), {target: {value: html}});
    fireEvent.click(screen.getByRole("button", {name: "Preview pasted source"}));
    await screen.findByRole("region", {name: "Import preview"});
    expect(invoke).toHaveBeenCalledWith("begin_source_import", {url: plan.source_url});
    expect(invoke).toHaveBeenCalledWith("append_source_import", {token: "transfer-1", chunk: html});
    expect(invoke).toHaveBeenCalledWith("finish_source_import", {token: "transfer-1"});
    expect(invoke.mock.calls.some(call => call[0] === "preview_mobalytics")).toBe(false);
    expect(invoke.mock.calls.some(call => call[0] === "save_imported_build")).toBe(false);
    expect(screen.getByLabelText("Page source HTML")).toHaveValue("");
  });
  it("reports an empty preview response instead of silently hiding the save action", async () => {
    render(<BuildSettings />);
    await waitFor(() => expect(screen.queryByText("Loading saved build…")).not.toBeInTheDocument());
    fireEvent.change(screen.getByLabelText("Mobalytics PoE2 build link"), {target: {value: plan.source_url}});
    fireEvent.click(screen.getByText("Paste page source instead (no download)"));
    fireEvent.change(screen.getByLabelText("Page source HTML"), {target: {value: "example source"}});
    finishResult = null;
    fireEvent.click(screen.getByRole("button", {name: "Preview pasted source"}));
    expect(await screen.findByRole("alert")).toHaveTextContent("No usable build preview was returned");
    expect(screen.getByLabelText("Page source HTML")).toHaveValue("example source");
    expect(screen.queryByRole("button", {name: "Save imported build"})).not.toBeInTheDocument();
  });
  it("keeps the saved build when a refresh fails", async () => {
    saved.plan = structuredClone(plan);
    render(<BuildSettings />);
    const refresh = await screen.findByRole("button", {name: "Check for guide update"});
    invoke.mockImplementationOnce(async () => {throw new Error("HTTP 403");});
    fireEvent.click(refresh);
    expect(await screen.findByRole("alert")).toHaveTextContent("HTTP 403");
    expect(screen.getByLabelText("Build checklist")).toBeInTheDocument();
    expect(invoke).not.toHaveBeenCalledWith("save_imported_build", undefined);
  });
  it("supports manual stages, separate profiles and removing the saved build", async () => {
    saved.plan = structuredClone(plan);
    render(<BuildSettings />);
    await screen.findByLabelText("Build checklist");
    fireEvent.change(screen.getByLabelText("Build stage"), {target: {value: "act2"}});
    await screen.findByText("Act 2 instructions");
    expect(screen.queryByRole("checkbox")).not.toBeInTheDocument();
    fireEvent.change(screen.getByLabelText("Character / progress profile"), {target: {value: "Second character"}});
    fireEvent.click(screen.getByRole("button", {name: "Use character"}));
    await screen.findByText(/Active profile: Second character/);
    fireEvent.click(screen.getByRole("button", {name: "Remove saved build"}));
    await waitFor(() => expect(screen.queryByLabelText("Build checklist")).not.toBeInTheDocument());
  });
});
