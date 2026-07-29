import { CheckCircle2, Info, AlertTriangle, XCircle, Terminal } from "lucide-react";
import type { LogEntry } from "../lib/ipc";

const ICONS = {
  success: CheckCircle2,
  info: Info,
  warn: AlertTriangle,
  error: XCircle,
} as const;

const COLORS = {
  success: "var(--good)",
  info: "var(--muted)",
  warn: "var(--warn)",
  error: "var(--bad)",
} as const;

export function ActivityLog({ entries }: { entries: LogEntry[] }) {
  return (
    <div className="glass flex h-full flex-col overflow-hidden">
      <div className="flex items-center gap-2 border-b border-[var(--border)] px-4 py-3">
        <Terminal size={15} className="text-[var(--accent)]" />
        <span className="text-[13px] font-semibold">Activity</span>
      </div>
      <div className="scroll flex-1 space-y-2 overflow-y-auto p-3">
        {entries.length === 0 && (
          <div className="px-1 py-6 text-center text-[12px] text-[var(--muted)]">
            No activity yet.
          </div>
        )}
        {entries.map((e, i) => {
          const Icon = ICONS[e.level] ?? Info;
          const color = COLORS[e.level] ?? "var(--muted)";
          const time = new Date(e.ts * 1000).toLocaleTimeString();
          return (
            <div key={i} className="flex items-start gap-2 text-[12px] leading-snug">
              <Icon size={14} style={{ color }} className="mt-0.5 flex-none" />
              <div className="min-w-0">
                <span className="mono text-[10px] text-[var(--muted)]">{time} </span>
                <span className="break-words text-[var(--text)]">{e.message}</span>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
