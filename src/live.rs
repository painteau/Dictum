/// Mode live streaming via whisper-stream.exe
/// Lance whisper-stream.exe en subprocess, capture stdout ligne par ligne, injecte au fil de l'eau.

use std::sync::{Arc, Mutex};
use crossbeam_channel::Sender;
use crate::config::Config;

#[derive(Debug, Clone, PartialEq)]
pub enum LiveState {
    Idle,
    Running,
}

pub struct LiveSession {
    pub state: Arc<Mutex<LiveState>>,
    child: Arc<Mutex<Option<std::process::Child>>>,
}

impl LiveSession {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(LiveState::Idle)),
            child: Arc::new(Mutex::new(None)),
        }
    }

    pub fn is_running(&self) -> bool {
        *self.state.lock().unwrap() == LiveState::Running
    }

    /// Démarre whisper-stream.exe et injecte le texte au fil de l'eau.
    pub fn start(&mut self, config: Config, inject_tx: Sender<String>) {
        if self.is_running() { return; }

        let stream_exe = Config::data_dir().join("whisper-stream.exe");
        if !stream_exe.exists() {
            log::error!("whisper-stream.exe introuvable — télécharger via le wizard");
            return;
        }

        let cpu_threads = if config.whisper_threads > 0 {
            config.whisper_threads as usize
        } else {
            std::thread::available_parallelism().map(|n| n.get().min(4)).unwrap_or(4)
        };

        let mut cmd = std::process::Command::new(&stream_exe);
        cmd.current_dir(Config::data_dir())
            .arg("--model").arg(&config.model_path)
            .arg("--threads").arg(cpu_threads.to_string())
            .arg("--step").arg("3000")    // chunk de 3s
            .arg("--length").arg("10000") // fenêtre de 10s
            .arg("--keep").arg("200")     // garde 200ms de contexte
            .arg("--language").arg(if config.language == "auto" { "auto" } else { &config.language })
            .arg("--print-special").arg("false")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null());

        if config.translate_to == "en" {
            cmd.arg("--task").arg("translate");
        }

        match cmd.spawn() {
            Ok(mut child) => {
                *self.state.lock().unwrap() = LiveState::Running;
                log::info!("whisper-stream.exe démarré (threads={} step=3000ms)", cpu_threads);

                let stdout = child.stdout.take().unwrap();
                let child_arc = self.child.clone();
                *child_arc.lock().unwrap() = Some(child);

                let state = self.state.clone();

                std::thread::spawn(move || {
                    use std::io::BufRead;
                    let reader = std::io::BufReader::new(stdout);
                    let mut last_injected = String::new();

                    for line in reader.lines() {
                        match line {
                            Ok(raw) => {
                                let text = clean_stream_output(&raw);
                                if text.is_empty() { continue; }
                                if text == last_injected { continue; }

                                // Filtre hallucinations courantes
                                if is_hallucination(&text) {
                                    log::debug!("Live : hallucination filtrée : {:?}", text);
                                    continue;
                                }

                                log::debug!("Live stream : «{}»", text);
                                last_injected = text.clone();
                                let _ = inject_tx.send(text);
                            }
                            Err(_) => break,
                        }
                    }

                    *state.lock().unwrap() = LiveState::Idle;
                    log::info!("whisper-stream.exe terminé");
                });
            }
            Err(e) => {
                log::error!("Impossible de lancer whisper-stream.exe : {e}");
            }
        }
    }

    /// Arrête le stream live en killant le subprocess.
    pub fn stop(&mut self) {
        if let Some(mut child) = self.child.lock().unwrap().take() {
            let _ = child.kill();
            log::info!("whisper-stream.exe arrêté");
        }
        *self.state.lock().unwrap() = LiveState::Idle;
    }

    pub fn toggle(&mut self, config: Config, inject_tx: Sender<String>) {
        if self.is_running() {
            self.stop();
        } else {
            self.start(config, inject_tx);
        }
    }
}

/// Vérifie si whisper-stream.exe est présent dans le dossier data.
pub fn is_stream_available() -> bool {
    Config::data_dir().join("whisper-stream.exe").exists()
}

/// Nettoie la sortie de whisper-stream (ANSI, timestamps, etc.)
fn clean_stream_output(raw: &str) -> String {
    // Supprimer codes ANSI
    let mut s = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            for ch in chars.by_ref() {
                if ch.is_ascii_alphabetic() { break; }
            }
        } else {
            s.push(c);
        }
    }

    // Supprimer timestamps [HH:MM:SS.mmm --> HH:MM:SS.mmm]
    let s = regex_remove_timestamps(&s);

    // Supprimer tags [BLANK_AUDIO] (musique) etc.
    let s = s.trim().to_string();
    if s.starts_with('[') || s.starts_with('(') { return String::new(); }

    // Nettoyer espaces multiples
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn regex_remove_timestamps(s: &str) -> String {
    // Pattern simple : [00:00:00.000 --> 00:00:00.000]
    let mut out = String::new();
    let mut skip = false;
    for c in s.chars() {
        if c == '[' { skip = true; }
        if !skip { out.push(c); }
        if c == ']' { skip = false; }
    }
    out
}

fn is_hallucination(text: &str) -> bool {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() < 4 { return false; }
    // Mot répété > 50%
    let first = words[0];
    let count = words.iter().filter(|&&w| w == first).count();
    count > words.len() / 2
}
