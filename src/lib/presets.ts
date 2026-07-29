

const hex = (n: number) =>
  Array.from({ length: n }, () => Math.floor(Math.random() * 16).toString(16)).join("");

const b36 = (n: number) =>
  Array.from({ length: n }, () => Math.floor(Math.random() * 36).toString(36))
    .join("")
    .toUpperCase();

export interface Preset {
  id: string;
  label: string;
  description: string;
  macProfile: string;

  hostname: () => string;
  machineId: boolean;
  hostnameOn: boolean;
  dhcp: boolean;
}

export const PRESETS: Preset[] = [
  {
    id: "balanced",
    label: "Balanced",
    description: "Everyday privacy — random MAC, fresh machine-id, hostname & DHCP identity.",
    macProfile: "random",
    hostname: () => `host-${hex(4)}`,
    machineId: true,
    hostnameOn: true,
    dhcp: true,
  },
  {
    id: "printer",
    label: "Pentest · Printer",
    description: "Blend in as an HP network printer on a scan (HP OUI + JetDirect-style name).",
    macProfile: "hp_printer",
    hostname: () => `NPI${hex(6).toUpperCase()}`,
    machineId: true,
    hostnameOn: true,
    dhcp: true,
  },
  {
    id: "workstation",
    label: "Pentest · Workstation",
    description: "Look like a Windows workstation (Dell OUI + DESKTOP-xxxx name).",
    macProfile: "dell",
    hostname: () => `DESKTOP-${b36(7)}`,
    machineId: true,
    hostnameOn: true,
    dhcp: true,
  },
  {
    id: "maconly",
    label: "MAC only",
    description: "Minimal footprint — spoof just the network MAC (and DHCP), leave the rest.",
    macProfile: "random",
    hostname: () => "",
    machineId: false,
    hostnameOn: false,
    dhcp: true,
  },
];

export const presetById = (id: string) => PRESETS.find((p) => p.id === id);
