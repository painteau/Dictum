#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};
use std::thread;
use anyhow::Result;
use crossbeam_channel::bounded;


mod audio;
mod config;
mod downloader;
mod history;
mod hotkey;
mod inject;
mod media;
mod notify;
mod setup;
mod substitution;
mod transcribe;
mod tray;
mod updater;

use config::Config;
use history::History;
use updater::UpdateInfo;

/// Transcrit un fichier audio et écrit le résultat dans <fichier>.txt (ou --output path)
fn cli_transcribe(input: &std::path::Path, lang_override: Option<&str>) -> Result<()> {
    use crate::config::Config;
    use crate::transcribe;

    let args: Vec<String> = std::env::args().collect();
    let output_path = args.windows(2)
        .find(|w| w[0] == "--output" || w[0] == "-o")
        .map(|w| std::path::PathBuf::from(&w[1]))
        .unwrap_or_else(|| input.with_extension("txt"));

    let mut config = Config::load()?;
    if let Some(lang) = lang_override {
        config.language = lang.to_string();
        log::info!("Langue forcée : {}", lang);
    }
    // --model path/to/model.bin
    if let Some(model) = args.windows(2)
        .find(|w| w[0] == "--model" || w[0] == "-m")
        .map(|w| std::path::PathBuf::from(&w[1]))
    {
        if model.exists() {
            log::info!("Modèle forcé : {}", model.display());
            config.model_path = model;
        } else {
            anyhow::bail!("Modèle introuvable : {}", model.display());
        }
    }
    let stdout_only = args.iter().any(|a| a == "--stdout");
    let quiet = stdout_only || args.iter().any(|a| a == "--quiet" || a == "-q");
    let no_save = stdout_only || args.iter().any(|a| a == "--no-save");

    let samples = read_audio_file(input)?;
    let text = transcribe::transcribe(&samples, &config)?;

    if text.is_empty() {
        if !quiet { println!("[Silence détecté — aucun texte transcrit]"); }
        return Ok(());
    }

    if !no_save {
        std::fs::write(&output_path, &text)?;
    }
    if quiet {
        print!("{}", text);
    } else {
        println!("{}", text);
        if !no_save {
            println!("\nSauvegardé : {}", output_path.display());
        }
    }
    Ok(())
}

/// Lit un fichier WAV en Vec<f32> mono 16kHz.
/// Supporte : mono/stéréo, int/float, tout sample rate (pas de resampling).
/// Stéréo → moyenne des canaux. Resampling non supporté (suggère ffmpeg).
fn read_audio_file(path: &std::path::Path) -> Result<Vec<f32>> {
    let mut r = hound::WavReader::open(path)
        .map_err(|e| anyhow::anyhow!(
            "Impossible de lire {} : {e}\nConseil : convertir avec ffmpeg -i {} -ar 16000 -ac 1 out.wav",
            path.display(), path.display()
        ))?;

    let spec = r.spec();
    let channels = spec.channels as usize;

    if spec.sample_rate != 16000 {
        log::warn!("Sample rate {} Hz détecté (Whisper attend 16000 Hz) — qualité dégradée", spec.sample_rate);
    }

    let interleaved: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => r.samples::<f32>().filter_map(|s| s.ok()).collect(),
        hound::SampleFormat::Int => {
            let max = (1i32 << (spec.bits_per_sample.saturating_sub(1))) as f32;
            r.samples::<i32>().filter_map(|s| s.ok()).map(|s| s as f32 / max).collect()
        }
    };

    // Mix-down stéréo → mono (moyenne des canaux)
    if channels <= 1 {
        return Ok(interleaved);
    }
    let mono: Vec<f32> = interleaved
        .chunks(channels)
        .map(|ch| ch.iter().sum::<f32>() / channels as f32)
        .collect();

    log::info!("WAV {} canaux → mono ({} échantillons)", channels, mono.len());
    Ok(mono)
}

fn init_logger() {
    let log_dir = Config::data_dir();
    std::fs::create_dir_all(&log_dir).ok();

    let level = if std::env::var("RUST_LOG").is_ok() {
        std::env::var("RUST_LOG").unwrap()
    } else {
        "info".to_string()
    };

    let file_spec = flexi_logger::FileSpec::default()
        .directory(&log_dir)
        .basename("dictum")
        .suffix("log")
        .suppress_timestamp();

    let rotate = flexi_logger::Criterion::Size(5_000_000); // 5 MB
    let naming = flexi_logger::Naming::Timestamps;
    let cleanup = flexi_logger::Cleanup::KeepLogFiles(3);

    let started = flexi_logger::Logger::try_with_str(&level)
        .map(|l| l
            .log_to_file(file_spec)
            .rotate(rotate, naming, cleanup)
            .duplicate_to_stderr(flexi_logger::Duplicate::Warn)
            .format(flexi_logger::opt_format)
            .start()
        );

    if started.is_err() {
        // Fallback console uniquement
        env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .init();
    }
}

static UPDATE_AVAILABLE: std::sync::Mutex<Option<UpdateInfo>> = std::sync::Mutex::new(None);

pub fn take_update() -> Option<UpdateInfo> {
    UPDATE_AVAILABLE.lock().unwrap().take()
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    RecordStart,
    RecordStop,
    ReloadConfig,
    TogglePause,
    Quit,
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Mutex<Config>>,
    pub history: Arc<Mutex<History>>,
    pub is_recording: Arc<Mutex<bool>>,
    pub is_transcribing: Arc<Mutex<bool>>,
    pub is_paused: Arc<Mutex<bool>>,
    pub session_count: Arc<Mutex<u32>>,
}

impl AppState {
    fn new() -> Result<Self> {
        Ok(Self {
            config: Arc::new(Mutex::new(Config::load()?)),
            history: Arc::new(Mutex::new(History::load()?)),
            is_recording: Arc::new(Mutex::new(false)),
            is_transcribing: Arc::new(Mutex::new(false)),
            is_paused: Arc::new(Mutex::new(false)),
            session_count: Arc::new(Mutex::new(0)),
        })
    }
}

fn main() -> Result<()> {
    // --debug active les logs détaillés avant l'init logger
    let debug_mode = std::env::args().any(|a| a == "--debug");
    if debug_mode {
        std::env::set_var("RUST_LOG", "debug");
    } else if std::env::var("RUST_LOG").is_err() {
        // Lire le niveau depuis la config si disponible
        if let Ok(cfg) = Config::load() {
            if !cfg.log_level.is_empty() {
                std::env::set_var("RUST_LOG", &cfg.log_level);
            }
        }
    }
    init_logger();
    let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    let log_label = match log_level.as_str() {
        "error" => "🔴 error", "warn" => "🟡 warn", "info" => "🟢 info",
        "debug" => "🔵 debug", "trace" => "⚪ trace", other => other,
    };
    log::info!("Dictum v{} démarrage [{}]", env!("CARGO_PKG_VERSION"), log_label);
    if let Ok(c) = Config::load() {
        log::info!("{} | {}", c.config_version_display(), c.is_ready_message());
        log::debug!("\n{}", c.diagnose());
    }

    // Mode CLI
    let args: Vec<String> = std::env::args().collect();

    // --config path/to/config.json
    let config_override = args.windows(2)
        .find(|w| w[0] == "--config" || w[0] == "-c")
        .map(|w| std::path::PathBuf::from(&w[1]));
    if let Some(ref path) = config_override {
        log::info!("Config personnalisée : {}", path.display());
    }

    match args.get(1).map(String::as_str) {
        Some("--version") | Some("-v") => {
            println!("Dictum v{}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Some("--version-full") => {
            println!("Dictum v{}", env!("CARGO_PKG_VERSION"));
            println!("Build  : {} ({})", env!("CARGO_PKG_VERSION"), std::env::consts::ARCH);
            println!("Target : {}-{}", std::env::consts::OS, std::env::consts::ARCH);
            // Afficher l'info whisper-cli si disponible
            let cli = Config::data_dir().join("whisper-cli.exe");
            if cli.exists() {
                if let Ok(out) = std::process::Command::new(&cli).arg("--version").output() {
                    let ver = String::from_utf8_lossy(&out.stdout);
                    let ver = ver.trim();
                    if !ver.is_empty() { println!("whisper  : {}", ver); }
                }
                println!("whisper-cli.exe : présent");
            } else {
                println!("whisper-cli.exe : absent");
            }
            println!("Data dir : {}", Config::data_dir().display());
            return Ok(());
        }
        Some("--help") | Some("-h") => {
            println!("Dictum v{} — Dictée vocale locale\n", env!("CARGO_PKG_VERSION"));
            println!("Usage:");
            println!("  dictum.exe                           Mode tray (normal)");
            println!("  dictum.exe fichier.wav [options]     Transcrire un fichier WAV");
            println!();
            println!("Options transcription :");
            println!("  -l, --language CODE   Langue ISO (fr, en, auto...)");
            println!("  -m, --model PATH      Chemin vers le modèle .bin");
            println!("  -o, --output PATH     Fichier de sortie (défaut : fichier.txt)");
            println!("  -q, --quiet           Stdout uniquement, sans métadonnées");
            println!("      --no-save         Ne pas sauvegarder de fichier .txt");
            println!();
            println!("Informations :");
            println!("  --list-devices        Lister les microphones disponibles");
            println!("  --list-languages      Lister les langues Whisper (57)");
            println!("  --list-models         Lister les modèles disponibles (CDN)");
            println!("  --stats               Statistiques historique de dictée");
            println!("  --search <texte>      Rechercher dans l'historique");
            println!("  --diagnose            Rapport complet : fichiers, audio, réseau, config");
            println!("  --config-check        Valider la configuration sans démarrer");
            println!("  --reset-history       Effacer l'historique de dictée");
            println!("  --export [chemin]     Exporter l'historique en Markdown");
            println!("  --version, -v         Afficher la version");
            return Ok(());
        }
        Some("--list-devices") => {
            let devices = audio::list_devices();
            if devices.is_empty() {
                println!("Aucun microphone détecté.");
            } else {
                println!("Microphones disponibles :");
                for (i, d) in devices.iter().enumerate() {
                    println!("  [{}] {}", i, d);
                }
            }
            return Ok(());
        }
        Some("--list-languages") => {
            println!("Langues Whisper supportées (codes ISO 639-1) :");
            let langs = ["af","ar","hy","az","be","bs","bg","ca","zh","hr","cs","da","nl","en",
                         "et","fi","fr","gl","de","el","he","hi","hu","is","id","it","ja","kn",
                         "kk","ko","lv","lt","mk","ms","mr","mi","ne","no","fa","pl","pt","ro",
                         "ru","sr","sk","sl","es","sw","sv","tl","ta","th","tr","uk","ur","vi","cy"];
            for chunk in langs.chunks(8) {
                println!("  {}", chunk.join("  "));
            }
            println!("\nDétection automatique : utiliser \"auto\"");
            return Ok(());
        }
        Some("--export") => {
            let out_path = args.get(2)
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| {
                    let ts = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    std::path::PathBuf::from(format!("dictum-history-{}.md", ts))
                });
            match History::load() {
                Ok(h) => {
                    if h.is_empty() {
                        println!("Historique vide — rien à exporter.");
                        return Ok(());
                    }
                    match h.export_to_file(&out_path) {
                        Ok(_) => println!("Exporté : {} ({} entrées)", out_path.display(), h.len()),
                        Err(e) => { println!("Erreur export : {e}"); std::process::exit(1); }
                    }
                }
                Err(e) => { println!("Impossible de charger l'historique : {e}"); std::process::exit(1); }
            }
            return Ok(());
        }
        Some("--reset-history") => {
            match History::load() {
                Ok(mut h) => {
                    let count = h.len();
                    h.clear();
                    match h.save() {
                        Ok(_) => println!("Historique effacé ({} entrée{} supprimée{}).", count, if count > 1 { "s" } else { "" }, if count > 1 { "s" } else { "" }),
                        Err(e) => { println!("Erreur lors de la sauvegarde : {e}"); std::process::exit(1); }
                    }
                }
                Err(e) => { println!("Impossible de charger l'historique : {e}"); std::process::exit(1); }
            }
            return Ok(());
        }
        Some("--search") => {
            let query = args.get(2).map(String::as_str).unwrap_or("");
            if query.is_empty() {
                println!("Usage : dictum --search <texte>");
                std::process::exit(1);
            }
            match History::load() {
                Ok(h) => {
                    let results = h.search(query);
                    if results.is_empty() {
                        println!("Aucun résultat pour «{}».", query);
                    } else {
                        println!("{} résultat(s) pour «{}» :", results.len(), query);
                        for (i, e) in results.iter().enumerate() {
                            let preview = if e.text.len() > 100 { format!("{}...", &e.text[..97]) } else { e.text.clone() };
                            println!("  {}. {}", i + 1, preview);
                        }
                    }
                }
                Err(e) => { println!("Erreur : {e}"); std::process::exit(1); }
            }
            return Ok(());
        }
        Some("--diagnose") => {
            let cfg = Config::load().unwrap_or_default();
            println!("=== Diagnostic Dictum v{} ===\n", env!("CARGO_PKG_VERSION"));
            println!("{}", cfg.diagnose());
            println!("\n--- Fichiers ---");
            let cli_path = Config::data_dir().join("whisper-cli.exe");
            println!("whisper-cli.exe : {}", if cli_path.exists() { "✓ présent" } else { "✗ manquant" });
            println!("Modèle         : {}", if cfg.model_path.exists() { "✓ présent" } else { "✗ manquant" });
            println!("Config         : {}", Config::data_dir().join("config.json").display());
            println!("Log            : {}", Config::log_path().display());
            println!("\n--- Audio ---");
            let devices = audio::list_devices();
            println!("Microphones    : {}", if devices.is_empty() { "aucun détecté".to_string() } else { devices.len().to_string() + " détecté(s)" });
            for d in &devices { println!("  - {}", d); }
            println!("\n--- Réseau ---");
            let net = downloader::has_internet();
            println!("Internet       : {}", if net { "✓ connecté" } else { "✗ hors ligne" });
            println!("\n--- Score config : {}/100 ({}) ---", cfg.score(), cfg.score_label());
            println!("{}", cfg.score_breakdown_display());
            let issues = cfg.validate();
            if issues.is_empty() {
                println!("\n✓ Aucun problème détecté.");
            } else {
                println!("\n⚠ {} problème(s) :", issues.len());
                for i in &issues { println!("  • {}", i); }
                std::process::exit(1);
            }
            return Ok(());
        }
        Some("--config-check") => {
            match Config::load() {
                Ok(cfg) => {
                    println!("Config : {}", Config::data_dir().join("config.json").display());
                    println!("{}", cfg.diagnose());
                    let issues = cfg.validate();
                    if issues.is_empty() {
                        println!("\n✓ Configuration valide (score {}/100)", cfg.score());
                    } else {
                        println!("\n⚠ {} problème(s) :", issues.len());
                        for issue in &issues {
                            println!("  • {}", issue);
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    println!("Impossible de charger la config : {e}");
                    std::process::exit(1);
                }
            }
            return Ok(());
        }
        Some("--list-models") => {
            println!("Modèles Whisper disponibles sur le CDN :");
            println!();
            match downloader::fetch_manifest() {
                Ok(manifest) => {
                    for (name, entry) in &manifest.models {
                        let size_gb = entry.size_bytes as f64 / 1_073_741_824.0;
                        let local = Config::models_dir().join(format!("ggml-{}.bin", name));
                        let status = if local.exists() { "✓ installé" } else { "  absent" };
                        println!("  {} {} — {:.1} GB", status, name, size_gb);
                    }
                }
                Err(_) => {
                    // Afficher les modèles connus sans CDN
                    println!("  (hors ligne) medium — 1.5 GB");
                    println!("  (hors ligne) large-v3 — 3.0 GB");
                    println!("\n⚠ Manifest CDN inaccessible — vérifier la connexion.");
                }
            }
            println!();
            println!("Dossier modèles : {}", Config::models_dir().display());
            return Ok(());
        }
        Some("--stats") => {
            match History::load() {
                Ok(h) => {
                    println!("Historique Dictum");
                    println!("-----------------");
                    println!("{}", h.stats_summary());
                    println!("Mots uniques  : {}", h.unique_words_count());
                    println!("Mot fréquent  : {}", h.most_common_word().unwrap_or_else(|| "—".into()));
                    println!("Dictées auj.  : {}", h.entries_today().len());
                    if let Some(secs) = h.time_since_last() {
                        if secs < 3600 {
                            println!("Dernière dictée : il y a {}min", secs / 60);
                        } else {
                            println!("Dernière dictée : il y a {}h{}min", secs / 3600, (secs % 3600) / 60);
                        }
                    }
                }
                Err(e) => println!("Impossible de charger l'historique : {e}"),
            }
            return Ok(());
        }
        Some(path) => {
            let input = std::path::PathBuf::from(path);
            if input.exists() {
                // Support: dictum.exe fichier.wav --language fr
                let lang_override = args.get(2)
                    .filter(|a| *a == "--language" || *a == "-l")
                    .and_then(|_| args.get(3))
                    .cloned();
                return cli_transcribe(&input, lang_override.as_deref());
            }
        }
        None => {}
    }

    // Premier lancement : wizard si aucun modèle ou whisper-cli absent
    {
        let config = Config::load()?;
        log::info!("{}", config.description());
        config.log_summary();
        if config.config_version < 1 {
            log::warn!("Config version {} détectée — réinitialisation", config.config_version);
            Config::default().save()?;
        }
        // Valider la config et alerter les problèmes
        let issues = config.validate();
        if !issues.is_empty() {
            for issue in &issues {
                log::warn!("Config: {}", issue);
            }
        }
        if setup::needs_setup(&config) {
            log::info!("Premier lancement ou binaires manquants — démarrage du wizard");
            setup::run_wizard()?;
        }
    }

    // Check silencieux mise à jour — délai 10s pour laisser l'app démarrer
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_secs(10));
        if downloader::has_internet() {
            if let Some(info) = updater::check_update() {
                log::info!("Mise à jour disponible : v{}", info.version);
                *UPDATE_AVAILABLE.lock().unwrap() = Some(info);
            }
        } else {
            log::debug!("Pas d'internet au démarrage, check update ignoré");
        }
    });

    let state = AppState::new()?;
    {
        let cfg = state.config.lock().unwrap();
        log::info!("Historique : {}", state.history.lock().unwrap().stats_summary());
        log::info!("Modèle : {} ({})", cfg.model_name(),
            if cfg.is_model_ready() { "présent" } else { "MANQUANT" });
        if cfg.has_substitutions() {
            log::info!("Substitutions : {} règle(s) — {}", cfg.substitution_count(), cfg.substitutions_display());
        }
        if cfg.pause_media { log::info!("Pause médias : activée"); }
        log::info!("Micro : {}", cfg.microphone_display());
        log::info!("Beep : {}", cfg.beep_description());
        log::info!("Whisper : {}", cfg.whisper_config_display());
        log::info!("Injection : mode=[{}] délai={}ms", cfg.inject_mode_label(), cfg.inject_delay_ms);
        log::info!("Enregistrement : durée=[{}] silence=[{}]",
            cfg.record_duration_label(), cfg.silence_level_label());
    }
    let (event_tx, event_rx) = bounded::<AppEvent>(32);

    // rdev::listen blocks — must live in its own thread
    {
        let tx = event_tx.clone();
        let config = state.config.lock().unwrap().clone();
        thread::spawn(move || {
            hotkey::start(config, tx);
        });
    }

    // Processing thread: record → transcribe → inject pipeline
    {
        let state = state.clone();
        thread::spawn(move || {
            let mut record_handle: Option<audio::RecordHandle> = None;
            let mut record_start: Option<std::time::Instant> = None;

            for event in &event_rx {
                match event {
                    AppEvent::RecordStart => {
                        if *state.is_paused.lock().unwrap() {
                            log::debug!("Dictée en pause — hotkey ignorée");
                            continue;
                        }
                        if *state.is_transcribing.lock().unwrap() {
                            log::warn!("Transcription en cours — hotkey ignorée");
                            continue;
                        }
                        let config = state.config.lock().unwrap().clone();
                        if config.beep_enabled { audio::beep(config.beep_start_freq, config.beep_duration_ms); }
                        if config.pause_media { media::toggle_media(); }
                        match audio::RecordHandle::start(config.microphone.as_deref(), config.max_record_secs) {
                            Ok(handle) => {
                                *state.is_recording.lock().unwrap() = true;
                                record_handle = Some(handle);
                                record_start = Some(std::time::Instant::now());
                                log::info!("Enregistrement démarré");
                            }
                            Err(e) => log::error!("Impossible de démarrer l'enregistrement : {e}"),
                        }
                    }
                    AppEvent::RecordStop => {
                        *state.is_recording.lock().unwrap() = false;
                        let cfg_snap = state.config.lock().unwrap().clone();
                        if cfg_snap.pause_media { media::toggle_media(); }
                        let elapsed_ms = record_start.take()
                            .map(|t| t.elapsed().as_millis() as u64)
                            .unwrap_or(0);
                        if elapsed_ms < cfg_snap.min_record_ms {
                            log::warn!("Enregistrement trop court ({}ms < {}ms), ignoré", elapsed_ms, cfg_snap.min_record_ms);
                            record_handle.take();
                            continue;
                        }
                        if let Some(handle) = record_handle.take() {
                            let state = state.clone();
                            thread::spawn(move || {
                                let samples = handle.stop();
                                if samples.len() < 1600 {
                                    log::warn!("Enregistrement trop court, ignoré");
                                    return;
                                }
                                *state.is_transcribing.lock().unwrap() = true;
                                let config = state.config.lock().unwrap().clone();
                                log::debug!("Pipeline : {} samples reçus, lancement transcription", samples.len());
                                if !config.is_fully_ready() {
                                    log::warn!("Dictum non prêt — relancer le wizard pour télécharger les outils");
                                    *state.is_transcribing.lock().unwrap() = false;
                                    return;
                                }
                                match transcribe::transcribe(&samples, &config) {
                                    Ok(text) if text.is_empty() => {
                                        log::info!("Résultat vide (silence ou filtré), rien injecté");
                                    }
                                    Ok(text) => {
                                        let text = state.config.lock().unwrap().apply_substitutions(&text);
                                        log::debug!("Pipeline reçoit texte : {} chars", text.len());
                                        *state.session_count.lock().unwrap() += 1;
                                        let max_h = config.max_history;
                                        state.history.lock().unwrap().push_with_limit(text.clone(), max_h);
                                        let _ = state.history.lock().unwrap().save();
                                        inject::inject_text(&text, &config);
                                    }
                                    Err(e) => log::error!("Transcription échouée : {e}"),
                                }
                                *state.is_transcribing.lock().unwrap() = false;
                            });
                        }
                    }
                    AppEvent::ReloadConfig => {
                        match Config::load() {
                            Ok(new_cfg) => {
                                new_cfg.log_summary();
                                *state.config.lock().unwrap() = new_cfg;
                                log::info!("Config rechargée depuis pipeline");
                            }
                            Err(e) => log::error!("Reload config échoué : {e}"),
                        }
                    }
                    AppEvent::TogglePause => {
                        let mut p = state.is_paused.lock().unwrap();
                        *p = !*p;
                        log::info!("Dictée {}", if *p { "mise en pause" } else { "reprise" });
                    }
                    AppEvent::Quit => break,
                }
            }
        });
    }

    // Main thread: tray icon + Windows message pump
    tray::run(state.clone(), event_tx)?;

    let count = *state.session_count.lock().unwrap();
    let hist = state.history.lock().unwrap();
    let hist_count = hist.len();
    let words = hist.words_count();
    drop(hist);
    log::info!("Dictum arrêt propre — {} transcription(s) cette session, {} mots total historique ({} entrées)", count, words, hist_count);
    Ok(())
}
