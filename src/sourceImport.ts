import { invoke } from "@tauri-apps/api/core";
import type { BuildPlan } from "./types";

async function bounded<T>(command: string, args: Record<string, unknown>): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  try {
    return await Promise.race([
      invoke<T>(command, args),
      new Promise<never>((_, reject) => {
        timer = setTimeout(() => reject(new Error("The app did not respond to source import within 20 seconds. Stop it with Ctrl+C, restart with npm.cmd run tauri dev, and try again.")), 20_000);
      }),
    ]);
  } finally { if (timer !== undefined) clearTimeout(timer); }
}
export async function previewSourceChunks(url: string, html: string, progress: (message: string) => void): Promise<BuildPlan> {
  if (new TextEncoder().encode(html).length > 8 * 1024 * 1024) throw new Error("Page source exceeds the 8 MiB limit.");
  progress("Starting source import…");
  const token = await bounded<string>("begin_source_import", {url});
  let success = false;
  try {
    for (let offset = 0; offset < html.length;) {
      let end = Math.min(offset + 16_384, html.length);
      // Never split a UTF-16 surrogate pair across separate JSON messages.
      const code = html.charCodeAt(end - 1);
      if (end < html.length && code >= 0xd800 && code <= 0xdbff) end--;
      await bounded<void>("append_source_import", {token, chunk: html.slice(offset, end)});
      offset = end;
      progress(`Reading page source… ${Math.round(offset / html.length * 100)}%`);
    }
    progress("Parsing build stages…");
    const plan = await bounded<BuildPlan>("finish_source_import", {token});
    success = true;
    return plan;
  } finally {
    if (!success) void invoke("cancel_source_import", {token}).catch(() => {});
  }
}
