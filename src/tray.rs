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
    let item_history   = MenuItem::new("📋 Historique (0)", true, None);
    let item_devices   = MenuItem::new("🎙  Microphones", true, None);
    let item_copy_last   = MenuItem::new("📋 Copier dernière dictée", true, None);
    let item_export_hist = MenuItem::new("💾 Exporter historique", true, None);
    let item_reset_cfg  = MenuItem::new("🔧 Réinitialiser la config", true, None);
    let item_clear_hist = MenuItem::new("🗑  Effacer l'historique", true, None);
    let item_reload     = MenuItem::new("↺  Recharger la config", true, None);
    let item_open_log   = MenuItem::new("📄 Ouvrir le log", true, None);
    let item_open_dir   = MenuItem::new("📁 Ouvrir le dossier Dictum", true, None);
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
        &item_export_hist,
        &item_clear_hist,
        &PredefinedMenuItem::separator(),
        // Système
        &item_devices,
        &item_open_log,
        &item_open_dir,
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
                let size_mb = (info.installer_size as f64 / 1_048_576.0).ceil() as u64;
                log::info!("Mise à jour v{} disponible ({} MB) — affiché dans le tray", info.version, size_mb);
                item_update.set_enabled(true);
                item_update.set_text(format!("🔄 v{} disponible ({} MB) — Cliquer pour installer", info.version, size_mb));
                update_info = Some(info);
            }
        }

        if let Ok(ref event) = menu_rx.try_recv() {
            log::debug!("Menu tray : event id={:?}", event.id);
            if event.id == item_quit.id() {
                let _ = event_tx.send(AppEvent::Quit);
                break;
            } else if event.id == item_update.id() {
                if let Some(ref info) = update_info {
                    let size_mb = (info.installer_size as f64 / 1_048_576.0).ceil() as u64;
                    let confirmed = confirm_dialog(
                        "Dictum — Mise à jour",
                        &format!("Installer Dictum v{} ({} MB) ?\n\nDictum va se fermer et l'installateur va démarrer.", info.version, size_mb)
                    );
                    if confirmed {
                        if let Err(e) = crate::updater::apply_update(info) {
                            log::error!("Mise à jour échouée : {e}");
                            show_dialog("Dictum — Erreur", &format!("Mise à jour échouée :\n{e}"));
                        }
                    }
                }
            } else if event.id == item_settings.id() {
                if let Err(e) = crate::config::Config::open_in_editor() {
                    log::error!("Impossible d'ouvrir la config : {e}");
                    // Fallback : ouvrir le dossier Dictum dans l'explorateur
                    let dir = crate::config::Config::data_dir();
                    std::process::Command::new("explorer").arg(&dir).spawn().ok();
                }
            } else if event.id == item_history.id() {
                let hist = state.history.lock().unwrap();
                let msg = if hist.is_empty() {
                    "Aucun historique.".to_string()
                } else {
                    hist.as_display_string()
                };
                drop(hist);
                show_dialog("Dictum — Historique", &msg);
            } else if event.id == item_copy_last.id() {
                let last = state.history.lock().unwrap().last_text();
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
                match crate::config::Config::reset_to_default() {
                    Ok(cfg) => {
                        *state.config.lock().unwrap() = cfg;
                        show_dialog("Dictum", "Config réinitialisée aux valeurs par défaut.\nRedémarrer pour appliquer le hotkey.");
                    }
                    Err(e) => log::error!("Reset config échoué : {e}"),
                }
            } else if event.id == item_export_hist.id() {
                let export_path = crate::config::Config::history_export_path();
                match state.history.lock().unwrap().export_to_file(&export_path) {
                    Ok(_) => {
                        // Ouvrir le fichier automatiquement après export
                        std::process::Command::new("notepad").arg(&export_path).spawn().ok();
                    }
                    Err(e) => show_dialog("Dictum", &format!("Erreur export : {e}")),
                }
            } else if event.id == item_clear_hist.id() {
                state.history.lock().unwrap().clear();
                let _ = state.history.lock().unwrap().save();
                show_dialog("Dictum", "Historique effacé.");
            } else if event.id == item_reload.id() {
                match crate::config::Config::load() {
                    Ok(new_cfg) => {
                        *state.config.lock().unwrap() = new_cfg;
                        // Notifier aussi le pipeline thread
                        let _ = event_tx.send(crate::AppEvent::ReloadConfig);
                        log::info!("Config rechargée");
                    }
                    Err(e) => log::error!("Erreur reload config : {e}"),
                }
            } else if event.id == item_open_dir.id() {
                if let Err(e) = crate::config::Config::open_data_dir() {
                    log::error!("Impossible d'ouvrir le dossier : {e}");
                }
            } else if event.id == item_open_log.id() {
                let log_path = crate::config::Config::log_path();
                if log_path.exists() {
                    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "notepad".to_string());
                    std::process::Command::new(&editor).arg(&log_path).spawn()
                        .or_else(|_| std::process::Command::new("notepad").arg(&log_path).spawn())
                        .ok();
                } else {
                    show_dialog("Dictum", "Aucun fichier de log trouvé.");
                }
            } else if event.id == item_about.id() {
                let config = state.config.lock().unwrap();
                let count = *state.session_count.lock().unwrap();
                let log_path = crate::config::Config::data_dir().join("dictum.log");
                let config_path = crate::config::Config::data_dir().join("config.json");
                let cpu_threads = std::thread::available_parallelism()
                    .map(|n| n.get().min(8)).unwrap_or(4);
                let model_status = if config.is_model_ready() { "✓ présent" } else { "✗ MANQUANT" };
                let cli_status = if crate::config::Config::is_whisper_cli_ready() { "✓ présent" } else { "✗ MANQUANT" };
                let msg = format!(
                    "Dictum v{}\ngithub.com/painteau/Dictum\n\nModèle  : {} ({})\nwhisper : {}\nLangue  : {}\nHotkey  : {}\nThreads : {}\n\nSession : {} transcription{}\nConfig  : {}\nLog     : {}",
                    env!("CARGO_PKG_VERSION"),
                    config.model_name(),
                    model_status,
                    cli_status,
                    config.language_display(),
                    config.hotkey_string(),
                    cpu_threads,
                    count,
                    if count > 1 { "s" } else { "" },
                    config_path.display(),
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

        // Mettre à jour le label historique avec le nombre d'entrées
        let hist_count = state.history.lock().unwrap().len();
        item_history.set_text(format!("📋 Historique ({})", hist_count));

        // Update tooltip + icône selon état
        let recording = *state.is_recording.lock().unwrap();
        let transcribing = *state.is_transcribing.lock().unwrap();
        let count = *state.session_count.lock().unwrap();
        let (model_name, hotkey_str) = {
            let cfg = state.config.lock().unwrap();
            let m = cfg.model_path.file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or(cfg.model_name())
                .to_string();
            let h = cfg.hotkey_string();
            (m, h)
        };
        let tooltip = if recording {
            format!("Dictum — Enregistrement... (relâcher {})", hotkey_str)
        } else if transcribing {
            "Dictum — Transcription en cours...".to_string()
        } else if count > 0 {
            format!("Dictum [{}] — {} dictée{} | {}", model_name, count, if count > 1 { "s" } else { "" }, hotkey_str)
        } else {
            format!("Dictum [{}] — Maintenir {} pour dicter", model_name, hotkey_str)
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

fn confirm_dialog(title: &str, message: &str) -> bool {
    #[cfg(windows)]
    unsafe {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use winapi::um::winuser::{MessageBoxW, MB_ICONQUESTION, MB_YESNO, IDYES};

        let wide_title: Vec<u16> = OsStr::new(title).encode_wide().chain(std::iter::once(0)).collect();
        let wide_msg: Vec<u16> = OsStr::new(message).encode_wide().chain(std::iter::once(0)).collect();

        MessageBoxW(std::ptr::null_mut(), wide_msg.as_ptr(), wide_title.as_ptr(), MB_YESNO | MB_ICONQUESTION) == IDYES
    }

    #[cfg(not(windows))]
    { let _ = (title, message); true }
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

    Icon::from_rgba(rgba, size, size).expect("Impossible de créer l'icône tray")
}
