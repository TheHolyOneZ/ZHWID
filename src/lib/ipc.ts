import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface InterfaceInfo {
  name: string;
  kind: "wifi" | "ethernet" | "other";
  current_mac: string;
  perm_mac: string;
  is_up: boolean;
  spoofed: boolean;
}

export interface IdentityStatus {
  active: boolean;
  interfaces: InterfaceInfo[];
  machine_id: string;
  hostname: string;
  product_uuid: string;
  board_serial: string;
}

export interface SpoofConfigIface {
  name: string;
  mac: string;
}

export interface SpoofConfig {
  interfaces: SpoofConfigIface[];
  set_machine_id: boolean;
  machine_id: string;
  set_hostname: boolean;
  hostname: string;
  pre_connect: boolean;
  spoof_dhcp: boolean;
}

export interface HardwareProfile {
  sys_vendor: string;
  product_name: string;
  product_version: string;
  product_uuid: string;
  product_serial: string;
  board_vendor: string;
  board_name: string;
  board_serial: string;
  bios_vendor: string;
  bios_version: string;
  chassis_serial: string;
  machine_id: string;
  disk_serial: string;
}

export interface LogEntry {
  level: "info" | "success" | "warn" | "error";
  message: string;
  ts: number;
}

export const getStatus = () => invoke<IdentityStatus>("get_status");
export const listInterfaces = () => invoke<InterfaceInfo[]>("list_interfaces");
export const macProfiles = () => invoke<[string, string][]>("mac_profiles");
export const generateMac = (profile: string) =>
  invoke<string>("generate_mac", { profile });
export const generateMachineId = () => invoke<string>("generate_machine_id");
export const generateHostname = () => invoke<string>("generate_hostname");
export const suggestHardwareProfile = () =>
  invoke<HardwareProfile>("suggest_hardware_profile");
export const cloakAvailable = () => invoke<boolean>("cloak_available");
export const nmAvailable = () => invoke<boolean>("nm_available");
export const hotkeyLabel = () => invoke<string>("hotkey_label");
export const setAutoSpoof = (enabled: boolean) =>
  invoke<void>("set_auto_spoof", { enabled });
export const applySpoof = (config: SpoofConfig) =>
  invoke<IdentityStatus>("apply_spoof", { config });
export const restoreAll = () => invoke<IdentityStatus>("restore_all");
export const launchCloaked = (appPath: string, profile: HardwareProfile) =>
  invoke<string>("launch_cloaked", { appPath, profile });

export const onLog = (cb: (e: LogEntry) => void): Promise<UnlistenFn> =>
  listen<LogEntry>("zhwid://log", (event) => cb(event.payload));
