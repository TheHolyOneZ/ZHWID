use crate::privileged::{self, SpoofConfig};
use crate::state::{self, SpoofState};

pub fn apply(config: &SpoofConfig) -> Result<String, String> {
    let backup = privileged::capture_backup(config);
    state::save(&backup)?;

    let script = privileged::build_apply_script(config, &backup)?;
    let out = privileged::run_privileged(&script)?;

    let mut applied = backup;
    applied.active = true;
    applied.captured_at = Some(SpoofState::now());
    state::save(&applied)?;
    Ok(out)
}

pub fn restore() -> Result<String, String> {
    let st = state::load();
    if st.interfaces.is_empty() && st.machine_id.is_none() && st.hostname.is_none() {
        return Err("No backup found — nothing to restore.".to_string());
    }
    let script = privileged::build_restore_script(&st)?;
    let out = privileged::run_privileged(&script)?;

    let mut cleared = st;
    cleared.active = false;
    cleared.captured_at = None;
    state::save(&cleared)?;
    Ok(out)
}
