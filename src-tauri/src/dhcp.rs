use rand::Rng;
use std::process::Command;

pub fn nmcli() -> Command {
    let mut cmd = Command::new("nmcli");
    cmd.env("LC_ALL", "C").env("LANG", "C");
    cmd
}

pub fn nm_available() -> bool {
    nmcli()
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn connection_for(iface: &str) -> Option<String> {
    let out = nmcli()
        .args([
            "-t",
            "-e",
            "no",
            "-g",
            "GENERAL.CONNECTION",
            "device",
            "show",
            iface,
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if name.is_empty() || name == "--" {
        None
    } else {
        Some(name)
    }
}

pub fn read_dhcp(conn: &str) -> (String, String) {
    let read = |field: &str| -> String {
        nmcli()
            .args(["-t", "-e", "no", "-g", field, "connection", "show", conn])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default()
    };
    (read("ipv4.dhcp-client-id"), read("ipv6.dhcp-duid"))
}

pub fn random_client_id() -> String {
    let mut rng = rand::thread_rng();
    (0..8)
        .map(|_| format!("{:02x}", rng.gen::<u8>()))
        .collect::<Vec<_>>()
        .join(":")
}

pub fn random_duid() -> String {
    let mut rng = rand::thread_rng();
    let mut parts = vec!["00".to_string(), "04".to_string()];
    for _ in 0..16 {
        parts.push(format!("{:02x}", rng.gen::<u8>()));
    }
    parts.join(":")
}

pub fn safe_conn(name: &str) -> bool {
    !name.is_empty() && !name.contains('\'') && name.len() <= 128
}
