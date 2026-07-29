import { useCallback, useEffect, useRef, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import {
  Power,
  RotateCcw,
  Network,
  Server,
  RefreshCw,
  Loader2,
  PlugZap,
  Shuffle,
  Layers,
  Zap,
} from "lucide-react";
import {
  applySpoof,
  cloakAvailable,
  generateHostname,
  generateMac,
  generateMachineId,
  getStatus,
  hotkeyLabel,
  macProfiles,
  nmAvailable,
  onLog,
  restoreAll,
  setAutoSpoof as ipcSetAutoSpoof,
  type IdentityStatus,
  type LogEntry,
} from "./lib/ipc";
import { MacRow, type PlannedIface } from "./components/MacRow";
import { ActivityLog } from "./components/ActivityLog";
import { AppCloakPanel } from "./components/AppCloakPanel";
import { TitleBar } from "./components/TitleBar";
import { About } from "./components/About";
import { Switch, CopyButton } from "./components/ui";
import { PRESETS, presetById, type Preset } from "./lib/presets";
import { listen } from "@tauri-apps/api/event";

const PREFS_KEY = "zhwid-prefs";
interface Prefs {
  setMid?: boolean;
  setHost?: boolean;
  setDhcp?: boolean;
  auto?: boolean;
  preset?: string;
  ifaces?: { name: string; profile: string; enabled: boolean }[];
}
const loadPrefs = (): Prefs => {
  try {
    return JSON.parse(localStorage.getItem(PREFS_KEY) || "{}");
  } catch {
    return {};
  }
};

export default function App() {
  const [status, setStatus] = useState<IdentityStatus | null>(null);
  const [profiles, setProfiles] = useState<[string, string][]>([]);
  const [cloakOk, setCloakOk] = useState(false);
  const [nmOk, setNmOk] = useState(false);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showAbout, setShowAbout] = useState(false);

  const prefs = useRef(loadPrefs());
  const [planned, setPlanned] = useState<PlannedIface[]>([]);
  const [machineId, setMachineId] = useState("");
  const [setMid, setSetMid] = useState(prefs.current.setMid ?? true);
  const [hostname, setHostname] = useState("");
  const [setHost, setSetHost] = useState(prefs.current.setHost ?? true);
  const [setDhcp, setSetDhcp] = useState(prefs.current.setDhcp ?? true);
  const [preset, setPreset] = useState(prefs.current.preset ?? "balanced");
  const [autoSpoof, setAutoSpoof] = useState(prefs.current.auto ?? false);
  const [hotkey, setHotkey] = useState("Ctrl+Alt+S");
  const initialized = useRef(false);
  const lastAuto = useRef(0);
  const handlers = useRef({
    toggle: () => {},
    applyOn: (_pre: boolean) => {},
    active: false,
    busy: false,
    autoSpoof: false,
  });

  const active = status?.active ?? false;
  const locked = busy || active;

  const refresh = useCallback(async () => {
    const [st, profs, ck, nm, hk] = await Promise.all([
      getStatus(),
      macProfiles(),
      cloakAvailable(),
      nmAvailable(),
      hotkeyLabel().catch(() => "Ctrl+Alt+S"),
    ]);
    setStatus(st);
    setProfiles(profs);
    setCloakOk(ck);
    setNmOk(nm);
    setHotkey(hk);

    if (!initialized.current) {
      initialized.current = true;
      const saved = prefs.current.ifaces ?? [];
      const profileFor = (name: string) =>
        saved.find((s) => s.name === name)?.profile ?? presetById(preset)?.macProfile ?? "random";
      const macs = await Promise.all(
        st.interfaces.map((iface) => generateMac(profileFor(iface.name))),
      );
      setPlanned(
        st.interfaces.map((iface, i) => ({
          name: iface.name,
          enabled: saved.find((s) => s.name === iface.name)?.enabled ?? true,
          profile: profileFor(iface.name),
          mac: macs[i],
        })),
      );
      setMachineId(await generateMachineId());
      setHostname(await generateHostname());
    }

  }, []);

  useEffect(() => {
    const unsubs: (() => void)[] = [];
    onLog((e) => setLogs((l) => [...l.slice(-200), e])).then((f) => unsubs.push(f));

    listen("zhwid://tray-restore", () => {
      restoreAll()
        .then(setStatus)
        .catch((e) => setError(String(e)));
    }).then((f) => unsubs.push(f));

    listen("zhwid://hotkey-toggle", () => handlers.current.toggle()).then((f) =>
      unsubs.push(f),
    );

    listen("zhwid://auto-trigger", () => {
      const h = handlers.current;
      if (!h.autoSpoof || h.active || h.busy) return;
      const now = Date.now();
      if (now - lastAuto.current < 8000) return;
      lastAuto.current = now;
      h.applyOn(true);
    }).then((f) => unsubs.push(f));
    refresh().catch((e) => setError(String(e)));
    return () => unsubs.forEach((f) => f());
  }, [refresh]);

  const updatePlan = (name: string, patch: Partial<PlannedIface>) =>
    setPlanned((ps) => ps.map((p) => (p.name === name ? { ...p, ...patch } : p)));

  const regen = async (name: string, profile: string) => {
    const mac = await generateMac(profile);
    updatePlan(name, { profile, mac });
  };


  useEffect(() => {
    const p: Prefs = {
      setMid,
      setHost,
      setDhcp,
      auto: autoSpoof,
      preset,
      ifaces: planned.map((x) => ({ name: x.name, profile: x.profile, enabled: x.enabled })),
    };
    localStorage.setItem(PREFS_KEY, JSON.stringify(p));
  }, [setMid, setHost, setDhcp, autoSpoof, preset, planned]);


  useEffect(() => {
    ipcSetAutoSpoof(autoSpoof).catch(() => {});
  }, [autoSpoof]);

  const applyPreset = async (p: Preset) => {
    setPreset(p.id);
    setSetMid(p.machineId);
    setSetHost(p.hostnameOn);
    setSetDhcp(p.dhcp);
    if (p.hostnameOn) setHostname(p.hostname());
    if (p.machineId) setMachineId(await generateMachineId());
    const macs = await Promise.all(planned.map(() => generateMac(p.macProfile)));
    setPlanned((ps) => ps.map((x, i) => ({ ...x, profile: p.macProfile, mac: macs[i] })));
  };

  const rerollAll = async () => {
    const macs = await Promise.all(planned.map((p) => generateMac(p.profile)));
    setPlanned((ps) => ps.map((x, i) => ({ ...x, mac: macs[i] })));
    setMachineId(await generateMachineId());
    const cur = presetById(preset);
    setHostname(cur && cur.hostnameOn ? cur.hostname() : await generateHostname());
  };

  const applyOn = async (preConnect: boolean) => {
    setBusy(true);
    setError(null);
    try {
      setStatus(
        await applySpoof({
          interfaces: planned
            .filter((p) => p.enabled)
            .map((p) => ({ name: p.name, mac: p.mac })),
          set_machine_id: setMid,
          machine_id: machineId,
          set_hostname: setHost,
          hostname,
          pre_connect: preConnect,
          spoof_dhcp: setDhcp && nmOk,
        }),
      );
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const restore = async () => {
    setBusy(true);
    setError(null);
    try {
      setStatus(await restoreAll());
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const toggle = () => (active ? restore() : applyOn(false));


  handlers.current = { toggle, applyOn, active, busy, autoSpoof };

  const changeCount =
    planned.filter((p) => p.enabled).length +
    (setMid ? 1 : 0) +
    (setHost ? 1 : 0) +
    (setDhcp && nmOk ? 1 : 0);

  return (
    <div className="flex h-screen flex-col text-[var(--text)]">
      <About open={showAbout} onClose={() => setShowAbout(false)} />
      <TitleBar active={active} onAbout={() => setShowAbout(true)} />

      <div className="grid min-h-0 flex-1 grid-cols-[1fr_340px] gap-4 p-4">

        <main className="scroll min-h-0 space-y-4 overflow-y-auto pr-1">
          <div className="glass flex flex-wrap items-center gap-2 p-3">
            <div className="flex items-center gap-1.5 pr-1 text-[12px] text-[var(--muted)]">
              <Layers size={14} /> Preset
            </div>
            {PRESETS.map((p) => (
              <button
                key={p.id}
                onClick={() => applyPreset(p)}
                disabled={locked}
                title={p.description}
                className={`btn !py-1.5 !text-[12px] ${preset === p.id ? "btn-accent" : ""}`}
              >
                {p.label}
              </button>
            ))}
            <button
              className="btn !py-1.5 !text-[12px] ml-auto"
              onClick={rerollAll}
              disabled={locked}
              title="Regenerate every planned identity"
            >
              <Shuffle size={13} /> Reroll all
            </button>
            <div className="basis-full text-[11px] text-[var(--muted)]">
              {presetById(preset)?.description}
            </div>
          </div>

          <HeroToggle
            active={active}
            busy={busy}
            changeCount={changeCount}
            chips={[
              "MAC address",
              "machine-id",
              "hostname",
              ...(setDhcp && nmOk ? ["DHCP client-ID + DUID"] : []),
            ]}
            onToggle={toggle}
            onPreConnect={() => applyOn(true)}
          />

          {error && (
            <div className="glass border-[color-mix(in_srgb,var(--bad)_40%,transparent)] p-3 text-[13px] text-[var(--bad)]">
              {error}
            </div>
          )}


          <Section icon={<Network size={16} />} title="Network Interfaces" subtitle="MAC address">
            <div className="space-y-3">
              {status?.interfaces.map((iface) => {
                const plan = planned.find((p) => p.name === iface.name);
                if (!plan) return null;
                return (
                  <MacRow
                    key={iface.name}
                    info={iface}
                    plan={plan}
                    profiles={profiles}
                    locked={locked}
                    onToggle={(v) => updatePlan(iface.name, { enabled: v })}
                    onProfile={(p) => regen(iface.name, p)}
                    onRegenerate={() => regen(iface.name, plan.profile)}
                    onEdit={(mac) => updatePlan(iface.name, { mac })}
                  />
                );
              })}
              {status && status.interfaces.length === 0 && (
                <div className="glass p-4 text-[13px] text-[var(--muted)]">
                  No physical network interfaces found.
                </div>
              )}
            </div>
          </Section>


          <Section
            icon={<Server size={16} />}
            title="System Identity"
            subtitle="machine-id · hostname · DHCP"
          >
            <div className="glass space-y-4 p-4">
              <ToggleField
                label="Machine ID"
                real={status?.machine_id}
                on={setMid}
                onToggle={setSetMid}
                locked={locked}
                value={machineId}
                onChange={setMachineId}
                onRegen={async () => setMachineId(await generateMachineId())}
                mono
              />
              <div className="h-px bg-[var(--border)]" />
              <ToggleField
                label="Hostname"
                real={status?.hostname}
                on={setHost}
                onToggle={setSetHost}
                locked={locked}
                value={hostname}
                onChange={setHostname}
                onRegen={async () => setHostname(await generateHostname())}
              />
              <div className="h-px bg-[var(--border)]" />
              <div>
                <div className="mb-1 flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <span className="text-[14px] font-semibold">DHCP identity</span>
                    <Switch
                      on={setDhcp && nmOk}
                      onChange={setSetDhcp}
                      disabled={locked || !nmOk}
                    />
                  </div>
                  <span className="mono text-[11px] text-[var(--muted)]">client-ID + DUID</span>
                </div>
                <p className="text-[11px] leading-snug text-[var(--muted)]">
                  {nmOk
                    ? "Randomizes the DHCPv4 client-ID and DHCPv6 DUID your connection sends, so a network can't re-identify you by them even after a MAC change. Restored on OFF."
                    : "NetworkManager not detected — DHCP identity spoofing is unavailable on this system."}
                </p>
              </div>
            </div>
          </Section>

          <AppCloakPanel available={cloakOk} onError={setError} />
        </main>


        <aside className="flex min-h-0 flex-col gap-4">
          <div className="glass p-4">
            <div className="mb-3 flex items-center gap-2">
              <Zap size={15} className="text-[var(--accent)]" />
              <span className="text-[13px] font-semibold">Automation</span>
            </div>
            <div className="flex items-center justify-between gap-3">
              <div className="min-w-0">
                <div className="text-[13px] font-medium">Auto-spoof new networks</div>
                <p className="text-[11px] leading-snug text-[var(--muted)]">
                  {nmOk
                    ? "Pre-connect spoof automatically whenever an interface joins a network."
                    : "Needs NetworkManager."}
                </p>
              </div>
              <Switch on={autoSpoof && nmOk} onChange={setAutoSpoof} disabled={!nmOk} />
            </div>
            <div className="mt-3 flex items-center justify-between border-t border-[var(--border)] pt-3">
              <span className="text-[12px] text-[var(--muted)]">Toggle hotkey</span>
              <kbd className="mono rounded-md border border-[var(--border)] bg-black/25 px-2 py-0.5 text-[11px]">
                {hotkey}
              </kbd>
            </div>
          </div>
          <div className="min-h-0 flex-1">
            <ActivityLog entries={logs} />
          </div>
          <button
            className="btn btn-danger justify-center py-3"
            onClick={restore}
            disabled={busy}
            title="Force-restore original identity from backup"
          >
            <RotateCcw size={15} /> Restore original identity
          </button>
        </aside>
      </div>
    </div>
  );
}

function HeroToggle({
  active,
  busy,
  changeCount,
  chips,
  onToggle,
  onPreConnect,
}: {
  active: boolean;
  busy: boolean;
  changeCount: number;
  chips: string[];
  onToggle: () => void;
  onPreConnect: () => void;
}) {
  return (
    <div className="glass p-6">
      <div className="flex items-center gap-6">
        <button
          className={`master ${active ? "on" : "off"}`}
          onClick={onToggle}
          disabled={busy}
        >
          {busy ? (
            <Loader2 size={40} className="animate-spin text-[var(--accent)]" />
          ) : (
            <Power size={44} style={{ color: active ? "var(--good)" : "var(--bad)" }} />
          )}
        </button>
        <div className="min-w-0 flex-1">
          <AnimatePresence mode="wait">
            <motion.div
              key={active ? "on" : "off"}
              initial={{ opacity: 0, y: 6 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -6 }}
              transition={{ duration: 0.2 }}
            >
              <div className="text-[22px] font-bold">
                {active ? "Identity Protected" : "Real Identity Exposed"}
              </div>
              <p className="mt-1 max-w-md text-[13px] leading-relaxed text-[var(--muted)]">
                {busy
                  ? "Waiting for authorization…"
                  : active
                    ? "Your MAC, machine-id and hostname are spoofed. Click to restore your real identity."
                    : `Click to spoof ${changeCount} identifier${changeCount === 1 ? "" : "s"}. A system authorization prompt will appear.`}
              </p>
            </motion.div>
          </AnimatePresence>
        </div>
      </div>

      <div className="mt-4 flex flex-wrap items-center gap-2">
        <span className="text-[11px] uppercase tracking-wide text-[var(--muted)]">
          System-wide spoof
        </span>
        {chips.map((t) => (
          <span
            key={t}
            className="mono rounded-md border border-[var(--border)] bg-black/25 px-2 py-0.5 text-[11px] text-[var(--text)]"
          >
            {t}
          </span>
        ))}
      </div>

      {!active && (
        <div className="mt-4 flex items-center gap-3 rounded-xl border border-[var(--border)] bg-black/20 p-3">
          <PlugZap size={18} className="flex-none text-[var(--accent)]" />
          <div className="min-w-0 flex-1">
            <div className="text-[13px] font-semibold">Spoof before connecting</div>
            <p className="text-[11px] leading-snug text-[var(--muted)]">
              Disconnects Wi-Fi/Ethernet first, then spoofs while offline — so the network's
              first sight of you is already fake. Interfaces are left disconnected; reconnect
              when ready.
            </p>
          </div>
          <button
            className="btn btn-accent flex-none"
            onClick={onPreConnect}
            disabled={busy}
          >
            <PlugZap size={14} /> Disconnect &amp; spoof
          </button>
        </div>
      )}
    </div>
  );
}

function Section({
  icon,
  title,
  subtitle,
  children,
}: {
  icon: React.ReactNode;
  title: string;
  subtitle: string;
  children: React.ReactNode;
}) {
  return (
    <section>
      <div className="mb-2 flex items-center gap-2 px-1">
        <span className="text-[var(--accent)]">{icon}</span>
        <h2 className="text-[14px] font-semibold">{title}</h2>
        <span className="text-[11px] text-[var(--muted)]">· {subtitle}</span>
      </div>
      {children}
    </section>
  );
}

function ToggleField({
  label,
  real,
  on,
  onToggle,
  locked,
  value,
  onChange,
  onRegen,
  mono,
}: {
  label: string;
  real?: string;
  on: boolean;
  onToggle: (v: boolean) => void;
  locked: boolean;
  value: string;
  onChange: (v: string) => void;
  onRegen: () => void;
  mono?: boolean;
}) {
  return (
    <div>
      <div className="mb-2 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className="text-[14px] font-semibold">{label}</span>
          <Switch on={on} onChange={onToggle} disabled={locked} />
        </div>
        <span className="mono flex items-center gap-1 text-[11px] text-[var(--muted)]">
          real {real ?? "—"}
          <CopyButton value={real} />
        </span>
      </div>
      {on && (
        <div className="flex gap-2">
          <input
            className={mono ? "mono flex-1" : "flex-1"}
            value={value}
            spellCheck={false}
            disabled={locked}
            onChange={(e) => onChange(e.target.value)}
          />
          <CopyButton value={value} />
          <button className="btn" onClick={onRegen} disabled={locked} title="Regenerate">
            <RefreshCw size={14} />
          </button>
        </div>
      )}
    </div>
  );
}
