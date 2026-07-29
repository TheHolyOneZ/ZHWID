import { RefreshCw, Wifi, Cable } from "lucide-react";
import type { InterfaceInfo } from "../lib/ipc";
import { Switch, Kind, CopyButton } from "./ui";

export interface PlannedIface {
  name: string;
  enabled: boolean;
  profile: string;
  mac: string;
}

export function MacRow({
  info,
  plan,
  profiles,
  locked,
  onToggle,
  onProfile,
  onRegenerate,
  onEdit,
}: {
  info: InterfaceInfo;
  plan: PlannedIface;
  profiles: [string, string][];
  locked: boolean;
  onToggle: (v: boolean) => void;
  onProfile: (p: string) => void;
  onRegenerate: () => void;
  onEdit: (mac: string) => void;
}) {
  const Icon = info.kind === "wifi" ? Wifi : Cable;
  return (
    <div className="glass glass-hover p-4">
      <div className="flex items-center justify-between gap-3">
        <div className="flex items-center gap-3">
          <div className="grid h-9 w-9 place-items-center rounded-lg border border-[var(--border)] bg-black/25">
            <Icon size={17} className="text-[var(--accent)]" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <span className="mono text-[14px] font-semibold">{info.name}</span>
              <Kind kind={info.kind} />
              {info.spoofed && (
                <span className="rounded-md bg-[color-mix(in_srgb,var(--good)_18%,transparent)] px-2 py-0.5 text-[10px] font-semibold text-[var(--good)]">
                  SPOOFED
                </span>
              )}
            </div>
            <div className="mono mt-0.5 text-[11px] text-[var(--muted)]">
              real {info.perm_mac || "—"}
            </div>
          </div>
        </div>
        <Switch on={plan.enabled} onChange={onToggle} disabled={locked} />
      </div>

      {plan.enabled && (
        <div className="mt-3 flex flex-wrap items-center gap-2">
          <select
            value={plan.profile}
            onChange={(e) => onProfile(e.target.value)}
            disabled={locked}
            className="field !w-auto text-[13px]"
          >
            {profiles.map(([id, label]) => (
              <option key={id} value={id}>
                {label}
              </option>
            ))}
          </select>
          <input
            className="mono flex-1 !w-auto min-w-[200px] text-[13px]"
            value={plan.mac}
            spellCheck={false}
            disabled={locked}
            onChange={(e) => onEdit(e.target.value)}
          />
          <CopyButton value={plan.mac} />
          <button
            type="button"
            className="btn"
            onClick={onRegenerate}
            disabled={locked}
            title="Generate new MAC"
          >
            <RefreshCw size={14} />
          </button>
        </div>
      )}
    </div>
  );
}
