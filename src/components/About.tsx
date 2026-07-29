import { openUrl } from "@tauri-apps/plugin-opener";
import { X, Fingerprint, Download, Github, Globe, ExternalLink } from "lucide-react";

const LINKS = [
  {
    icon: Download,
    label: "Download & landing page",
    sub: "zsync.eu/zhwid",
    url: "https://zsync.eu/zhwid/",
  },
  {
    icon: Github,
    label: "Source code",
    sub: "github.com/TheHolyOneZ/ZHWID",
    url: "https://github.com/TheHolyOneZ/ZHWID",
  },
  {
    icon: Globe,
    label: "More projects",
    sub: "zsync.eu",
    url: "https://zsync.eu",
  },
  {
    icon: Github,
    label: "GitHub",
    sub: "TheHolyOneZ",
    url: "https://github.com/TheHolyOneZ",
  },
];

export function About({ open, onClose }: { open: boolean; onClose: () => void }) {
  if (!open) return null;
  return (
    <div className="about-backdrop" onClick={onClose}>
      <div className="about-card glass" onClick={(e) => e.stopPropagation()}>
        <button className="winbtn about-close" onClick={onClose} title="Close">
          <X size={16} />
        </button>

        <div className="about-head">
          <div className="about-badge">
            <Fingerprint size={26} className="text-[var(--accent)]" />
          </div>
          <div>
            <div className="text-[20px] font-bold leading-none">
              <span className="accent-text">Z-HWID</span>
            </div>
            <div className="mt-1 text-[12px] text-[var(--muted)]">
              Linux Identity Switch · v0.1.0
            </div>
          </div>
        </div>

        <p className="about-tagline">
          Spoof your MAC address, machine-id, hostname &amp; DHCP identity — and restore it
          anytime.
        </p>

        <div className="about-links">
          {LINKS.map((l) => (
            <button key={l.url} className="about-link" onClick={() => openUrl(l.url)}>
              <l.icon size={17} className="flex-none text-[var(--accent)]" />
              <span className="about-link-text">
                <span className="text-[13px] font-medium text-[var(--text)]">{l.label}</span>
                <span className="mono text-[11px] text-[var(--muted)]">{l.sub}</span>
              </span>
              <ExternalLink size={14} className="flex-none text-[var(--muted)]" />
            </button>
          ))}
        </div>

        <div className="about-foot">© 2026 TheHolyOneZ · GPL-3.0</div>
      </div>
    </div>
  );
}
