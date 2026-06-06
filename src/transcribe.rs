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

/// Calcule le RMS des samples. Si trop bas = silence, pas la peine de transcrire.
fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() { return 0.0; }
    let sum: f32 = samples.iter().map(|s| s * s).sum();
    (sum / samples.len() as f32).sqrt()
}

pub fn transcribe(samples: &[f32], config: &Config) -> Result<String> {
    let _start = std::time::Instant::now();
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

    let lang_args: Vec<&str> = if config.language == "auto" {
        vec![]
    } else {
        vec!["--language", &config.language]
    };

    let mut cmd = Command::new(&cli);
    // current_dir = data_dir so Windows finds ggml.dll etc. in the same folder
    cmd.current_dir(Config::data_dir())
        .arg("--model").arg(&config.model_path)
        .arg("--no-timestamps")
        .arg("--file").arg(&wav_path);

    for arg in &lang_args {
        cmd.arg(arg);
    }

    let mut child = cmd.spawn()
        .map_err(|e| anyhow!("Impossible de lancer whisper-cli : {e}"))?;

    // Timeout : 5 minutes max pour la transcription
    let timeout = std::time::Duration::from_secs(300);
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
                    return Err(anyhow!("Timeout whisper-cli (5 min dépassées)"));
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(e) => return Err(anyhow!("Erreur attente subprocess : {e}")),
        }
    };

    // Nettoyage — whisper-cli génère parfois un .txt même sans --output-txt
    std::fs::remove_file(&wav_path).ok();
    let txt_sidecar = wav_path.with_extension("wav.txt");
    std::fs::remove_file(&txt_sidecar).ok();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("whisper-cli a échoué : {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let text = stdout
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
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();

    let elapsed = start.elapsed();
    let duration_secs = samples.len() as f32 / 16000.0;
    log::info!("Transcription : {:.1}s audio en {:.1}s ({:.1}x temps réel)",
        duration_secs, elapsed.as_secs_f32(),
        duration_secs / elapsed.as_secs_f32().max(0.001)
    );

    Ok(text)
}

pub fn is_ready(config: &Config) -> bool {
    whisper_cli_path().exists() && config.model_path.exists()
}
