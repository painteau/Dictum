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

pub fn transcribe(samples: &[f32], config: &Config) -> Result<String> {
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
    cmd.arg("--model").arg(&config.model_path)
        .arg("--output-txt")
        .arg("--no-timestamps")
        .arg("--file").arg(&wav_path);

    for arg in &lang_args {
        cmd.arg(arg);
    }

    let output = cmd.output()
        .map_err(|e| anyhow!("Impossible de lancer whisper-cli : {e}"))?;

    std::fs::remove_file(&wav_path).ok();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("whisper-cli a échoué : {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let text = stdout
        .lines()
        .filter(|l| !l.starts_with('[') && !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();

    Ok(text)
}

pub fn is_ready(config: &Config) -> bool {
    whisper_cli_path().exists() && config.model_path.exists()
}
