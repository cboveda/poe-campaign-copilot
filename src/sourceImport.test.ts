import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { previewSourceChunks } from "./sourceImport";
const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
beforeEach(() => {
  invoke.mockReset();
  invoke.mockImplementation(async command => command === "begin_source_import" ? "token" : command === "finish_source_import" ? {title: "Build"} : undefined);
});
afterEach(() => { vi.useRealTimers(); });
describe("source transfer", () => {
  it("uses small messages, preserves Unicode boundaries and reports progress", async () => {
    const source = "x".repeat(16_383) + "🐇" + "y".repeat(40_000);
    const progress = vi.fn();
    await previewSourceChunks("url", source, progress);
    const chunks: string[] = invoke.mock.calls.filter(c => c[0] === "append_source_import").map(c => c[1].chunk);
    expect(chunks.length).toBeGreaterThan(1);
    expect(chunks.join("")).toBe(source);
    expect(chunks.every(chunk => new TextEncoder().encode(chunk).length <= 64 * 1024)).toBe(true);
    expect(chunks[0]).not.toMatch(/[\ud800-\udbff]$/);
    expect(progress).toHaveBeenCalledWith("Parsing build stages…");
  });
  it("fails with a restart instruction when the native app never responds", async () => {
    vi.useFakeTimers();
    invoke.mockImplementation(() => new Promise(() => {}));
    const result = expect(previewSourceChunks("url", "source", vi.fn())).rejects.toThrow("within 20 seconds");
    await vi.advanceTimersByTimeAsync(20_000);
    await result;
  });
  it("cancels an incomplete transfer after a chunk fails", async () => {
    invoke.mockImplementation(async command => {
      if (command === "begin_source_import") return "token";
      if (command === "append_source_import") throw new Error("Transfer failed");
    });
    await expect(previewSourceChunks("url", "source", vi.fn())).rejects.toThrow("Transfer failed");
    expect(invoke).toHaveBeenCalledWith("cancel_source_import", {token: "token"});
  });
});
