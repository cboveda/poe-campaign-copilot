import { invoke } from "@tauri-apps/api/core";
import { useLayoutEffect, useRef, useState } from "react";

export interface ActionBounds { x: number; y: number; width: number; height: number }

// Register only a fully visible button, never its containing overlay/card.
export function visibleActionBounds(rect: DOMRect, viewportWidth: number, viewportHeight: number): ActionBounds | null {
  if (![rect.left, rect.top, rect.width, rect.height, viewportWidth, viewportHeight].every(Number.isFinite)
    || rect.left < 0 || rect.top < 0 || rect.width < 1 || rect.width > 420
    || rect.height < 1 || rect.height > 100
    || rect.right > viewportWidth || rect.bottom > viewportHeight) return null;
  return {x:rect.left, y:rect.top, width:rect.width, height:rect.height};
}

export function TownReturnButton({onConfirm, setupMode}: {onConfirm: () => Promise<void>; setupMode: boolean}) {
  const button = useRef<HTMLButtonElement>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  useLayoutEffect(() => {
    const send = (bounds: ActionBounds | null) => {
      void invoke("set_overlay_action_bounds", {bounds}).catch((reason) => {
        console.error("set_overlay_action_bounds failed:", reason);
      });
    };
    if (setupMode || busy) { send(null); return; }
    const report = () => {
      const node = button.current;
      let bounds = node ? visibleActionBounds(node.getBoundingClientRect(), window.innerWidth, window.innerHeight) : null;
      const container = node?.closest(".filmstrip");
      if (bounds && container) {
        const clip = container.getBoundingClientRect();
        if (bounds.y < clip.top || bounds.y + bounds.height > clip.bottom) bounds = null;
      }
      send(bounds);
    };
    const observer = typeof ResizeObserver !== "undefined" ? new ResizeObserver(report) : null;
    if (button.current) {
      observer?.observe(button.current);
      const container = button.current.closest(".filmstrip");
      if (container) observer?.observe(container);
    }
    window.addEventListener("resize", report);
    document.addEventListener("scroll", report, true);
    // Refresh the native expiry, so a crashed/reloaded webview fails closed.
    const refresh = window.setInterval(report, 250);
    report();
    return () => {
      observer?.disconnect();
      window.clearInterval(refresh);
      window.removeEventListener("resize", report);
      document.removeEventListener("scroll", report, true);
      send(null);
    };
  }, [setupMode, busy]);

  async function confirm() {
    if (busy) return;
    setBusy(true); setError(null);
    try { await onConfirm(); }
    catch (reason) {setError(String(reason));}
    finally {setBusy(false);}
  }
  return <div className="town-return-action">
    <button ref={button} type="button" className="town-return-button"
      disabled={busy || setupMode} onClick={() => void confirm()}
      title="Confirm only after completing the objectives in the zone you left">
      {busy ? "Confirming…" : "Continue town step"}
    </button>
    <span className="town-return-help">Only after completing the previous zone’s objectives.</span>
    {error && <span role="alert" className="town-return-error">{error}</span>}
  </div>;
}
