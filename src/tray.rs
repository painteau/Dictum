use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    Icon, TrayIconBuilder,
};
use arboard;
use crossbeam_channel::Sender;
use anyhow::Result;
use crate::{AppEvent, AppState};

pub fn run(state: AppState, event_tx: Sender<AppEvent>) -> Result<()> {
    let tray_menu = Menu::new();
    let item_update    = MenuItem::new("🔄 Mise à jour disponible !", false, None);
    let item_sep_up    = PredefinedMenuItem::separator();
    let item_settings  = MenuItem::new("⚙  Paramètres", true, None);
    let item_history   = MenuItem::new("📋 Historique", true, None);
    let item_devices   = MenuItem::new("🎙  Microphones", true, None);
    let item_copy_last  = MenuItem::new("📋 Copier dernière dictée", true, None);
    let item_reset_cfg  = MenuItem::new("🔧 Réinitialiser la config", true, None);
    let item_clear_hist = MenuItem::new("🗑  Effacer l'historique", true, None);
    let item_reload     = MenuItem::new("↺  Recharger la config", true, None);
    let item_open_log   = MenuItem::new("📄 Ouvrir le log", true, None);
    let item_about      = MenuItem::new(
        format!("ℹ  Dictum v{}", env!("CARGO_PKG_VERSION")),
        true, None
    );
    let item_sep        = PredefinedMenuItem::separator();
    let item_quit       = MenuItem::new("✕  Quitter", true, None);

    tray_menu.append_items(&[
        &item_update,         // caché au départ, visible si update
        &item_sep_up,         // séparateur update
        // Config
        &item_settings,
        &item_reload,
        &item_reset_cfg,
        &PredefinedMenuItem::separator(),
        // Historique
        &item_history,
        &item_copy_last,
        &item_clear_hist,
        &PredefinedMenuItem::separator(),
        // Système
        &item_devices,
        &item_open_log,
        &item_about,
        &item_sep,
        &item_quit,
    ])?;

    let icon = make_icon(false, false);

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
            } else if event.id == item_copy_last.id() {
                let last = state.history.lock().unwrap()
                    .entries().front().map(|e| e.text.clone());
                match last {
                    Some(text) => {
                        match arboard::Clipboard::new().and_then(|mut c| c.set_text(&text)) {
                            Ok(_) => log::info!("Copié dans le presse-papiers"),
                            Err(e) => show_dialog("Dictum", &format!("Erreur clipboard : {e}")),
                        }
                    }
                    None => show_dialog("Dictum", "Aucune dictée à copier."),
                }
            } else if event.id == item_reset_cfg.id() {
                let default_cfg = crate::config::Config::default();
                if let Err(e) = default_cfg.save() {
                    log::error!("Reset config échoué : {e}");
                } else {
                    *state.config.lock().unwrap() = default_cfg;
                    show_dialog("Dictum", "Config réinitialisée aux valeurs par défaut.\nRedémarrer pour appliquer le hotkey.");
                }
            } else if event.id == item_clear_hist.id() {
                state.history.lock().unwrap().clear();
                let _ = state.history.lock().unwrap().save();
                show_dialog("Dictum", "Historique effacé.");
            } else if event.id == item_reload.id() {
                match crate::config::Config::load() {
                    Ok(new_cfg) => {
                        *state.config.lock().unwrap() = new_cfg;
                        log::info!("Config rechargée");
                    }
                    Err(e) => log::error!("Erreur reload config : {e}"),
                }
            } else if event.id == item_open_log.id() {
                let log_path = crate::config::Config::data_dir().join("dictum.log");
                if log_path.exists() {
                    std::process::Command::new("notepad").arg(&log_path).spawn().ok();
                } else {
                    show_dialog("Dictum", "Aucun fichier de log trouvé.");
                }
            } else if event.id == item_about.id() {
                let config = state.config.lock().unwrap();
                let count = *state.session_count.lock().unwrap();
                let log_path = crate::config::Config::data_dir().join("dictum.log");
                let msg = format!(
                    "Dictum v{}\ngithub.com/painteau/Dictum\n\nModèle : {}\nLangue  : {}\nHotkey  : {}{}{}{}\n\nSession : {} transcription{}\nLog     : {}",
                    env!("CARGO_PKG_VERSION"),
                    config.model_path.file_name().and_then(|n| n.to_str()).unwrap_or("?"),
                    config.language,
                    if config.hotkey.ctrl  { "Ctrl+" } else { "" },
                    if config.hotkey.alt   { "Alt+"  } else { "" },
                    if config.hotkey.shift { "Shift+"} else { "" },
                    config.hotkey.key,
                    count,
                    if count > 1 { "s" } else { "" },
                    log_path.display()
                );
                show_dialog("À propos de Dictum", &msg);
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
        let count = *state.session_count.lock().unwrap();
        let tooltip = if recording {
            "Dictum — Enregistrement...".to_string()
        } else if transcribing {
            "Dictum — Transcription...".to_string()
        } else if count > 0 {
            format!("Dictum — {} dictée{} cette session", count, if count > 1 { "s" } else { "" })
        } else {
            "Dictum — Dictée vocale".to_string()
        };
        let _ = _tray.set_tooltip(Some(&tooltip));
        let _ = _tray.set_icon(Some(make_icon(recording, transcribing)));

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

/// Génère une icône 32x32 :
/// rouge = enregistrement, orange = transcription, bleu = repos
fn make_icon(recording: bool, transcribing: bool) -> Icon {
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
                    rgba[idx] = 220; rgba[idx+1] = 50;  rgba[idx+2] = 50;  // rouge
                } else if transcribing {
                    rgba[idx] = 220; rgba[idx+1] = 140; rgba[idx+2] = 20;  // orange
                } else {
                    rgba[idx] = 70;  rgba[idx+1] = 130; rgba[idx+2] = 180; // bleu
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
