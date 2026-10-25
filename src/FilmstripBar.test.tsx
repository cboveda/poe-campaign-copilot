import { render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { FilmstripBar, type FilmstripBarProps } from "./FilmstripBar";
import { IDLE_RUN_TIMER } from "./runTimer";
import type { UiModel } from "./types";

vi.mock("@tauri-apps/api/core", () => ({invoke:vi.fn().mockResolvedValue(undefined)}));

const PIXEL =
  "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNsb2j4DwAFKAJ003oL8QAAAABJRU5ErkJggg==";

function model(overrides: Partial<UiModel["overlay"]> = {}, extra: Partial<UiModel> = {}): UiModel {
  return {
    overlay: {
      zone_name: "The Coast",
      area_id: "1_1_2",
      act: 1,
      off_route_zone: null,
      layout_images: [{ file: "a.png", stale: false }],
      layout_notes: [{ text: "Follow the right wall.", category: "layout" }],
      steps_in_zone: ["Get waypoint", "➞ The Mud Flats"],
      sub_hints: ["Go ↗"],
      primary: "Get waypoint",
      next_zone: "The Mud Flats",
      pending_count: 2,
      town_reminders: [],
      build_reminders: [],
      is_town: false,
      route_complete: false,
      location_status: "on_track",
      groups_behind: 0,
      ...overrides,
    },
    images: [{ file: "a.png", stale: false, data_url: PIXEL }],
    waiting_for_log: false,
    build_summary: null,
    ...extra,
  };
}

// Shared render helper: defaults zoom/setupMode/compact to false so each
// test only needs to specify the prop(s) it cares about.
function renderBar(overrides: Partial<FilmstripBarProps> & { model: UiModel }) {
  return render(
    <FilmstripBar
      zoom={false}
      setupMode={false}
      compact={false}
      runTimer={IDLE_RUN_TIMER}
      showRunTimer={true}
      nowMs={1_000_000}
      {...overrides}
    />,
  );
}

describe("FilmstripBar", () => {
  it.each([false, true])("shows the pending confirmation directly on the overlay (compact=%s)", (compact) => {
    renderBar({model:model({}, {pending_town_return:true}), compact, onConfirmTownReturn:async () => {}});
    expect(screen.getByRole("button", {name:/continue town step/i})).toBeInTheDocument();
  });
  it("only offers overlay confirmation for a pending town return", () => {
    renderBar({model:model(), onConfirmTownReturn:async () => {}});
    expect(screen.queryByRole("button", {name:/continue town step/i})).not.toBeInTheDocument();
  });
  it("renders zone, act, primary, next and pending badge", () => {
    renderBar({ model: model() });
    expect(screen.getByText("The Coast")).toBeInTheDocument();
    expect(screen.getByText(/act 1/i)).toBeInTheDocument();
    expect(screen.getByText("Get waypoint")).toBeInTheDocument();
    expect(screen.getByText(/^next$/i)).toBeInTheDocument();
    expect(screen.getByText("The Mud Flats")).toBeInTheDocument();
    expect(screen.getByText(/2 pending/i)).toBeInTheDocument();
    expect(screen.getByRole("img")).toHaveAttribute("src", PIXEL);
  });

  it("shows waiting state before any log data", () => {
    renderBar({ model: model({}, { waiting_for_log: true }) });
    expect(screen.getByText(/waiting for client\.txt/i)).toBeInTheDocument();
    expect(screen.queryByText("The Coast")).not.toBeInTheDocument();
  });

  it("shows off-route banner and town reminders in town", () => {
    renderBar({
      model: model({
        off_route_zone: "Lioneye's Watch",
        is_town: true,
        town_reminders: ["Claim quest reward: Quicksilver Flask"],
      }),
    });
    expect(screen.getByText(/off route/i)).toBeInTheDocument();
    expect(screen.getByText(/quicksilver flask/i)).toBeInTheDocument();
  });

  it("shows build reminders with the build class", () => {
    renderBar({
      model: model({
        is_town: true,
        build_reminders: ["Gem available: Frostblink"],
      }),
    });
    const item = screen.getByText(/gem available: frostblink/i);
    expect(item).toHaveClass("build");
  });

  it("marks stale images with the outdated badge", () => {
    // Notes are never stale (outdated notes are dropped upstream); only
    // images can be stale, shown dimmed with an "outdated" badge.
    const m = model();
    m.images = [{ file: "a.png", stale: true, data_url: PIXEL }];
    renderBar({ model: m });
    expect(screen.getByText(/outdated/i)).toBeInTheDocument();
    expect(screen.getByRole("img")).toHaveClass("stale");
  });

  it("renders notes as colour-coded category cards", () => {
    const { container } = renderBar({
      model: model({
        layout_notes: [
          { text: "Cap fire resistance.", category: "danger" },
          { text: "Follow the wall.", category: "layout" },
        ],
      }),
    });
    // The route step is shown as the gold "objective" card...
    expect(screen.getByText("Get waypoint").closest(".note-card")).toHaveClass("objective");
    // ...and each layout note gets its category class.
    expect(screen.getByText("Cap fire resistance.").closest(".note-card")).toHaveClass("danger");
    expect(screen.getByText("Follow the wall.").closest(".note-card")).toHaveClass("layout");
    // The next zone gets its own bar.
    expect(container.querySelector(".next-bar .nb-dest")?.textContent).toBe("The Mud Flats");
  });

  it("renders campaign complete state", () => {
    renderBar({
      model: model({ route_complete: true, zone_name: "Campaign complete" }),
    });
    expect(screen.getByText(/campaign complete/i)).toBeInTheDocument();
  });

  it("applies zoom and setup-mode classes", () => {
    const { container } = renderBar({
      model: model(),
      zoom: true,
      setupMode: true,
    });
    expect(container.firstChild).toHaveClass("zoom");
    expect(screen.getByText(/drag to move/i)).toBeInTheDocument();
  });

  it("renders the build summary when present, with the build-summary class", () => {
    renderBar({
      model: model({}, { build_summary: "Ranger (Deadeye) — 12 milestones" }),
    });
    const summary = screen.getByText("Ranger (Deadeye) — 12 milestones");
    expect(summary).toHaveClass("build-summary");
  });

  it("omits the build summary line when there is no build", () => {
    const { container } = renderBar({ model: model() });
    expect(container.querySelector(".build-summary")).not.toBeInTheDocument();
  });

  // Tauri v2 only starts a window drag when the mousedown TARGET element
  // itself carries data-tauri-drag-region, so a bare attribute on the root
  // does not make clicks on child elements draggable. Setup mode instead
  // renders a dedicated, full-window .drag-layer element that carries the
  // attribute, so a click anywhere in the bar lands on it and drags.
  it("renders the drag layer in the normal-playing state only in setup mode", () => {
    const { container, rerender } = renderBar({
      model: model(),
      setupMode: true,
    });
    const dragLayer = container.querySelector(".drag-layer");
    expect(dragLayer).toBeInTheDocument();
    expect(dragLayer).toHaveAttribute("data-tauri-drag-region");
    // No other element carries the drag attribute — only the dedicated layer.
    expect(container.querySelector("[data-tauri-drag-region]")).toBe(dragLayer);

    rerender(
      <FilmstripBar
        model={model()}
        zoom={false}
        setupMode={false}
        compact={false}
        runTimer={IDLE_RUN_TIMER}
        showRunTimer={true}
        nowMs={1_000_000}
      />,
    );
    expect(container.querySelector(".drag-layer")).not.toBeInTheDocument();
    expect(container.querySelector("[data-tauri-drag-region]")).not.toBeInTheDocument();
  });

  it("renders the drag layer in the waiting state only in setup mode, alongside the setup hint", () => {
    const { container, rerender } = renderBar({
      model: model({}, { waiting_for_log: true }),
      setupMode: true,
    });
    const dragLayer = container.querySelector(".drag-layer");
    expect(dragLayer).toBeInTheDocument();
    expect(dragLayer).toHaveAttribute("data-tauri-drag-region");
    expect(screen.getByText(/drag to move/i)).toBeInTheDocument();

    rerender(
      <FilmstripBar
        model={model({}, { waiting_for_log: true })}
        zoom={false}
        setupMode={false}
        compact={false}
        runTimer={IDLE_RUN_TIMER}
        showRunTimer={true}
        nowMs={1_000_000}
      />,
    );
    expect(container.querySelector(".drag-layer")).not.toBeInTheDocument();
    expect(container.querySelector("[data-tauri-drag-region]")).not.toBeInTheDocument();
    expect(screen.queryByText(/drag to move/i)).not.toBeInTheDocument();
  });

  it("renders the drag layer in the campaign-complete state only in setup mode", () => {
    const { container, rerender } = renderBar({
      model: model({ route_complete: true }),
      setupMode: true,
    });
    const dragLayer = container.querySelector(".drag-layer");
    expect(dragLayer).toBeInTheDocument();
    expect(dragLayer).toHaveAttribute("data-tauri-drag-region");

    rerender(
      <FilmstripBar
        model={model({ route_complete: true })}
        zoom={false}
        setupMode={false}
        compact={false}
        runTimer={IDLE_RUN_TIMER}
        showRunTimer={true}
        nowMs={1_000_000}
      />,
    );
    expect(container.querySelector(".drag-layer")).not.toBeInTheDocument();
    expect(container.querySelector("[data-tauri-drag-region]")).not.toBeInTheDocument();
  });

  describe("location status chip", () => {
    it("shows a Catching up chip when catching_up", () => {
      renderBar({ model: model({ location_status: "catching_up" }) });
      const chip = screen.getByText("Catching up");
      expect(chip).toHaveClass("location-chip");
      expect(chip).toHaveClass("catching-up");
    });

    it("shows a Revisiting chip when revisiting", () => {
      renderBar({ model: model({ location_status: "revisiting" }) });
      const chip = screen.getByText("Revisiting");
      expect(chip).toHaveClass("location-chip");
      expect(chip).toHaveClass("revisiting");
    });

    it("omits the chip when on_track", () => {
      renderBar({ model: model({ location_status: "on_track" }) });
      expect(screen.queryByText("Catching up")).not.toBeInTheDocument();
      expect(screen.queryByText("Revisiting")).not.toBeInTheDocument();
    });

    it("shows the chip in compact mode too", () => {
      renderBar({
        model: model({ location_status: "catching_up" }),
        compact: true,
      });
      const chip = screen.getByText("Catching up");
      expect(chip).toHaveClass("location-chip");
    });
  });

  describe("backtrack breadcrumb", () => {
    it("shows a singular breadcrumb at groups_behind 1", () => {
      renderBar({ model: model({ groups_behind: 1 }) });
      expect(screen.getByText(/1 zone behind your furthest point/i)).toBeInTheDocument();
    });

    it("shows a plural breadcrumb at groups_behind 3", () => {
      renderBar({ model: model({ groups_behind: 3 }) });
      expect(screen.getByText(/3 zones behind your furthest point/i)).toBeInTheDocument();
    });

    it("omits the breadcrumb at groups_behind 0", () => {
      const { container } = renderBar({ model: model({ groups_behind: 0 }) });
      expect(container.querySelector(".breadcrumb-line")).not.toBeInTheDocument();
    });

    it("omits the breadcrumb in compact mode even when groups_behind is set", () => {
      const { container } = renderBar({
        model: model({ groups_behind: 3 }),
        compact: true,
      });
      expect(container.querySelector(".breadcrumb-line")).not.toBeInTheDocument();
    });
  });

  describe("compact mode", () => {
    it("renders zone name, primary action, and next-zone arrow only", () => {
      renderBar({ model: model(), compact: true });
      expect(screen.getByText("The Coast")).toBeInTheDocument();
      expect(screen.getByText("Get waypoint")).toBeInTheDocument();
      expect(screen.getByText(/the mud flats/i)).toBeInTheDocument();
    });

    it("omits images, notes, header badges, off-route banner, and build summary", () => {
      const { container } = renderBar({
        model: model(
          {
            off_route_zone: "Lioneye's Watch",
            is_town: true,
            town_reminders: ["Claim quest reward: Quicksilver Flask"],
            build_reminders: ["Gem available: Frostblink"],
          },
          { build_summary: "Ranger (Deadeye) — 12 milestones" },
        ),
        compact: true,
      });
      expect(container.querySelector(".image-row")).not.toBeInTheDocument();
      expect(container.querySelector(".note-card")).not.toBeInTheDocument();
      expect(container.querySelector(".header-row")).not.toBeInTheDocument();
      expect(container.querySelector(".off-route-banner")).not.toBeInTheDocument();
      expect(container.querySelector(".pending-badge")).not.toBeInTheDocument();
      expect(container.querySelector(".sub-hints")).not.toBeInTheDocument();
      expect(container.querySelector(".town-reminders")).not.toBeInTheDocument();
      expect(container.querySelector(".build-reminders")).not.toBeInTheDocument();
      expect(container.querySelector(".build-summary")).not.toBeInTheDocument();
      expect(screen.queryByRole("img")).not.toBeInTheDocument();
    });

    it("omits the next-zone arrow when there is no next zone", () => {
      const { container } = renderBar({
        model: model({ next_zone: null }),
        compact: true,
      });
      expect(container.querySelector(".compact-next")).not.toBeInTheDocument();
    });

    it("applies the compact root class", () => {
      const { container } = renderBar({ model: model(), compact: true });
      expect(container.firstChild).toHaveClass("compact");
    });

    it("renders the drag layer when compact and setup mode are both on", () => {
      const { container } = renderBar({
        model: model(),
        compact: true,
        setupMode: true,
      });
      const dragLayer = container.querySelector(".drag-layer");
      expect(dragLayer).toBeInTheDocument();
      expect(dragLayer).toHaveAttribute("data-tauri-drag-region");
      expect(container.firstChild).toHaveClass("compact");
    });

    it("does not affect the waiting state", () => {
      renderBar({ model: model({}, { waiting_for_log: true }), compact: true });
      expect(screen.getByText(/waiting for client\.txt/i)).toBeInTheDocument();
    });

    it("does not affect the campaign-complete state", () => {
      renderBar({ model: model({ route_complete: true }), compact: true });
      expect(screen.getByText(/campaign complete/i)).toBeInTheDocument();
    });
  });

  describe("full (non-compact) mode", () => {
    it("still renders the full layout including the image row", () => {
      const { container } = renderBar({ model: model(), compact: false });
      expect(container.querySelector(".image-row")).toBeInTheDocument();
      expect(container.querySelector(".header-row")).toBeInTheDocument();
      expect(container.querySelector(".note-card")).toBeInTheDocument();
    });
  });

  describe("run timer chip", () => {
    it("shows elapsed time while running", () => {
      renderBar({
        model: model(),
        runTimer: { accumulated_ms: 60_000, running_since_ms: 990_000 },
        nowMs: 1_000_000,
      });
      // 60s accumulated + 10s live stretch
      expect(screen.getByText("0:01:10")).toBeInTheDocument();
    });

    it("shows 0:00:00 when never started", () => {
      renderBar({ model: model() });
      expect(screen.getByText("0:00:00")).toBeInTheDocument();
    });

    it("is hidden when disabled in settings", () => {
      renderBar({ model: model(), showRunTimer: false });
      expect(screen.queryByText("0:00:00")).not.toBeInTheDocument();
    });

    it("is hidden on the waiting screen", () => {
      renderBar({ model: model({}, { waiting_for_log: true }) });
      expect(screen.queryByText("0:00:00")).not.toBeInTheDocument();
    });

    it("marks a paused timer", () => {
      renderBar({
        model: model(),
        runTimer: { accumulated_ms: 90_000, running_since_ms: null },
      });
      const chip = screen.getByText("0:01:30");
      expect(chip.closest(".run-timer")).toHaveClass("paused");
    });

    it("appears in compact mode too", () => {
      renderBar({
        model: model(),
        compact: true,
        runTimer: { accumulated_ms: 5_000, running_since_ms: null },
      });
      expect(screen.getByText("0:00:05")).toBeInTheDocument();
    });
  });
});


describe("campaign and optional objective groups", () => {
  it("separates optional loot from campaign/permanent-bonus steps, even when the source starts optional", () => {
    renderBar({model:model({primary:"(Opt) Talk to Delwyn",steps_in_zone:[
      "(Opt) Talk to Delwyn", "Kill The Crowbell +2 Skill Points", "(Opt) Find a cache", "Enter Freythorn",
    ]})});
    const campaign = screen.getByRole("region",{name:"Campaign objectives"});
    const optional = screen.getByRole("region",{name:"Optional objectives"});
    expect(within(campaign).getByText("Kill The Crowbell +2 Skill Points")).toBeInTheDocument();
    expect(within(campaign).getByText("Enter Freythorn")).toBeInTheDocument();
    expect(within(campaign).queryByText("Talk to Delwyn")).not.toBeInTheDocument();
    expect(within(optional).getByText("Talk to Delwyn")).toBeInTheDocument();
    expect(within(optional).getByText("Find a cache")).toBeInTheDocument();
    expect(screen.queryByText(/\(Opt\)/)).not.toBeInTheDocument();
  });
  it("shows an optional-only side area without an empty campaign card or losing route hints", () => {
    renderBar({model:model({primary:"(Opt) Kill The Devourer",steps_in_zone:["(Opt) Kill The Devourer"],sub_hints:["Parent route: Clearfell"]})});
    expect(screen.queryByRole("region",{name:"Campaign objectives"})).not.toBeInTheDocument();
    expect(within(screen.getByRole("region",{name:"Optional objectives"})).getByText("Kill The Devourer")).toBeInTheDocument();
    expect(screen.getByText("Parent route: Clearfell")).toBeInTheDocument();
  });
  it("does not render an empty optional section", () => {
    renderBar({model:model()});
    expect(screen.queryByRole("region",{name:"Optional objectives"})).not.toBeInTheDocument();
  });
  it("prioritizes campaign guidance in compact mode and summarizes optional items separately", () => {
    renderBar({compact:true,model:model({primary:"(Opt) Find a cache",steps_in_zone:["(Opt) Find a cache","Enter The Grelwood"]})});
    expect(screen.getByText("Enter The Grelwood")).toBeInTheDocument();
    expect(screen.getByText("1 optional")).toBeInTheDocument();
    expect(screen.queryByText("Find a cache")).not.toBeInTheDocument();
  });
  it("labels optional-only compact guidance without the source marker", () => {
    renderBar({compact:true,model:model({primary:"(Opt) Kill The Devourer",steps_in_zone:["(Opt) Kill The Devourer"]})});
    expect(screen.getByText("Optional")).toBeInTheDocument();
    expect(screen.getByText("Kill The Devourer")).toBeInTheDocument();
    expect(screen.queryByText(/\(Opt\)/)).not.toBeInTheDocument();
  });
});

describe("remaining chapter labels", () => {
  it("uses the Interlude label rather than calling it Act 5", () => {
    renderBar({model:model({act:5,zone_name:"Kriar Village"},{chapter_label:"Interlude 5.3"})});
    expect(screen.getByText("Interlude 5.3")).toBeInTheDocument();
    expect(screen.queryByText("ACT 5")).not.toBeInTheDocument();
  });
});


describe("zone navigation", () => {
  it("keeps landmark directions separate from campaign and optional objectives", () => {
    renderBar({ model: model({ steps_in_zone: ["Find Una", "(Opt) Kill Beira"] }, {
      navigation: ["Follow glowing roots to the Tree of Souls."],
    }) });
    const navigation = screen.getByRole("region", { name: "Zone navigation" });
    expect(within(navigation).getByText(/Follow glowing roots/)).toBeInTheDocument();
    expect(within(navigation).queryByText("Find Una")).not.toBeInTheDocument();
    expect(screen.getByRole("region", { name: "Campaign objectives" })).toBeInTheDocument();
    expect(screen.getByRole("region", { name: "Optional objectives" })).toBeInTheDocument();
  });
  it("omits navigation for missing content and in compact mode", () => {
    const view = renderBar({ model: model() });
    expect(screen.queryByRole("region", { name: "Zone navigation" })).not.toBeInTheDocument();
    view.unmount();
    renderBar({ model: model({}, { navigation: ["Follow the river"] }), compact: true });
    expect(screen.queryByRole("region", { name: "Zone navigation" })).not.toBeInTheDocument();
  });
});
