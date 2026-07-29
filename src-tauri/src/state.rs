use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct IfaceBackup {
    pub name: String,

    pub perm_mac: String,

    pub orig_mac: String,

    #[serde(default)]
    pub nm_connection: Option<String>,

    #[serde(default)]
    pub dhcp_client_id: Option<String>,

    #[serde(default)]
    pub dhcp_duid: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SpoofState {
    pub active: bool,

    pub captured_at: Option<u64>,
    pub interfaces: Vec<IfaceBackup>,
    pub machine_id: Option<String>,
    pub hostname: Option<String>,
}

impl SpoofState {
    pub fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

pub fn state_dir() -> PathBuf {
    let mut dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    dir.push("z-hwid");
    dir
}

fn state_path() -> PathBuf {
    state_dir().join("state.json")
}

pub fn load() -> SpoofState {
    match fs::read_to_string(state_path()) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => SpoofState::default(),
    }
}

pub fn save(state: &SpoofState) -> Result<(), String> {
    let dir = state_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("create state dir: {e}"))?;
    let json = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    fs::write(state_path(), json).map_err(|e| format!("write state: {e}"))
}

pub fn is_active() -> bool {
    load().active
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_roundtrips_through_json() {
        let s = SpoofState {
            active: true,
            captured_at: Some(1_700_000_000),
            interfaces: vec![IfaceBackup {
                name: "wlan0".into(),
                perm_mac: "aa:bb:cc:dd:ee:ff".into(),
                orig_mac: "aa:bb:cc:dd:ee:ff".into(),
                nm_connection: Some("MyWifi".into()),
                dhcp_client_id: Some("".into()),
                dhcp_duid: Some("".into()),
            }],
            machine_id: Some("0123456789abcdef0123456789abcdef".into()),
            hostname: Some("my-host".into()),
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: SpoofState = serde_json::from_str(&json).unwrap();
        assert_eq!(back.active, s.active);
        assert_eq!(back.interfaces.len(), 1);
        assert_eq!(back.interfaces[0].name, "wlan0");
        assert_eq!(back.machine_id, s.machine_id);
        assert_eq!(back.hostname, s.hostname);
    }
}
