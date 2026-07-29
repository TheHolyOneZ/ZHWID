pub mod cloak;
pub mod dhcp;
pub mod engine;
pub mod generate;
pub mod identity;
pub mod mac;
pub mod presets;
pub mod privileged;
pub mod profile;
pub mod state;

use std::io::BufRead;
use std::sync::atomic::{AtomicBool, Ordering};

use identity::{IdentityStatus, InterfaceInfo};
use privileged::SpoofConfig;
use profile::HardwareProfile;
use serde::Serialize;
use state::SpoofState;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

static AUTO_SPOOF: AtomicBool = AtomicBool::new(false);

const HOTKEY_MODS: Modifiers = Modifiers::CONTROL.union(Modifiers::ALT);
const HOTKEY_CODE: Code = Code::KeyS;
pub const HOTKEY_LABEL: &str = "Ctrl+Alt+S";

#[derive(Clone, Serialize)]
struct LogEntry {
    level: String,
    message: String,
    ts: u64,
}

fn emit_log(app: &tauri::AppHandle, level: &str, message: &str) {
    let _ = app.emit(
        "zhwid://log",
        LogEntry {
            level: level.to_string(),
            message: message.to_string(),
            ts: SpoofState::now(),
        },
    );
}

#[tauri::command]
fn get_status() -> IdentityStatus {
    identity::full_status(state::is_active())
}

#[tauri::command]
fn list_interfaces() -> Vec<InterfaceInfo> {
    identity::list_interfaces()
}

#[tauri::command]
fn mac_profiles() -> Vec<(String, String)> {
    mac::PROFILES
        .iter()
        .map(|(id, label)| (id.to_string(), label.to_string()))
        .collect()
}

#[tauri::command]
fn generate_mac(profile: String) -> String {
    mac::generate(&profile)
}

#[tauri::command]
fn generate_machine_id() -> String {
    generate::machine_id()
}

#[tauri::command]
fn generate_hostname() -> String {
    generate::hostname()
}

#[tauri::command]
fn suggest_hardware_profile() -> HardwareProfile {
    HardwareProfile::random()
}

#[tauri::command]
fn cloak_available() -> bool {
    cloak::lib_path().is_some()
}

#[tauri::command]
fn nm_available() -> bool {
    dhcp::nm_available()
}

#[tauri::command]
fn hotkey_label() -> String {
    HOTKEY_LABEL.to_string()
}

#[tauri::command]
fn set_auto_spoof(enabled: bool) {
    AUTO_SPOOF.store(enabled, Ordering::Relaxed);
}

#[tauri::command]
async fn apply_spoof(app: tauri::AppHandle, config: SpoofConfig) -> Result<IdentityStatus, String> {
    emit_log(&app, "info", "Capturing original identity…");

    let result = tauri::async_runtime::spawn_blocking(move || engine::apply(&config))
        .await
        .map_err(|e| e.to_string())?;

    match result {
        Ok(out) => {
            emit_log(
                &app,
                "success",
                "Identity spoof applied — you are now protected.",
            );
            if !out.is_empty() {
                emit_log(&app, "info", &out);
            }
        }
        Err(e) => {
            emit_log(&app, "error", &e);
            return Err(e);
        }
    }
    Ok(identity::full_status(state::is_active()))
}

#[tauri::command]
async fn restore_all(app: tauri::AppHandle) -> Result<IdentityStatus, String> {
    emit_log(&app, "info", "Restoring original identity…");

    let result = tauri::async_runtime::spawn_blocking(engine::restore)
        .await
        .map_err(|e| e.to_string())?;

    match result {
        Ok(out) => {
            emit_log(&app, "success", "Original identity restored.");
            if !out.is_empty() {
                emit_log(&app, "info", &out);
            }
        }
        Err(e) => {
            emit_log(&app, "error", &e);
            return Err(e);
        }
    }
    Ok(identity::full_status(state::is_active()))
}

#[tauri::command]
fn launch_cloaked(
    app: tauri::AppHandle,
    app_path: String,
    profile: HardwareProfile,
) -> Result<String, String> {
    match cloak::launch(&app_path, &profile) {
        Ok(msg) => {
            emit_log(&app, "success", &msg);
            Ok(msg)
        }
        Err(e) => {
            emit_log(&app, "error", &e);
            Err(e)
        }
    }
}

fn build_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show / hide Z-HWID", true, None::<&str>)?;
    let restore = MenuItem::with_id(app, "restore", "Restore identity", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &restore, &quit])?;

    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("Z-HWID — identity switch")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => toggle_window(app),
            "restore" => {
                show_window(app);
                let _ = app.emit("zhwid://tray-restore", ());
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_window(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

fn show_window(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.set_focus();
    }
}

fn toggle_window(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        if w.is_visible().unwrap_or(false) {
            let _ = w.hide();
        } else {
            let _ = w.show();
            let _ = w.set_focus();
        }
    }
}

fn start_network_monitor(app: tauri::AppHandle) {
    std::thread::spawn(move || loop {
        let child = dhcp::nmcli()
            .arg("monitor")
            .stdout(std::process::Stdio::piped())
            .spawn();
        match child {
            Ok(mut c) => {
                if let Some(out) = c.stdout.take() {
                    for line in std::io::BufReader::new(out).lines().map_while(Result::ok) {
                        if !AUTO_SPOOF.load(Ordering::Relaxed) {
                            continue;
                        }
                        let l = line.to_lowercase();
                        if l.contains("connecting") {
                            let _ = app.emit("zhwid://auto-trigger", line);
                        }
                    }
                }
                let _ = c.wait();
            }
            Err(_) => return,
        }
        std::thread::sleep(std::time::Duration::from_secs(3));
    });
}

fn register_hotkey(app: &tauri::AppHandle) {
    let shortcut = Shortcut::new(Some(HOTKEY_MODS), HOTKEY_CODE);
    let _ = app.global_shortcut().register(shortcut);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        show_window(app);
                        let _ = app.emit("zhwid://hotkey-toggle", ());
                    }
                })
                .build(),
        )
        .setup(|app| {
            let handle = app.handle();
            build_tray(handle)?;
            register_hotkey(handle);
            start_network_monitor(handle.clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_status,
            list_interfaces,
            mac_profiles,
            generate_mac,
            generate_machine_id,
            generate_hostname,
            suggest_hardware_profile,
            cloak_available,
            nm_available,
            hotkey_label,
            set_auto_spoof,
            apply_spoof,
            restore_all,
            launch_cloaked,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
