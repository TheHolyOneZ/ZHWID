use serde::Deserialize;
use std::path::Path;
use std::process::Command;

use crate::dhcp;
use crate::identity;
use crate::mac;
use crate::state::{IfaceBackup, SpoofState};

#[derive(Deserialize, Debug)]
pub struct IfaceSpoof {
    pub name: String,
    pub mac: String,
}

#[derive(Deserialize, Debug)]
pub struct SpoofConfig {
    pub interfaces: Vec<IfaceSpoof>,
    pub set_machine_id: bool,
    pub machine_id: String,
    pub set_hostname: bool,
    pub hostname: String,

    #[serde(default)]
    pub pre_connect: bool,

    #[serde(default)]
    pub spoof_dhcp: bool,
}

fn valid_machine_id(id: &str) -> bool {
    id.len() == 32 && id.bytes().all(|b| b.is_ascii_hexdigit())
}

fn valid_hostname(h: &str) -> bool {
    !h.is_empty()
        && h.len() <= 63
        && h.bytes()
            .next()
            .map(|b| b.is_ascii_alphanumeric())
            .unwrap_or(false)
        && h.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

fn valid_iface_name(n: &str) -> bool {
    !n.is_empty()
        && n.len() <= 15
        && n.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

pub fn capture_backup(config: &SpoofConfig) -> SpoofState {
    let ifaces = identity::list_interfaces();
    let want_dhcp = config.spoof_dhcp && dhcp::nm_available();
    let mut backup = Vec::new();
    for req in &config.interfaces {
        let info = ifaces.iter().find(|i| i.name == req.name);
        let (perm, cur) = match info {
            Some(i) => (i.perm_mac.clone(), i.current_mac.clone()),
            None => (String::new(), String::new()),
        };

        let (nm_connection, dhcp_client_id, dhcp_duid) = if want_dhcp {
            match dhcp::connection_for(&req.name).filter(|c| dhcp::safe_conn(c)) {
                Some(conn) => {
                    let (cid, duid) = dhcp::read_dhcp(&conn);
                    (Some(conn), Some(cid), Some(duid))
                }
                None => (None, None, None),
            }
        } else {
            (None, None, None)
        };

        backup.push(IfaceBackup {
            name: req.name.clone(),
            perm_mac: if perm.is_empty() { cur.clone() } else { perm },
            orig_mac: cur,
            nm_connection,
            dhcp_client_id,
            dhcp_duid,
        });
    }
    SpoofState {
        active: false,
        captured_at: None,
        interfaces: backup,
        machine_id: Some(identity::read_machine_id()),
        hostname: Some(identity::read_hostname()),
    }
}

pub fn build_apply_script(config: &SpoofConfig, backup: &SpoofState) -> Result<String, String> {
    let mut lines: Vec<String> = vec!["set -e".to_string()];

    for iface in &config.interfaces {
        if !valid_iface_name(&iface.name) {
            return Err(format!("invalid interface name: {}", iface.name));
        }
        if !mac::is_valid(&iface.mac) {
            return Err(format!("invalid MAC address: {}", iface.mac));
        }

        let conn = if config.spoof_dhcp {
            backup
                .interfaces
                .iter()
                .find(|b| b.name == iface.name)
                .and_then(|b| b.nm_connection.clone())
                .filter(|c| dhcp::safe_conn(c))
        } else {
            None
        };
        if let Some(conn) = &conn {
            lines.push(format!(
                "nmcli connection modify '{}' ipv4.dhcp-client-id '{}' ipv6.dhcp-duid '{}' || true",
                conn,
                dhcp::random_client_id(),
                dhcp::random_duid()
            ));
        }

        if config.pre_connect {
            lines.push(format!("nmcli device disconnect '{}' || true", iface.name));
        }
        lines.push(format!("ip link set dev '{}' down", iface.name));
        lines.push(format!(
            "ip link set dev '{}' address '{}'",
            iface.name, iface.mac
        ));
        if config.pre_connect {
            lines.push(format!(
                "echo 'left {} disconnected — connect when ready'",
                iface.name
            ));
        } else {
            lines.push(format!("ip link set dev '{}' up || true", iface.name));

            if let Some(conn) = &conn {
                lines.push(format!(
                    "nmcli connection up '{}' 2>/dev/null || true",
                    conn
                ));
            }
        }
    }

    if config.set_machine_id {
        if !valid_machine_id(&config.machine_id) {
            return Err("invalid machine-id (need 32 hex chars)".to_string());
        }
        lines.push(machine_id_write(&config.machine_id));
    }

    if config.set_hostname {
        if !valid_hostname(&config.hostname) {
            return Err(format!("invalid hostname: {}", config.hostname));
        }
        lines.push(format!("hostnamectl hostname '{}'", config.hostname));
    }

    Ok(lines.join("\n"))
}

pub fn build_restore_script(state: &SpoofState) -> Result<String, String> {
    let mut lines: Vec<String> = vec!["set +e".to_string()];

    for iface in &state.interfaces {
        if let Some(conn) = iface.nm_connection.as_ref().filter(|c| dhcp::safe_conn(c)) {
            if let (Some(cid), Some(duid)) = (&iface.dhcp_client_id, &iface.dhcp_duid) {
                if !cid.contains('\'') && !duid.contains('\'') {
                    lines.push(format!(
                        "nmcli connection modify '{}' ipv4.dhcp-client-id '{}' ipv6.dhcp-duid '{}' || true",
                        conn, cid, duid
                    ));
                }
            }
        }
        if iface.perm_mac.is_empty() || !mac::is_valid(&iface.perm_mac) {
            continue;
        }
        lines.push(format!("ip link set dev '{}' down", iface.name));
        lines.push(format!(
            "ip link set dev '{}' address '{}'",
            iface.name, iface.perm_mac
        ));
        lines.push(format!("ip link set dev '{}' up || true", iface.name));
        if let Some(conn) = iface.nm_connection.as_ref().filter(|c| dhcp::safe_conn(c)) {
            lines.push(format!(
                "nmcli connection up '{}' 2>/dev/null || true",
                conn
            ));
        }
    }

    if let Some(id) = &state.machine_id {
        if valid_machine_id(id) {
            lines.push(machine_id_write(id));
        }
    }

    if let Some(host) = &state.hostname {
        if valid_hostname(host) {
            lines.push(format!("hostnamectl hostname '{}'", host));
        }
    }

    Ok(lines.join("\n"))
}

fn machine_id_write(id: &str) -> String {
    format!(
        "printf '%s\\n' '{id}' > /etc/machine-id\n\
         if [ -e /var/lib/dbus/machine-id ] && [ ! -L /var/lib/dbus/machine-id ]; then \
         printf '%s\\n' '{id}' > /var/lib/dbus/machine-id; fi"
    )
}

fn tool_exists(name: &str) -> bool {
    Path::new(&format!("/usr/bin/{name}")).exists() || Path::new(&format!("/bin/{name}")).exists()
}

fn bash_path() -> &'static str {
    if Path::new("/usr/bin/bash").exists() {
        "/usr/bin/bash"
    } else {
        "/bin/bash"
    }
}

pub fn run_privileged(script: &str) -> Result<String, String> {
    let has_gui = std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok();
    if has_gui && tool_exists("pkexec") {
        run_pkexec(script)
    } else if tool_exists("sudo") {
        run_sudo(script)
    } else if tool_exists("pkexec") {
        run_pkexec(script)
    } else {
        Err("no privilege escalation tool found (need pkexec or sudo)".to_string())
    }
}

pub fn run_pkexec(script: &str) -> Result<String, String> {
    let output = Command::new("pkexec")
        .arg(bash_path())
        .arg("-c")
        .arg(script)
        .output()
        .map_err(|e| format!("failed to launch pkexec: {e}"))?;
    finish("pkexec", output)
}

pub fn run_sudo(script: &str) -> Result<String, String> {
    let status = Command::new("sudo")
        .arg(bash_path())
        .arg("-c")
        .arg(script)
        .status()
        .map_err(|e| format!("failed to launch sudo: {e}"))?;
    if status.success() {
        Ok(String::new())
    } else {
        Err(format!(
            "privileged action failed (sudo exit {})",
            status.code().unwrap_or(-1)
        ))
    }
}

fn finish(tool: &str, output: std::process::Output) -> Result<String, String> {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if output.status.success() {
        Ok(format!("{stdout}{stderr}").trim().to_string())
    } else {
        let code = output.status.code().unwrap_or(-1);
        let hint = match (tool, code) {
            ("pkexec", 126) => " — authorization dialog was dismissed",
            ("pkexec", 127) => " — authentication failed",
            _ => "",
        };
        let detail = if stderr.trim().is_empty() {
            stdout.trim()
        } else {
            stderr.trim()
        };
        Err(format!("privileged action failed{hint}: {detail}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::IfaceBackup;

    fn iface(name: &str, mac: &str) -> IfaceSpoof {
        IfaceSpoof {
            name: name.into(),
            mac: mac.into(),
        }
    }

    fn cfg(interfaces: Vec<IfaceSpoof>) -> SpoofConfig {
        SpoofConfig {
            interfaces,
            set_machine_id: true,
            machine_id: "0123456789abcdef0123456789abcdef".into(),
            set_hostname: true,
            hostname: "test-host".into(),
            pre_connect: false,
            spoof_dhcp: false,
        }
    }

    #[test]
    fn apply_script_has_expected_commands() {
        let c = cfg(vec![iface("wlan0", "aa:bb:cc:dd:ee:ff")]);
        let s = build_apply_script(&c, &SpoofState::default()).unwrap();
        assert!(s.contains("ip link set dev 'wlan0' address 'aa:bb:cc:dd:ee:ff'"));
        assert!(s.contains("ip link set dev 'wlan0' up"));
        assert!(s.contains("> /etc/machine-id"));
        assert!(s.contains("hostnamectl hostname 'test-host'"));
    }

    #[test]
    fn pre_connect_disconnects_and_leaves_down() {
        let mut c = cfg(vec![iface("wlan0", "aa:bb:cc:dd:ee:ff")]);
        c.pre_connect = true;
        let s = build_apply_script(&c, &SpoofState::default()).unwrap();
        assert!(s.contains("nmcli device disconnect 'wlan0'"));
        assert!(!s.contains("ip link set dev 'wlan0' up"));
    }

    #[test]
    fn invalid_values_are_rejected() {
        let bad_mac = cfg(vec![iface("wlan0", "zz:zz:zz:zz:zz:zz")]);
        assert!(build_apply_script(&bad_mac, &SpoofState::default()).is_err());

        let mut bad_host = cfg(vec![iface("wlan0", "aa:bb:cc:dd:ee:ff")]);
        bad_host.hostname = "bad host".into();
        assert!(build_apply_script(&bad_host, &SpoofState::default()).is_err());

        let mut bad_mid = cfg(vec![iface("wlan0", "aa:bb:cc:dd:ee:ff")]);
        bad_mid.machine_id = "nothex".into();
        assert!(build_apply_script(&bad_mid, &SpoofState::default()).is_err());

        let bad_iface = cfg(vec![iface("wlan0; reboot", "aa:bb:cc:dd:ee:ff")]);
        assert!(build_apply_script(&bad_iface, &SpoofState::default()).is_err());
    }

    #[test]
    fn shell_injection_in_hostname_is_blocked() {
        let mut c = cfg(vec![iface("wlan0", "aa:bb:cc:dd:ee:ff")]);
        c.hostname = "x'; reboot; '".into();
        assert!(build_apply_script(&c, &SpoofState::default()).is_err());
    }

    #[test]
    fn dhcp_modify_included_only_for_safe_connection_names() {
        let mut c = cfg(vec![iface("wlan0", "aa:bb:cc:dd:ee:ff")]);
        c.spoof_dhcp = true;
        let mut backup = SpoofState {
            interfaces: vec![IfaceBackup {
                name: "wlan0".into(),
                perm_mac: "aa:bb:cc:dd:ee:ff".into(),
                orig_mac: "aa:bb:cc:dd:ee:ff".into(),
                nm_connection: Some("Home WiFi".into()),
                dhcp_client_id: Some(String::new()),
                dhcp_duid: Some(String::new()),
            }],
            ..Default::default()
        };
        let s = build_apply_script(&c, &backup).unwrap();
        assert!(s.contains("nmcli connection modify 'Home WiFi'"));

        backup.interfaces[0].nm_connection = Some("evil'; reboot; '".into());
        let s2 = build_apply_script(&c, &backup).unwrap();
        assert!(!s2.contains("reboot"));
    }

    #[test]
    fn restore_script_targets_permanent_mac_and_originals() {
        let st = SpoofState {
            interfaces: vec![IfaceBackup {
                name: "wlan0".into(),
                perm_mac: "10:20:30:40:50:60".into(),
                orig_mac: "10:20:30:40:50:60".into(),
                nm_connection: None,
                dhcp_client_id: None,
                dhcp_duid: None,
            }],
            machine_id: Some("0123456789abcdef0123456789abcdef".into()),
            hostname: Some("my-host".into()),
            ..Default::default()
        };
        let s = build_restore_script(&st).unwrap();
        assert!(s.contains("ip link set dev 'wlan0' address '10:20:30:40:50:60'"));
        assert!(s.contains("hostnamectl hostname 'my-host'"));
        assert!(s.contains("> /etc/machine-id"));
    }
}
