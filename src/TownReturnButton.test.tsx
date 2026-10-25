import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { TownReturnButton, visibleActionBounds } from "./TownReturnButton";
vi.mock("@tauri-apps/api/core", () => ({invoke: vi.fn().mockResolvedValue(undefined)}));
beforeEach(() => {vi.mocked(invoke).mockClear();});

describe("overlay town return", () => {
  it("requires a click and suppresses repeat clicks during acknowledgment", async () => {
    let finish!: () => void;
    const onConfirm = vi.fn(() => new Promise<void>(resolve => {finish = resolve;}));
    render(<TownReturnButton setupMode={false} onConfirm={onConfirm} />);
    const button = screen.getByRole("button", {name:/continue town step/i});
    expect(onConfirm).not.toHaveBeenCalled();
    fireEvent.click(button);
    fireEvent.click(button);
    expect(onConfirm).toHaveBeenCalledTimes(1);
    expect(button).toBeDisabled();
    finish();
    await waitFor(() => expect(button).not.toBeDisabled());
  });
  it("shows a failure and allows retry without changing route state locally", async () => {
    const onConfirm = vi.fn().mockRejectedValue(new Error("No planned town return"));
    render(<TownReturnButton setupMode={false} onConfirm={onConfirm} />);
    fireEvent.click(screen.getByRole("button"));
    expect(await screen.findByRole("alert")).toHaveTextContent("No planned town return");
    expect(screen.getByRole("button")).not.toBeDisabled();
  });
  it("does not confirm while moving/resizing in setup mode", () => {
    const onConfirm = vi.fn();
    render(<TownReturnButton setupMode onConfirm={onConfirm} />);
    fireEvent.click(screen.getByRole("button"));
    expect(onConfirm).not.toHaveBeenCalled();
    expect(invoke).toHaveBeenCalledWith("set_overlay_action_bounds", {bounds:null});
  });
  it("refreshes only the button rectangle and unregisters it on removal", async () => {
    const spy = vi.spyOn(HTMLButtonElement.prototype, "getBoundingClientRect")
      .mockReturnValue(new DOMRect(10,20,180,34));
    try {
      const {unmount} = render(<TownReturnButton setupMode={false} onConfirm={vi.fn().mockResolvedValue(undefined)} />);
      expect(invoke).toHaveBeenCalledWith("set_overlay_action_bounds", {bounds:{x:10,y:20,width:180,height:34}});
      unmount();
      expect(vi.mocked(invoke).mock.calls.at(-1)).toEqual(["set_overlay_action_bounds",{bounds:null}]);
    } finally {spy.mockRestore();}
  });
});

describe("action geometry", () => {
  it("rejects clipped, oversized and nonfinite hit regions", () => {
    expect(visibleActionBounds(new DOMRect(10,20,180,34),960,150)).toEqual({x:10,y:20,width:180,height:34});
    for (const rect of [new DOMRect(-1,20,180,34),new DOMRect(10,149,180,34),
      new DOMRect(10,20,500,34),new DOMRect(10,20,180,101),new DOMRect(NaN,20,180,34)]) {
      expect(visibleActionBounds(rect,960,150)).toBeNull();
    }
  });
});
