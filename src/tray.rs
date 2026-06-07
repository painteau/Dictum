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
    let item_pause     = MenuItem::new("⏸  Pause dictée", true, None);
    let item_live      = MenuItem::new("🎙  Mode live (streaming)", true, None);
    let item_settings  = MenuItem::new("⚙  Paramètres", true, None);
    let item_history   = MenuItem::new("📋 Historique (0)", true, None);
    let item_devices   = MenuItem::new("🎙  Microphones", true, None);
    let item_copy_last   = MenuItem::new("📋 Copier dernière dictée", true, None);
    let item_copy_all    = MenuItem::new("📋 Copier tout l'historique", true, None);
    let item_export_hist = MenuItem::new("💾 Exporter historique (Markdown)", true, None);
    let item_export_csv  = MenuItem::new("📊 Exporter historique (CSV)", true, None);
    let item_reset_cfg  = MenuItem::new("🔧 Réinitialiser la config", true, None);
    let item_session_stats = MenuItem::new("📊 Statistiques session", true, None);
    let item_clear_hist = MenuItem::new("🗑  Effacer l'historique", true, None);
    let item_reload     = MenuItem::new("↺  Recharger la config", true, None);
    let item_drop_file  = MenuItem::new("📂 Transcrire un fichier...", true, None);
    let item_reformulate = MenuItem::new("✨ Reformuler sélection", true, None);
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
        // Contrôle
        &item_pause,
        &item_live,
        &PredefinedMenuItem::separator(),
        // Config
        &item_settings,
        &item_reload,
        &item_reset_cfg,
        &PredefinedMenuItem::separator(),
        // Historique
        &item_history,
        &item_copy_last,
        &item_copy_all,
        &item_export_hist,
        &item_export_csv,
        &item_session_stats,
        &item_clear_hist,
        &PredefinedMenuItem::separator(),
        // Système
        &item_devices,
        &item_drop_file,
        &item_reformulate,
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
                crate::notify::notify_update(&info.version, size_mb);
                update_info = Some(info);
            }
        }

        // Mettre à jour labels selon état
        let paused = *state.is_paused.lock().unwrap();
        item_pause.set_text(if paused { "▶  Reprendre dictée" } else { "⏸  Pause dictée" });
        let live_running = state.live_session.lock().unwrap().is_running();
        item_live.set_text(if live_running { "⏹  Arrêter mode live" } else { "🎙  Mode live (streaming)" });

        if let Ok(ref event) = menu_rx.try_recv() {
            log::debug!("Menu tray : event id={:?}", event.id);
            if event.id == item_live.id() {
                if !crate::live::is_stream_available() {
                    show_dialog("Dictum — Mode live", "whisper-stream.exe introuvable.\nTélécharger via le wizard (paramètres → Avancé).");
                } else {
                    let cfg = state.config.lock().unwrap().clone();
                    let (tx, rx) = crossbeam_channel::bounded::<String>(32);
                    let cfg2 = cfg.clone();
                    // Thread d'injection live
                    std::thread::spawn(move || {
                        for text in rx {
                            crate::inject::inject_text(&text, &cfg2);
                        }
                    });
                    state.live_session.lock().unwrap().toggle(cfg, tx);
                    let running = state.live_session.lock().unwrap().is_running();
                    log::info!("Mode live : {}", if running { "démarré" } else { "arrêté" });
                }
            } else if event.id == item_pause.id() {
                let mut p = state.is_paused.lock().unwrap();
                *p = !*p;
                let new_state = *p;
                drop(p);
                let state_str = if new_state { "en pause" } else { "active" };
                log::info!("Dictée {} via menu tray", state_str);
                crate::notify::notify_pause(new_state);
            } else if event.id == item_quit.id() {
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
                let cfg = state.config.lock().unwrap().clone();
                let state_clone = state.clone();
                let tx_clone = event_tx.clone();
                std::thread::spawn(move || {
                    crate::settings::open(cfg);
                    // Après fermeture, recharger la config automatiquement
                    let _ = tx_clone.send(crate::AppEvent::ReloadConfig);
                    log::info!("Config rechargée après fermeture fenêtre paramètres");
                    // Notifier l'utilisateur si score a changé
                    if let Ok(new_cfg) = crate::config::Config::load() {
                        log::info!("Score config : {}/100", new_cfg.score());
                        let _ = state_clone; // suppress unused warning
                    }
                });
            } else if event.id == item_history.id() {
                let hist = state.history.lock().unwrap();
                let msg = if hist.is_empty() {
                    "Aucun historique.".to_string()
                } else {
                    hist.as_display_string()
                };
                drop(hist);
                show_dialog("Dictum — Historique", &msg);
            } else if event.id == item_session_stats.id() {
                let count = *state.session_count.lock().unwrap();
                let hist = state.history.lock().unwrap();
                let msg = format!(
                    "Session courante :\n  {} transcription{}\n  {} mots totaux\n\n{}",
                    count, if count > 1 { "s" } else { "" },
                    hist.words_count(),
                    hist.stats_summary()
                );
                drop(hist);
                show_dialog("Dictum — Statistiques session", &msg);
            } else if event.id == item_copy_all.id() {
                let hist = state.history.lock().unwrap();
                if hist.is_empty() {
                    show_dialog("Dictum", "Historique vide.");
                } else {
                    let all = hist.all_texts().join("\n");
                    drop(hist);
                    match arboard::Clipboard::new().and_then(|mut c| c.set_text(&all)) {
                        Ok(_) => log::info!("Historique complet copié ({} entrées)", all.lines().count()),
                        Err(e) => show_dialog("Dictum", &format!("Erreur clipboard : {e}")),
                    }
                }
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
            } else if event.id == item_export_csv.id() {
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default().as_secs();
                let export_path = crate::config::Config::data_dir()
                    .join(format!("history-{}.csv", ts));
                let csv = state.history.lock().unwrap().export_csv();
                match std::fs::write(&export_path, &csv) {
                    Ok(_) => {
                        std::process::Command::new("notepad").arg(&export_path).spawn().ok();
                    }
                    Err(e) => show_dialog("Dictum", &format!("Erreur export CSV : {e}")),
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
                let hist_stats = state.history.lock().unwrap().stats_summary();
                let log_path = crate::config::Config::log_path();
                let config_path = crate::config::Config::data_dir().join("config.json");
                let score = config.score();
                let diagnostic = config.diagnose();
                let issues = config.validate();
                let issues_str = if issues.is_empty() {
                    "✓ Configuration valide".to_string()
                } else {
                    format!("⚠ {} problème(s) :\n{}", issues.len(), issues.iter().map(|i| format!("  • {}", i)).collect::<Vec<_>>().join("\n"))
                };
                let score_label = config.score_label();
                let score_breakdown = config.score_breakdown_display();
                let lang_name = config.language_name();
                let model_dname = config.model_display_name();
                let msg = format!(
                    "github.com/painteau/Dictum\n\n{}\nLangue : {} | Modèle : {}\n\nScore config : {}/100 ({})\n{}\n\n{}\n\nSession : {} transcription{}\nHistorique : {}\nConfig  : {}\nLog     : {}",
                    diagnostic,
                    lang_name,
                    model_dname,
                    score,
                    score_label,
                    score_breakdown,
                    issues_str,
                    count,
                    if count > 1 { "s" } else { "" },
                    hist_stats,
                    config_path.display(),
                    log_path.display()
                );
                show_dialog("À propos de Dictum", &msg);
            } else if event.id == item_drop_file.id() {
                let cfg = state.config.lock().unwrap().clone();
                std::thread::spawn(move || {
                    crate::dropper::open(cfg);
                });
            } else if event.id == item_reformulate.id() {
                let cfg = state.config.lock().unwrap().clone();
                std::thread::spawn(move || {
                    // Récupérer le texte sélectionné
                    match crate::reformulate::get_selected_text() {
                        Some(text) if !text.is_empty() => {
                            log::info!("Reformulation : {} chars sélectionnés", text.len());
                            match crate::reformulate::reformulate(&text, &cfg.reformulation_style, &cfg.ollama_model, &cfg.ollama_url) {
                                Ok(result) => {
                                    // Injecter le résultat
                                    crate::inject::inject_text(&result, &cfg);
                                    log::info!("Reformulation injectée ({} chars)", result.len());
                                }
                                Err(e) => {
                                    log::error!("Reformulation échouée : {e}");
                                    crate::notify::show_toast("Dictum — Reformulation", &format!("Erreur : {e}"));
                                }
                            }
                        }
                        _ => {
                            crate::notify::show_toast("Dictum — Reformulation", "Aucun texte sélectionné.");
                        }
                    }
                });
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

        // Mettre à jour le label historique avec le nombre d'entrées et mots
        let (hist_count, hist_words) = {
            let h = state.history.lock().unwrap();
            (h.len(), h.words_count())
        };
        item_history.set_text(if hist_words > 0 {
            format!("📋 Historique ({} entrées, {} mots)", hist_count, hist_words)
        } else {
            format!("📋 Historique ({})", hist_count)
        });

        // Update tooltip + icône selon état
        let recording = *state.is_recording.lock().unwrap();
        let transcribing = *state.is_transcribing.lock().unwrap();
        let count = *state.session_count.lock().unwrap();
        let (desc, hotkey_str, lang_emoji) = {
            let cfg = state.config.lock().unwrap();
            (cfg.description(), cfg.hotkey_string(), cfg.language_emoji().to_string())
        };
        let words = state.history.lock().unwrap().words_count();
        let is_paused = *state.is_paused.lock().unwrap();
        let tooltip = if is_paused {
            format!("{} — EN PAUSE (cliquer tray pour reprendre)", desc)
        } else if recording {
            format!("Enregistrement... relâcher {}", hotkey_str)
        } else if transcribing {
            "Transcription en cours...".to_string()
        } else if count > 0 {
            format!("{} {} — {} dictée{} ({} mots)", lang_emoji, desc, count, if count > 1 { "s" } else { "" }, words)
        } else {
            format!("{} — Maintenir {}", desc, hotkey_str)
        };
        let _ = _tray.set_tooltip(Some(&tooltip));
        let icon = if is_paused { make_icon_paused() } else { make_icon(recording, transcribing) };
        let _ = _tray.set_icon(Some(icon));

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
/// rouge = enregistrement, orange = transcription, gris = pause, bleu = repos
fn make_icon_paused() -> Icon { make_icon_full(false, false, true) }
fn make_icon(recording: bool, transcribing: bool) -> Icon { make_icon_full(recording, transcribing, false) }
fn make_icon_full(recording: bool, transcribing: bool, paused: bool) -> Icon {
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
                } else if paused {
                    rgba[idx] = 100; rgba[idx+1] = 100; rgba[idx+2] = 100; // gris
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
