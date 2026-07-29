use std::path::Path;
use std::process::Command;

use crate::profile::HardwareProfile;

const ZCLOAK_LIB: &str = env!("ZCLOAK_LIB");

pub fn lib_path() -> Option<String> {
    if Path::new(ZCLOAK_LIB).exists() {
        Some(ZCLOAK_LIB.to_string())
    } else {
        None
    }
}

pub fn launch(app: &str, p: &HardwareProfile) -> Result<String, String> {
    if app.trim().is_empty() {
        return Err("no application specified".to_string());
    }
    let lib = lib_path().ok_or("cloak library not built (libzcloak.so missing)")?;

    Command::new(app)
        .env("LD_PRELOAD", &lib)
        .env("ZCLOAK_MACHINE_ID", &p.machine_id)
        .env("ZCLOAK_DISK_SERIAL", &p.disk_serial)
        .env("ZCLOAK_DMI_SYS_VENDOR", &p.sys_vendor)
        .env("ZCLOAK_DMI_PRODUCT_NAME", &p.product_name)
        .env("ZCLOAK_DMI_PRODUCT_VERSION", &p.product_version)
        .env("ZCLOAK_DMI_PRODUCT_UUID", &p.product_uuid)
        .env("ZCLOAK_DMI_PRODUCT_SERIAL", &p.product_serial)
        .env("ZCLOAK_DMI_BOARD_VENDOR", &p.board_vendor)
        .env("ZCLOAK_DMI_BOARD_NAME", &p.board_name)
        .env("ZCLOAK_DMI_BOARD_SERIAL", &p.board_serial)
        .env("ZCLOAK_DMI_BIOS_VENDOR", &p.bios_vendor)
        .env("ZCLOAK_DMI_BIOS_VERSION", &p.bios_version)
        .env("ZCLOAK_DMI_CHASSIS_SERIAL", &p.chassis_serial)
        .spawn()
        .map_err(|e| format!("failed to launch '{app}': {e}"))?;

    Ok(format!(
        "Launched '{app}' cloaked as {} {} (uuid {})",
        p.sys_vendor, p.product_name, p.product_uuid
    ))
}
