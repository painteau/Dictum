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
mod setup;
mod substitution;
mod transcribe;
mod tray;

use config::Config;
use history::History;

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
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    // Premier lancement : wizard si aucun modèle présent
    {
        let config = Config::load()?;
        if setup::needs_setup(&config) {
            log::info!("Premier lancement — démarrage du wizard");
            setup::run_wizard()?;
        }
    }

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
                        match audio::RecordHandle::start(config.microphone.as_deref()) {
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
