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
beforeEach(() => {
  saved = {plan: null, profile: "Current character", profiles: {}};
  invoke.mockReset(); listen.mockReset();
  listen.mockResolvedValue(() => {});
  invoke.mockImplementation(async (command: string, args?: Record<string, unknown>) => {
    switch (command) {
      case "get_build": return structuredClone(saved);
      case "get_model": return {build_act: 1};
      case "preview_mobalytics": return structuredClone(plan);
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
