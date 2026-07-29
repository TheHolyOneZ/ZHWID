import { useEffect, useState } from "react";
import { Ghost, Play, RefreshCw, Rocket } from "lucide-react";
import {
  launchCloaked,
  suggestHardwareProfile,
  type HardwareProfile,
} from "../lib/ipc";
import { Field, CopyButton } from "./ui";

const QUICK_APPS = ["gnome-terminal", "konsole", "xterm", "kitty", "alacritty"];

const FIELDS: [keyof HardwareProfile, string][] = [
  ["sys_vendor", "sys_vendor"],
  ["product_name", "product_name"],
  ["product_uuid", "product_uuid"],
  ["product_serial", "product_serial"],
  ["board_vendor", "board_vendor"],
  ["board_name", "board_name"],
  ["board_serial", "board_serial"],
  ["bios_vendor", "bios_vendor"],
  ["bios_version", "bios_version"],
  ["chassis_serial", "chassis_serial"],
  ["disk_serial", "disk_serial"],
  ["machine_id", "machine_id"],
];

export function AppCloakPanel({
  available,
  onError,
}: {
  available: boolean;
  onError: (msg: string) => void;
}) {
  const [app, setApp] = useState("");
  const [profile, setProfile] = useState<HardwareProfile | null>(null);
  const [busy, setBusy] = useState(false);

  const roll = () => suggestHardwareProfile().then(setProfile).catch(() => {});
  useEffect(() => {
    roll();
  }, []);

  const launch = async () => {
    if (!app.trim() || !profile) return;
    setBusy(true);
    try {
      await launchCloaked(app.trim(), profile);
    } catch (e) {
      onError(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="glass p-5">
      <div className="mb-4 flex items-center gap-2">
        <Ghost size={16} className="text-[var(--accent-2)]" />
        <h3 className="text-[14px] font-semibold">App Cloak</h3>
        <span className="ml-auto text-[11px] text-[var(--muted)]">
          per-app firmware IDs · leaves no trace
        </span>
      </div>

      {!available ? (
        <div className="rounded-lg border border-[var(--border)] bg-black/20 p-3 text-[12px] text-[var(--muted)]">
          Cloak library unavailable (libzcloak.so not built).
        </div>
      ) : (
        <div className="space-y-4">
          <p className="text-[12px] leading-relaxed text-[var(--muted)]">
            Launch an app that reads a fake motherboard / BIOS / disk identity &amp; machine-id.
            Only the launched process is affected — nothing on disk changes.
          </p>

          <Field label="Application">
            <div className="flex gap-2">
              <input
                className="mono flex-1"
                placeholder="e.g. gnome-terminal"
                value={app}
                spellCheck={false}
                onChange={(e) => setApp(e.target.value)}
              />
              <button className="btn btn-accent" onClick={launch} disabled={busy || !app.trim()}>
                {busy ? <Rocket size={14} /> : <Play size={14} />} Launch
              </button>
            </div>
          </Field>

          <div className="flex flex-wrap gap-1.5">
            {QUICK_APPS.map((a) => (
              <button key={a} className="btn !py-1 !text-[12px]" onClick={() => setApp(a)}>
                {a}
              </button>
            ))}
          </div>

          <div className="rounded-xl border border-[var(--border)] bg-black/20 p-3">
            <div className="mb-2 flex items-center justify-between">
              <span className="text-[12px] font-medium">
                Spoofed identity
                {profile && (
                  <span className="ml-2 text-[var(--accent-2)]">
                    {profile.sys_vendor} · {profile.product_name}
                  </span>
                )}
              </span>
              <button className="btn !py-1 !text-[12px]" onClick={roll}>
                <RefreshCw size={13} /> Reroll
              </button>
            </div>
            <dl className="mono grid grid-cols-1 gap-x-6 gap-y-1 text-[11px] md:grid-cols-2">
              {FIELDS.map(([key, label]) => (
                <div key={key} className="flex items-center justify-between gap-2">
                  <dt className="text-[var(--muted)]">{label}</dt>
                  <dd className="flex min-w-0 items-center gap-1">
                    <span className="truncate text-[var(--text)]" title={profile?.[key]}>
                      {profile?.[key] ?? "—"}
                    </span>
                    <CopyButton value={profile?.[key]} />
                  </dd>
                </div>
              ))}
            </dl>
          </div>
        </div>
      )}
    </div>
  );
}
