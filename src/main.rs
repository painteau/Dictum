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

    let started = flexi_logger::Logger::try_with_str(&level)
        .map(|l| l
            .log_to_file(file_spec)
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
    Quit,
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Mutex<Config>>,
    pub history: Arc<Mutex<History>>,
    pub is_recording: Arc<Mutex<bool>>,
    pub is_transcribing: Arc<Mutex<bool>>,
    pub session_count: Arc<Mutex<u32>>,
}

impl AppState {
    fn new() -> Result<Self> {
        Ok(Self {
            config: Arc::new(Mutex::new(Config::load()?)),
            history: Arc::new(Mutex::new(History::load()?)),
            is_recording: Arc::new(Mutex::new(false)),
            is_transcribing: Arc::new(Mutex::new(false)),
            session_count: Arc::new(Mutex::new(0)),
        })
    }
}

fn main() -> Result<()> {
    // --debug active les logs détaillés avant l'init logger
    let debug_mode = std::env::args().any(|a| a == "--debug");
    if debug_mode {
        std::env::set_var("RUST_LOG", "debug");
    }
    init_logger();
    log::info!("Dictum v{} démarrage", env!("CARGO_PKG_VERSION"));

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
        log::info!("Config : langue={}, hotkey={}{}{}{}, modèle={}",
            config.language,
            if config.hotkey.ctrl { "Ctrl+" } else { "" },
            if config.hotkey.alt  { "Alt+"  } else { "" },
            if config.hotkey.shift{ "Shift+"} else { "" },
            config.hotkey.key,
            config.model_path.file_name().and_then(|n| n.to_str()).unwrap_or("?")
        );
        if config.config_version < 1 {
            log::warn!("Config version {} détectée — réinitialisation", config.config_version);
            Config::default().save()?;
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
        let hist_len = state.history.lock().unwrap().len();
        log::info!("Historique : {} entrée(s)", hist_len);
        log::info!("Modèle : {} ({})",
            cfg.model_path.file_name().and_then(|n| n.to_str()).unwrap_or("?"),
            if cfg.model_path.exists() { "présent" } else { "MANQUANT" }
        );
        if !cfg.substitutions.is_empty() {
            log::info!("Substitutions : {} règle(s)", cfg.substitutions.len());
        }
        if cfg.pause_media { log::info!("Pause médias : activée"); }
        if cfg.beep_enabled { log::info!("Beep : activé"); }
        log::info!("Whisper : threads={} temp={:.1} no_speech={}",
            if cfg.whisper_threads == 0 { "auto".to_string() } else { cfg.whisper_threads.to_string() },
            cfg.whisper_temperature,
            cfg.whisper_no_speech
        );
        log::info!("Injection : délai={}ms prefix_space={}", cfg.inject_delay_ms, cfg.prefix_space);
        log::info!("Enregistrement : max={}s min={}ms silence_thold={:.3}",
            cfg.max_record_secs, cfg.min_record_ms, cfg.silence_threshold);
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
                        if *state.is_transcribing.lock().unwrap() {
                            log::warn!("Busy transcribing, ignoring record request");
                            continue;
                        }
                        let config = state.config.lock().unwrap().clone();
                        if config.beep_enabled { audio::beep(config.beep_start_freq, 80); }
                        if config.pause_media { media::toggle_media(); }
                        match audio::RecordHandle::start(config.microphone.as_deref(), config.max_record_secs) {
                            Ok(handle) => {
                                *state.is_recording.lock().unwrap() = true;
                                record_handle = Some(handle);
                                record_start = Some(std::time::Instant::now());
                                log::info!("Recording started");
                            }
                            Err(e) => log::error!("Failed to start recording: {e}"),
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
                                    log::warn!("Recording too short, skipping");
                                    return;
                                }
                                *state.is_transcribing.lock().unwrap() = true;
                                let config = state.config.lock().unwrap().clone();
                                if !transcribe::is_ready(&config) {
                                    log::warn!("whisper-cli ou modèle manquant — relancer le wizard");
                                    *state.is_transcribing.lock().unwrap() = false;
                                    return;
                                }
                                match transcribe::transcribe(&samples, &config) {
                                    Ok(text) if text.is_empty() => {
                                        log::debug!("Silence détecté, rien injecté");
                                    }
                                    Ok(text) => {
                                        let text = substitution::apply(
                                            &state.config.lock().unwrap().substitutions,
                                            &text,
                                        );
                                        let preview = if text.len() > 100 {
                                            format!("{}...", &text[..97])
                                        } else {
                                            text.clone()
                                        };
                                        log::info!("Transcrit ({} chars) : {}", text.len(), preview);
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
                                *state.config.lock().unwrap() = new_cfg;
                                log::info!("Config rechargée depuis pipeline");
                            }
                            Err(e) => log::error!("Reload config échoué : {e}"),
                        }
                    }
                    AppEvent::Quit => break,
                }
            }
        });
    }

    // Main thread: tray icon + Windows message pump
    tray::run(state.clone(), event_tx)?;

    log::info!("Dictum arrêt propre ({} transcription(s) cette session)",
        *state.session_count.lock().unwrap()
    );
    Ok(())
}
