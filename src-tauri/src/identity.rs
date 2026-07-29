use serde::Serialize;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Serialize, Clone, Debug)]
pub struct InterfaceInfo {
    pub name: String,

    pub kind: String,
    pub current_mac: String,

    pub perm_mac: String,
    pub is_up: bool,

    pub spoofed: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct IdentityStatus {
    pub active: bool,
    pub interfaces: Vec<InterfaceInfo>,
    pub machine_id: String,
    pub hostname: String,

    pub product_uuid: String,
    pub board_serial: String,
}

const SYS_NET: &str = "/sys/class/net";

pub fn list_interfaces() -> Vec<InterfaceInfo> {
    let mut out = Vec::new();
    let entries = match fs::read_dir(SYS_NET) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "lo" {
            continue;
        }
        let base = format!("{SYS_NET}/{name}");
        let is_wireless = Path::new(&format!("{base}/wireless")).exists()
            || Path::new(&format!("{base}/phy80211")).exists();
        let has_device = Path::new(&format!("{base}/device")).exists();
        if !is_wireless && !has_device {
            continue;
        }
        let kind = if is_wireless { "wifi" } else { "ethernet" };

        let current_mac = read_trim(&format!("{base}/address")).unwrap_or_default();
        let operstate = read_trim(&format!("{base}/operstate")).unwrap_or_default();
        let is_up = operstate == "up";
        let perm_mac = permanent_mac(&name).unwrap_or_else(|| current_mac.clone());
        let spoofed = !perm_mac.is_empty()
            && !current_mac.is_empty()
            && !perm_mac.eq_ignore_ascii_case(&current_mac);

        out.push(InterfaceInfo {
            name,
            kind: kind.to_string(),
            current_mac,
            perm_mac,
            is_up,
            spoofed,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

pub fn permanent_mac(iface: &str) -> Option<String> {
    let output = Command::new("ethtool").arg("-P").arg(iface).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);

    let addr = text
        .split_whitespace()
        .last()
        .map(|s| s.trim().to_lowercase())?;
    if addr == "00:00:00:00:00:00" || addr.len() != 17 {
        return None;
    }
    Some(addr)
}

pub fn read_machine_id() -> String {
    read_trim("/etc/machine-id").unwrap_or_else(|| "—".to_string())
}

pub fn read_hostname() -> String {
    read_trim("/proc/sys/kernel/hostname")
        .or_else(|| read_trim("/etc/hostname"))
        .unwrap_or_else(|| "—".to_string())
}

fn read_dmi(field: &str) -> String {
    read_trim(&format!("/sys/class/dmi/id/{field}"))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "—".to_string())
}

pub fn full_status(active: bool) -> IdentityStatus {
    IdentityStatus {
        active,
        interfaces: list_interfaces(),
        machine_id: read_machine_id(),
        hostname: read_hostname(),
        product_uuid: read_dmi("product_uuid"),
        board_serial: read_dmi("board_serial"),
    }
}

fn read_trim(path: &str) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}
