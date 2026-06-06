use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    Icon, TrayIconBuilder,
};
use crossbeam_channel::Sender;
use anyhow::Result;
use crate::{AppEvent, AppState};

pub fn run(state: AppState, event_tx: Sender<AppEvent>) -> Result<()> {
    let tray_menu = Menu::new();
    let item_update    = MenuItem::new("🔄 Mise à jour disponible !", false, None); // caché au départ
    let item_sep_up    = PredefinedMenuItem::separator();
    let item_settings  = MenuItem::new("⚙  Paramètres", true, None);
    let item_history   = MenuItem::new("📋 Historique", true, None);
    let item_devices   = MenuItem::new("🎙  Microphones", true, None);
    let item_sep       = PredefinedMenuItem::separator();
    let item_quit      = MenuItem::new("✕  Quitter", true, None);

    tray_menu.append_items(&[
        &item_update,
        &item_sep_up,
        &item_settings,
        &item_history,
        &item_devices,
        &item_sep,
        &item_quit,
    ])?;

    let icon = make_icon(false);

    let _tray = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Dictum — Dictée vocale")
        .with_icon(icon)
        .build()?;

    let menu_rx = MenuEvent::receiver();
    let item_update = MenuItem::new("🔄 Mise à jour disponible !", true, None);
    let mut update_info: Option<crate::updater::UpdateInfo> = None;

    // Windows message pump — keeps the tray alive and processes events
    loop {
        pump_messages();

        // Vérifier si une mise à jour a été détectée en arrière-plan
        if update_info.is_none() {
            if let Some(info) = crate::take_update() {
                item_update.set_enabled(true);
                item_update.set_text(format!("🔄 Mise à jour v{} disponible !", info.version));
                update_info = Some(info);
            }
        }

        if let Ok(event) = menu_rx.try_recv() {
            if event.id == item_quit.id() {
                let _ = event_tx.send(AppEvent::Quit);
                break;
            } else if event.id == item_update.id() {
                if let Some(ref info) = update_info {
                    if let Err(e) = crate::updater::apply_update(info) {
                        log::error!("Mise à jour échouée : {e}");
                        show_dialog("Dictum — Erreur", &format!("Mise à jour échouée :\n{e}"));
                    }
                }
            } else if event.id == item_settings.id() {
                if let Err(e) = crate::config::Config::open_in_editor() {
                    log::error!("Failed to open settings: {e}");
                }
            } else if event.id == item_history.id() {
                let msg = state.history.lock().unwrap().as_display_string();
                show_dialog("Dictum — Historique", &msg);
            } else if event.id == item_devices.id() {
                let devices = crate::audio::list_devices();
                let msg = if devices.is_empty() {
                    "Aucun microphone détecté.".to_string()
                } else {
                    devices.join("\n")
                };
                show_dialog("Dictum — Microphones disponibles", &msg);
            }
        }

        // Update tooltip + icône selon état
        let recording = *state.is_recording.lock().unwrap();
        let transcribing = *state.is_transcribing.lock().unwrap();
        let tooltip = if recording {
            "Dictum — Enregistrement..."
        } else if transcribing {
            "Dictum — Transcription..."
        } else {
            "Dictum — Dictée vocale"
        };
        let _ = _tray.set_tooltip(Some(tooltip));
        let _ = _tray.set_icon(Some(make_icon(recording)));

        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    Ok(())
}

fn pump_messages() {
    #[cfg(windows)]
    unsafe {
        use std::mem::zeroed;
        use winapi::um::winuser::{
            DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE,
        };
        let mut msg: MSG = zeroed();
        while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

fn show_dialog(title: &str, message: &str) {
    #[cfg(windows)]
    unsafe {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use winapi::um::winuser::{MessageBoxW, MB_ICONINFORMATION, MB_OK};

        let wide_title: Vec<u16> = OsStr::new(title)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let wide_msg: Vec<u16> = OsStr::new(message)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        MessageBoxW(
            std::ptr::null_mut(),
            wide_msg.as_ptr(),
            wide_title.as_ptr(),
            MB_OK | MB_ICONINFORMATION,
        );
    }

    #[cfg(not(windows))]
    log::info!("[{}] {}", title, message);
}

/// Generate a simple colored circle icon (32x32 RGBA).
fn make_icon(recording: bool) -> Icon {
    let size = 32u32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - 16.0;
            let dy = y as f32 - 16.0;
            let dist = (dx * dx + dy * dy).sqrt();
            let idx = ((y * size + x) * 4) as usize;
            if dist < 13.0 {
                if recording {
                    // Red when recording
                    rgba[idx]     = 220;
                    rgba[idx + 1] = 50;
                    rgba[idx + 2] = 50;
                } else {
                    // Steel blue at rest
                    rgba[idx]     = 70;
                    rgba[idx + 1] = 130;
                    rgba[idx + 2] = 180;
                }
                rgba[idx + 3] = 255;
            } else if dist < 15.0 {
                // White border
                rgba[idx]     = 255;
                rgba[idx + 1] = 255;
                rgba[idx + 2] = 255;
                rgba[idx + 3] = 180;
            }
        }
    }

    Icon::from_rgba(rgba, size, size).expect("Failed to create tray icon")
}
