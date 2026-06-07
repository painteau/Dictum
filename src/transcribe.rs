use std::path::PathBuf;
use std::process::Command;
use anyhow::{anyhow, Result};
use crate::config::Config;

fn whisper_cli_path() -> PathBuf {
    Config::data_dir().join("whisper-cli.exe")
}

fn write_wav(samples: &[f32], path: &std::path::Path) -> Result<()> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec)
        .map_err(|e| anyhow!("Impossible de créer le WAV : {e}"))?;
    for &s in samples {
        let v = (s * 32767.0).clamp(-32768.0, 32767.0) as i16;
        writer.write_sample(v)
            .map_err(|e| anyhow!("Erreur écriture WAV : {e}"))?;
    }
    writer.finalize()
        .map_err(|e| anyhow!("Erreur finalisation WAV : {e}"))?;
    Ok(())
}

/// Supprime les codes d'échappement ANSI d'une chaîne.
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Consommer jusqu'à la lettre finale de la séquence
            for ch in chars.by_ref() {
                if ch.is_ascii_alphabetic() { break; }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Calcule le RMS des samples. Si trop bas = silence, pas la peine de transcrire.
fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() { return 0.0; }
    let sum: f32 = samples.iter().map(|s| s * s).sum();
    (sum / samples.len() as f32).sqrt()
}

pub fn transcribe(samples: &[f32], config: &Config) -> Result<String> {
    let transcribe_start = std::time::Instant::now();

    // Timeout adaptatif : 10s par seconde audio, minimum 60s, maximum 300s
    let audio_secs = (samples.len() as f64 / 16000.0) as u64;
    let adaptive_timeout = (audio_secs * 10).max(60).min(300);
    log::debug!("Timeout adaptatif whisper : {}s pour {:.1}s audio", adaptive_timeout, audio_secs as f64);

    if rms(samples) < config.silence_threshold {
        log::debug!("Silence détecté (RMS={:.4}), transcription ignorée", rms(samples));
        return Ok(String::new());
    }
    let cli = whisper_cli_path();
    if !cli.exists() {
        return Err(anyhow!(
            "whisper-cli.exe introuvable : {}\nLancer le wizard pour le télécharger.",
            cli.display()
        ));
    }
    if !config.model_path.exists() {
        return Err(anyhow!(
            "Modèle introuvable : {}\nLancer le wizard pour le télécharger.",
            config.model_path.display()
        ));
    }

    let wav_path = std::env::temp_dir().join("dictum_record.wav");
    write_wav(samples, &wav_path)?;

    // Vérifier que le WAV a bien été écrit
    let wav_size = std::fs::metadata(&wav_path).map(|m| m.len()).unwrap_or(0);
    if wav_size < 100 {
        return Err(anyhow!("Fichier WAV temp invalide ({} bytes)", wav_size));
    }
    log::debug!("WAV temp : {} KB", wav_size / 1024);

    let lang_str = config.language.clone();
    let lang_args: Vec<&str> = if lang_str == "auto" {
        vec![]
    } else {
        vec!["--language", &lang_str]
    };

    let cpu_threads = if config.whisper_threads > 0 {
        config.whisper_threads as usize
    } else {
        std::thread::available_parallelism()
            .map(|n| n.get().min(8))
            .unwrap_or(4)
    };
    log::debug!("whisper-cli : {} threads", cpu_threads);

    let mut cmd = Command::new(&cli);
    // current_dir = data_dir so Windows finds ggml.dll etc. in the same folder
    cmd.current_dir(Config::data_dir())
        .arg("--model").arg(&config.model_path)
        .arg("--no-timestamps")
        .arg("--threads").arg(cpu_threads.to_string())
        .arg("--best-of").arg("1")
        .arg("--beam-size").arg("1")
        .arg("--print-colors").arg("false")
        .arg("--print-special").arg("false")
        .arg("--word-thold").arg("0.01");

    if config.whisper_no_speech {
        cmd.arg("--no-speech-thold").arg("0.6");
    }
    if config.whisper_temperature != 0.0 {
        cmd.arg("--temperature").arg(format!("{:.2}", config.whisper_temperature));
    }

    cmd.arg("--file").arg(&wav_path);

    log::info!("whisper-cli : threads={} temp={:.1} lang={}",
        cpu_threads,
        config.whisper_temperature,
        if config.language == "auto" { "auto" } else { &config.language }
    );
    log::debug!("whisper-cli args: {:?}", cmd);

    for arg in &lang_args {
        cmd.arg(arg);
    }

    let mut child = cmd.spawn()
        .map_err(|e| anyhow!("Impossible de lancer whisper-cli : {e}"))?;

    // Timeout adaptatif calculé plus haut
    let timeout = std::time::Duration::from_secs(adaptive_timeout);
    let start = std::time::Instant::now();
    let output = loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                break child.wait_with_output()
                    .map_err(|e| anyhow!("Erreur lecture output : {e}"))?;
            }
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    return Err(anyhow!("Timeout whisper-cli ({}s dépassées)", adaptive_timeout));
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(e) => return Err(anyhow!("Erreur attente subprocess : {e}")),
        }
    };

    // Log stderr si non vide (warnings whisper)
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.trim().is_empty() {
        log::debug!("whisper-cli stderr : {}", stderr.trim());
    }

    // Nettoyage — whisper-cli génère parfois un .txt même sans --output-txt
    std::fs::remove_file(&wav_path).ok();
    let txt_sidecar = wav_path.with_extension("wav.txt");
    std::fs::remove_file(&txt_sidecar).ok();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout_len = output.stdout.len();
        // Si le code d'erreur est non-zéro mais stdout contient du texte, on tente quand même
        if stdout_len < 10 {
            return Err(anyhow!("whisper-cli a échoué (code {:?}) : {}", output.status.code(), stderr.trim()));
        }
        log::warn!("whisper-cli code {:?} mais {} bytes stdout — on tente l'extraction", output.status.code(), stdout_len);
    }

    let raw_stdout = String::from_utf8_lossy(&output.stdout);
    // Normaliser les fins de ligne Windows (CRLF → LF)
    let stdout_clean = raw_stdout.replace("\r\n", "\n").replace('\r', "\n");
    let text = stdout_clean
        .lines()
        // Filtre les tags Whisper : [BLANK_AUDIO], [Music], (Music), timestamps, etc.
        .filter(|l| {
            let t = l.trim();
            !t.is_empty()
                && !t.starts_with('[')
                && !t.starts_with('(')
                && !t.starts_with("-->")
                && t != "BLANK_AUDIO"
        })
        // Filtre les numéros de séquence SRT (lignes purement numériques)
        .filter(|l| !l.trim().chars().all(|c| c.is_ascii_digit()))
        .map(|l| l.trim())
        .map(strip_ansi)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();

    // Nettoyer les espaces multiples générés par la jointure de segments
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");

    // Détecter si Whisper a hallucination (texte répétitif)
    if !text.is_empty() {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() >= 4 {
            let first = words[0];
            let repeated = words.iter().filter(|&&w| w == first).count();
            if repeated > words.len() / 2 {
                log::warn!("Possible hallucination Whisper détectée (mot '{}' répété {}/{})", first, repeated, words.len());
            }
        }
    }

    let elapsed = transcribe_start.elapsed();
    let duration_secs = samples.len() as f32 / 16000.0;
    let preview = if text.len() > 50 { format!("{}...", &text[..47]) } else { text.clone() };
    let speed = duration_secs / elapsed.as_secs_f32().max(0.001);
    if text.is_empty() {
        log::warn!("Transcription vide après filtrage (stdout={} bytes)", output.stdout.len());
    } else {
        log::info!("Transcription OK [{:.1}x] : «{}»", speed, preview);
    }
    log::debug!("Détails : {:.1}s audio en {:.1}s", duration_secs, elapsed.as_secs_f32());

    Ok(text)
}

pub fn is_ready(config: &Config) -> bool {
    whisper_cli_path().exists() && config.model_path.exists()
}
