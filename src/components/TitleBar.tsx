import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  Minus,
  Square,
  X,
  Fingerprint,
  Info,
  ShieldCheck,
  ShieldAlert,
} from "lucide-react";

const appWindow = getCurrentWindow();

export function TitleBar({ active, onAbout }: { active: boolean; onAbout: () => void }) {
  return (
    <div data-tauri-drag-region className="titlebar">
      <div data-tauri-drag-region className="flex items-center gap-2.5">
        <div className="grid h-6 w-6 place-items-center rounded-md border border-[var(--border-strong)] bg-black/30">
          <Fingerprint size={13} className="text-[var(--accent)]" />
        </div>
        <span data-tauri-drag-region className="accent-text text-[13px] font-bold">
          Z-HWID
        </span>
        <span data-tauri-drag-region className="text-[11px] text-[var(--muted)]">
          Linux Identity Switch
        </span>
      </div>

      <div data-tauri-drag-region className="flex flex-1 justify-center">
        <StatusPill active={active} />
      </div>

      <div className="flex items-center">
        <button className="winbtn" onClick={onAbout} title="About">
          <Info size={15} />
        </button>
        <button className="winbtn" onClick={() => appWindow.minimize()} title="Minimize">
          <Minus size={15} />
        </button>
        <button
          className="winbtn"
          onClick={() => appWindow.toggleMaximize()}
          title="Maximize"
        >
          <Square size={12} />
        </button>
        <button className="winbtn winbtn-close" onClick={() => appWindow.close()} title="Close">
          <X size={15} />
        </button>
      </div>
    </div>
  );
}

function StatusPill({ active }: { active: boolean }) {
  return (
    <div
      className="flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-[11px] font-semibold"
      style={{
        borderColor: active
          ? "color-mix(in srgb, var(--good) 45%, transparent)"
          : "color-mix(in srgb, var(--bad) 45%, transparent)",
        color: active ? "var(--good)" : "var(--bad)",
        background: active
          ? "color-mix(in srgb, var(--good) 10%, transparent)"
          : "color-mix(in srgb, var(--bad) 10%, transparent)",
      }}
    >
      {active ? <ShieldCheck size={13} /> : <ShieldAlert size={13} />}
      {active ? "Protected" : "Exposed"}
    </div>
  );
}
