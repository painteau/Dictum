use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub key: String, // "F9", "F10", "Space", etc.
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self { ctrl: false, alt: false, shift: false, key: "F9".to_string() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Substitution {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Path to the .bin Whisper model (ggml format)
    pub model_path: PathBuf,
    /// Language code: "fr", "en", "auto"
    pub language: String,
    /// Hold this key to record, release to transcribe
    pub hotkey: HotkeyConfig,
    /// Press Enter automatically after injecting text
    pub auto_enter: bool,
    /// Add non-breaking spaces before ? ! : ;
    pub french_typography: bool,
    /// Capitalize first letter of transcription
    pub auto_capitalize: bool,
    /// Abbreviation/correction substitutions applied after transcription
    pub substitutions: Vec<Substitution>,
    /// Microphone device name (None = system default)
    pub microphone: Option<String>,
    /// Maximum recording duration in seconds
    pub max_record_secs: u64,
    /// Beep sonore au début et à la fin de chaque enregistrement
    pub beep_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model_path: data_dir().join("models").join("ggml-medium.bin"),
            language: "auto".to_string(),
            hotkey: HotkeyConfig::default(),
            auto_enter: false,
            french_typography: true,
            auto_capitalize: true,
            substitutions: vec![],
            microphone: None,
            max_record_secs: 30,
            beep_enabled: true,
        }
    }
}

fn data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Dictum")
}

fn config_path() -> PathBuf {
    data_dir().join("config.json")
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&content).unwrap_or_default())
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path();
        std::fs::create_dir_all(path.parent().unwrap())?;
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn open_in_editor() -> Result<()> {
        let path = config_path();
        std::fs::create_dir_all(path.parent().unwrap())?;
        // Ensure the file exists before opening
        if !path.exists() {
            Config::default().save()?;
        }
        std::process::Command::new("notepad").arg(&path).spawn()?;
        Ok(())
    }

    pub fn data_dir() -> PathBuf {
        data_dir()
    }
}
