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

/// Transcrit un fichier audio et écrit le résultat dans <fichier>.txt
fn cli_transcribe(input: &std::path::Path) -> Result<()> {
    use crate::config::Config;
    use crate::transcribe;

    let config = Config::load()?;
    let samples = read_audio_file(input)?;
    let text = transcribe::transcribe(&samples, &config)?;

    let output = input.with_extension("txt");
    std::fs::write(&output, &text)?;
    println!("{}", text);
    println!("\nSauvegardé : {}", output.display());
    Ok(())
}

/// Lit un fichier audio (WAV 16kHz mono) en Vec<f32>.
/// Pour les autres formats, whisper-cli les gère nativement — on lui passe le fichier directement.
fn read_audio_file(path: &std::path::Path) -> Result<Vec<f32>> {
    let reader = hound::WavReader::open(path);
    match reader {
        Ok(mut r) => {
            let spec = r.spec();
            let samples: Vec<f32> = match spec.sample_format {
                hound::SampleFormat::Float => {
                    r.samples::<f32>().filter_map(|s| s.ok()).collect()
                }
                hound::SampleFormat::Int => {
                    let max = (1i32 << (spec.bits_per_sample - 1)) as f32;
                    r.samples::<i32>().filter_map(|s| s.ok()).map(|s| s as f32 / max).collect()
                }
            };
            Ok(samples)
        }
        Err(_) => {
            anyhow::bail!(
                "Format non supporté en CLI. Convertir en WAV 16kHz mono d'abord.\nEx: ffmpeg -i {} -ar 16000 -ac 1 out.wav",
                path.display()
            )
        }
    }
}

fn init_logger() {
    let log_dir = Config::data_dir();
    std::fs::create_dir_all(&log_dir).ok();

    let file_spec = flexi_logger::FileSpec::default()
        .directory(&log_dir)
        .basename("dictum")
        .suffix("log")
        .suppress_timestamp();

    flexi_logger::Logger::try_with_str("info")
        .unwrap()
        .log_to_file(file_spec)
        .duplicate_to_stderr(flexi_logger::Duplicate::Warn)
        .format(flexi_logger::opt_format)
        .start()
        .ok();
}

static UPDATE_AVAILABLE: std::sync::Mutex<Option<UpdateInfo>> = std::sync::Mutex::new(None);

pub fn take_update() -> Option<UpdateInfo> {
    UPDATE_AVAILABLE.lock().unwrap().take()
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    RecordStart,
    RecordStop,
    Quit,
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Mutex<Config>>,
    pub history: Arc<Mutex<History>>,
    pub is_recording: Arc<Mutex<bool>>,
    pub is_transcribing: Arc<Mutex<bool>>,
}

impl AppState {
    fn new() -> Result<Self> {
        Ok(Self {
            config: Arc::new(Mutex::new(Config::load()?)),
            history: Arc::new(Mutex::new(History::load()?)),
            is_recording: Arc::new(Mutex::new(false)),
            is_transcribing: Arc::new(Mutex::new(false)),
        })
    }
}

fn main() -> Result<()> {
    init_logger();
    log::info!("Dictum v{} démarrage", env!("CARGO_PKG_VERSION"));

    // Mode CLI
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("--version") | Some("-v") => {
            println!("Dictum v{}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Some("--help") | Some("-h") => {
            println!("Dictum v{} — Dictée vocale locale\n", env!("CARGO_PKG_VERSION"));
            println!("Usage:");
            println!("  dictum.exe               Lancer en mode tray (normal)");
            println!("  dictum.exe fichier.wav   Transcrire un fichier audio");
            println!("  dictum.exe --version     Afficher la version");
            return Ok(());
        }
        Some(path) => {
            let input = std::path::PathBuf::from(path);
            if input.exists() {
                return cli_transcribe(&input);
            }
        }
        None => {}
    }

    // Premier lancement : wizard si aucun modèle présent
    {
        let config = Config::load()?;
        if setup::needs_setup(&config) {
            log::info!("Premier lancement — démarrage du wizard");
            setup::run_wizard()?;
        }
    }

    // Check silencieux mise à jour en arrière-plan
    std::thread::spawn(|| {
        if let Some(info) = updater::check_update() {
            log::info!("Mise à jour disponible : v{}", info.version);
            // Notifié via le tray — on stocke dans une variable statique
            *UPDATE_AVAILABLE.lock().unwrap() = Some(info);
        }
    });

    let state = AppState::new()?;
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

            for event in &event_rx {
                match event {
                    AppEvent::RecordStart => {
                        if *state.is_transcribing.lock().unwrap() {
                            log::warn!("Busy transcribing, ignoring record request");
                            continue;
                        }
                        let config = state.config.lock().unwrap().clone();
                        if config.beep_enabled { audio::beep(800, 80); }
                        if config.pause_media { media::toggle_media(); }
                        match audio::RecordHandle::start(config.microphone.as_deref(), config.max_record_secs) {
                            Ok(handle) => {
                                *state.is_recording.lock().unwrap() = true;
                                record_handle = Some(handle);
                                log::info!("Recording started");
                            }
                            Err(e) => log::error!("Failed to start recording: {e}"),
                        }
                    }
                    AppEvent::RecordStop => {
                        *state.is_recording.lock().unwrap() = false;
                        let cfg_snap = state.config.lock().unwrap().clone();
                        if cfg_snap.pause_media { media::toggle_media(); }
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
                                    Ok(text) => {
                                        let text = substitution::apply(
                                            &state.config.lock().unwrap().substitutions,
                                            &text,
                                        );
                                        log::info!("Transcribed: {text}");
                                        state.history.lock().unwrap().push(text.clone());
                                        let _ = state.history.lock().unwrap().save();
                                        inject::inject_text(&text, &config);
                                    }
                                    Err(e) => log::error!("Transcription failed: {e}"),
                                }
                                *state.is_transcribing.lock().unwrap() = false;
                            });
                        }
                    }
                    AppEvent::Quit => break,
                }
            }
        });
    }

    // Main thread: tray icon + Windows message pump
    tray::run(state, event_tx)?;

    Ok(())
}
