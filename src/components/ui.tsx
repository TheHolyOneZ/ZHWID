import { useState, type ReactNode } from "react";
import { Copy, Check } from "lucide-react";

export function CopyButton({ value }: { value?: string }) {
  const [done, setDone] = useState(false);
  if (!value || value === "—") return null;
  return (
    <button
      type="button"
      className="copybtn"
      title="Copy"
      onClick={async () => {
        try {
          await navigator.clipboard.writeText(value);
          setDone(true);
          setTimeout(() => setDone(false), 1200);
        } catch {

        }
      }}
    >
      {done ? <Check size={13} className="text-[var(--good)]" /> : <Copy size={13} />}
    </button>
  );
}

export function Switch({
  on,
  onChange,
  disabled,
}: {
  on: boolean;
  onChange: (v: boolean) => void;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      className={`switch ${on ? "on" : ""}`}
      onClick={() => !disabled && onChange(!on)}
      aria-pressed={on}
      disabled={disabled}
    >
      <span className="knob" />
    </button>
  );
}

export function Field({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: ReactNode;
  children: ReactNode;
}) {
  return (
    <label className="block">
      <div className="mb-1 flex items-center justify-between">
        <span className="text-[13px] font-medium text-[var(--text)]">{label}</span>
        {hint && <span className="text-[11px] text-[var(--muted)]">{hint}</span>}
      </div>
      {children}
    </label>
  );
}

export function Kind({ kind }: { kind: string }) {
  const label = kind === "wifi" ? "Wi-Fi" : kind === "ethernet" ? "Ethernet" : "Other";
  return (
    <span className="rounded-md border border-[var(--border)] bg-black/20 px-2 py-0.5 text-[10px] uppercase tracking-wide text-[var(--muted)]">
      {label}
    </span>
  );
}
