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
    #[serde(default)]
    pub case_insensitive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Version du fichier config (pour migrations futures)
    #[serde(default = "default_config_version")]
    pub config_version: u32,
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
    /// Durée minimale d'enregistrement en millisecondes (évite déclenchements accidentels)
    pub min_record_ms: u64,
    /// Nombre maximum d'entrées dans l'historique
    pub max_history: usize,
    /// Beep sonore au début et à la fin de chaque enregistrement
    pub beep_enabled: bool,
    /// Seuil RMS en-dessous duquel l'audio est considéré comme silence (0.0–1.0)
    pub silence_threshold: f32,
    /// Mettre en pause les médias (Spotify, VLC...) pendant l'enregistrement
    pub pause_media: bool,
    /// Ajouter un espace avant le texte injecté (utile si curseur milieu de phrase)
    pub prefix_space: bool,
    /// Nombre de threads CPU pour whisper-cli (0 = auto-détection)
    #[serde(default)]
    pub whisper_threads: u32,
    /// Délai en ms avant injection texte (laisser le système traiter le key-release)
    #[serde(default = "default_inject_delay")]
    pub inject_delay_ms: u64,
    /// Passer --no-speech-threshold à whisper pour ignorer segments sans parole
    #[serde(default)]
    pub whisper_no_speech: bool,
    /// Température whisper (0.0=déterministe, 1.0=créatif) — défaut 0.0 pour dictée
    #[serde(default = "default_whisper_temperature")]
    pub whisper_temperature: f32,
    /// Fréquence Hz du beep de début d'enregistrement
    #[serde(default = "default_beep_start_freq")]
    pub beep_start_freq: u32,
    /// Fréquence Hz du beep de fin d'enregistrement
    #[serde(default = "default_beep_end_freq")]
    pub beep_end_freq: u32,
    /// Durée en ms des beeps
    #[serde(default = "default_beep_duration")]
    pub beep_duration_ms: u32,
    /// Niveau de log : "error", "warn", "info", "debug"
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            config_version: 1,
        model_path: data_dir().join("models").join("ggml-medium.bin"),
            language: "auto".to_string(),
            hotkey: HotkeyConfig::default(),
            auto_enter: false,
            french_typography: true,
            auto_capitalize: true,
            substitutions: vec![],
            microphone: None,
            max_record_secs: 30,
            min_record_ms: 300,
            max_history: 10,
            beep_enabled: true,
            silence_threshold: 0.005,
            pause_media: false,
            prefix_space: false,
            whisper_threads: 0,
            inject_delay_ms: 80,
            whisper_no_speech: false,
            whisper_temperature: 0.0,
            beep_start_freq: 800,
            beep_end_freq: 600,
            beep_duration_ms: 80,
            log_level: "info".to_string(),
        }
    }
}

fn default_config_version() -> u32 { 1 }
fn default_inject_delay() -> u64 { 80 }
fn default_whisper_temperature() -> f32 { 0.0 }
fn default_beep_start_freq() -> u32 { 800 }
fn default_beep_end_freq() -> u32 { 600 }
fn default_beep_duration() -> u32 { 80 }
fn default_log_level() -> String { "info".to_string() }

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
        Self::load_from(&config_path())
    }

    pub fn load_from(path: &std::path::Path) -> Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let mut config: Config = serde_json::from_str(&content).unwrap_or_default();
            config.sanitize();
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Corrige les valeurs invalides et log les corrections.
    fn sanitize(&mut self) {
        if self.max_record_secs == 0 {
            self.max_record_secs = 30;
            log::warn!("max_record_secs=0 invalide, réinitialisé à 30");
        }
        if self.min_record_ms > 5000 {
            self.min_record_ms = 5000;
            log::warn!("min_record_ms > 5000, limité à 5000");
        }
        if self.max_history == 0 || self.max_history > 100 {
            self.max_history = 10;
            log::warn!("max_history hors limites, réinitialisé à 10");
        }
        let before = self.silence_threshold;
        self.silence_threshold = self.silence_threshold.clamp(0.0, 1.0);
        if (before - self.silence_threshold).abs() > 0.001 {
            log::warn!("silence_threshold {} hors limites, clamped à {:.4}", before, self.silence_threshold);
        }
        if self.hotkey.key.is_empty() { self.hotkey.key = "F9".to_string(); }
        if self.language.is_empty() { self.language = "auto".to_string(); }
        if self.inject_delay_ms > 1000 { self.inject_delay_ms = 1000; }
        if !["error","warn","info","debug","trace"].contains(&self.log_level.as_str()) {
            log::warn!("log_level '{}' invalide, réinitialisé à 'info'", self.log_level);
            self.log_level = "info".to_string();
        }
        if self.whisper_threads > 32 { self.whisper_threads = 32; }
        self.whisper_temperature = self.whisper_temperature.clamp(0.0, 1.0);
        if self.beep_start_freq < 100 || self.beep_start_freq > 20000 { self.beep_start_freq = 800; }
        if self.beep_end_freq < 100 || self.beep_end_freq > 20000 { self.beep_end_freq = 600; }
        if self.beep_duration_ms == 0 || self.beep_duration_ms > 2000 { self.beep_duration_ms = 80; }
        // model_path doit pointer vers un .bin
        if let Some(ext) = self.model_path.extension() {
            if ext != "bin" {
                self.model_path = data_dir().join("models").join("ggml-medium.bin");
                log::warn!("model_path invalide, réinitialisé au défaut");
            }
        }
    }

    pub fn save(&self) -> Result<()> {
        self.save_to(&config_path())
    }

    pub fn save_to(&self, path: &std::path::Path) -> Result<()> {
        std::fs::create_dir_all(path.parent().unwrap())?;
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn open_in_editor() -> Result<()> {
        let path = config_path();
        std::fs::create_dir_all(path.parent().unwrap())?;
        if !path.exists() {
            Config::default().save()?;
        }
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "notepad".to_string());
        std::process::Command::new(&editor).arg(&path).spawn()
            .or_else(|_| std::process::Command::new("notepad").arg(&path).spawn())?;
        Ok(())
    }

    pub fn data_dir() -> PathBuf {
        data_dir()
    }

    /// Version de l'application (depuis Cargo.toml).
    pub fn app_version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Réinitialise et sauvegarde la config aux valeurs par défaut.
    pub fn reset_to_default() -> Result<Self> {
        let cfg = Config::default();
        cfg.save()?;
        log::info!("Config réinitialisée aux valeurs par défaut (v{})", Self::app_version());
        Ok(cfg)
    }

    pub fn open_data_dir() -> Result<()> {
        std::process::Command::new("explorer")
            .arg(data_dir())
            .spawn()?;
        Ok(())
    }

    /// Affiche un résumé compact de la config dans les logs.
    pub fn log_summary(&self) {
        let flags: Vec<&str> = [
            if self.french_typography { Some("typo_fr") } else { None },
            if self.auto_capitalize { Some("capitalize") } else { None },
            if self.beep_enabled { Some("beep") } else { None },
            if self.pause_media { Some("pause_media") } else { None },
            if self.prefix_space { Some("prefix_space") } else { None },
            if self.auto_enter { Some("auto_enter") } else { None },
        ].iter().filter_map(|&o| o).collect();
        let flags_str = if flags.is_empty() { "aucun".to_string() } else { flags.join(",") };
        log::info!("Config résumé [{}] : lang={} model=[{}|{}] hotkey={} threads={} rec=[{}] silence={} inject=[{}]",
            self.profile_name(),
            self.language_display(),
            self.model_name(),
            self.whisper_speed_label(),
            self.hotkey_string(),
            self.threads_display(),
            self.record_duration_label(),
            self.silence_level_label(),
            self.inject_mode_label()
        );
    }

    /// Vérifie si le modèle Whisper est téléchargé.
    pub fn is_model_ready(&self) -> bool {
        self.model_path.exists()
    }

    /// Vérifie si whisper-cli.exe est disponible.
    pub fn is_whisper_cli_ready() -> bool {
        data_dir().join("whisper-cli.exe").exists()
    }

    /// Vérifie que tout est en ordre pour transcrire.
    pub fn is_fully_ready(&self) -> bool {
        self.is_model_ready() && Self::is_whisper_cli_ready()
    }

    pub fn log_path() -> PathBuf {
        data_dir().join("dictum.log")
    }

    pub fn history_export_path() -> PathBuf {
        data_dir().join("historique_export.txt")
    }

    pub fn models_dir() -> PathBuf {
        data_dir().join("models")
    }

    pub fn substitution_count(&self) -> usize {
        self.substitutions.len()
    }

    /// Retourne la liste des problèmes de configuration (vide si tout est OK).
    pub fn description(&self) -> String {
        format!("[{}] {} | {} | hotkey:{} | silence:{} | beep:{}",
            self.profile_name(),
            self.language_display(),
            self.model_name(),
            self.hotkey_string(),
            self.silence_level_label(),
            self.beep_description()
        )
    }

    pub fn whisper_config_display(&self) -> String {
        format!("threads={} temp={:.1} no_speech={} speed={}",
            self.threads_display(),
            self.whisper_temperature,
            self.whisper_no_speech,
            self.whisper_speed_label()
        )
    }

    pub fn profile_name(&self) -> &'static str {
        match (self.french_typography, self.auto_capitalize, self.pause_media) {
            (true, true, false) => "Français standard",
            (true, true, true)  => "Français professionnel",
            (false, true, false) => "Standard",
            (false, false, false) => "Minimal",
            _ => "Personnalisé",
        }
    }

    pub fn is_ready_message(&self) -> String {
        let issues = self.validate();
        if issues.is_empty() {
            format!("Dictum {} prêt — {}", Self::app_version(), self.model_name())
        } else {
            format!("{} problème(s) : {}", issues.len(), issues.join(", "))
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();
        if !self.is_model_ready() {
            issues.push(format!("Modèle absent : {}", self.model_path.display()));
        }
        if !Self::is_whisper_cli_ready() {
            issues.push("whisper-cli.exe manquant".to_string());
        }
        if self.hotkey.key.is_empty() {
            issues.push("Hotkey non définie".to_string());
        }
        if self.max_record_secs == 0 {
            issues.push("max_record_secs = 0 invalide".to_string());
        }
        if self.silence_threshold >= 1.0 {
            issues.push(format!("silence_threshold={} très élevé — aucune dictée passera", self.silence_threshold));
        }
        if self.inject_delay_ms > 500 {
            issues.push(format!("inject_delay_ms={} élevé — injection lente", self.inject_delay_ms));
        }
        if self.model_path.exists() {
            if let Some(ext) = self.model_path.extension() {
                if ext != "bin" {
                    issues.push(format!("Modèle '{}' n'est pas un .bin", self.model_path.display()));
                }
            }
        }
        if self.whisper_temperature > 0.5 {
            issues.push(format!("whisper_temperature={:.1} élevé — risque d'hallucinations", self.whisper_temperature));
        }
        issues
    }

    pub fn full_status(&self) -> String {
        format!(
            "Dictum v{} | Modèle: {} [{}] | Langue: {} | Hotkey: {}\nWhisper: threads={} temp={:.1} | Enregistrement: {} | Silence: {}\nInjection: [{}] | Beep: {}",
            Self::app_version(),
            self.model_name(), if self.is_model_ready() { "✓" } else { "✗" },
            self.language_display(),
            self.hotkey_string(),
            self.threads_display(), self.whisper_temperature,
            self.record_duration_label(),
            self.silence_level_label(),
            self.inject_mode_label(),
            self.beep_description()
        )
    }

    pub fn threads_display(&self) -> String {
        if self.whisper_threads == 0 {
            format!("auto (max {})", std::thread::available_parallelism().map(|n| n.get().min(8)).unwrap_or(4))
        } else {
            self.whisper_threads.to_string()
        }
    }

    pub fn record_duration_label(&self) -> String {
        format!("{}ms–{}s", self.min_record_ms, self.max_record_secs)
    }

    pub fn inject_mode_label(&self) -> String {
        let mut parts = Vec::new();
        if self.auto_capitalize { parts.push("majuscule"); }
        if self.french_typography { parts.push("typo_fr"); }
        if self.prefix_space { parts.push("espace_avant"); }
        if self.auto_enter { parts.push("auto_enter"); }
        if parts.is_empty() { "minimal".to_string() } else { parts.join("+") }
    }

    pub fn whisper_speed_label(&self) -> &'static str {
        match self.model_name() {
            name if name.contains("tiny") => "ultra-rapide",
            name if name.contains("base") => "très rapide",
            name if name.contains("small") => "rapide",
            name if name.contains("medium") => "normal",
            name if name.contains("large") => "lent",
            _ => "inconnu",
        }
    }

    pub fn beep_description(&self) -> String {
        if !self.beep_enabled {
            "désactivé".to_string()
        } else {
            format!("{}Hz/{}Hz {}ms", self.beep_start_freq, self.beep_end_freq, self.beep_duration_ms)
        }
    }

    pub fn silence_level_label(&self) -> &'static str {
        match self.silence_threshold {
            t if t < 0.001 => "désactivé",
            t if t < 0.003 => "très faible",
            t if t < 0.008 => "normal",
            t if t < 0.02  => "élevé",
            _              => "très élevé",
        }
    }

    pub fn has_substitutions(&self) -> bool {
        !self.substitutions.is_empty()
    }

    pub fn substitutions_display(&self) -> String {
        if self.substitutions.is_empty() {
            "aucune".to_string()
        } else {
            let previews: Vec<String> = self.substitutions.iter().take(3)
                .map(|s| format!("{}→{}", s.from, s.to))
                .collect();
            let more = if self.substitutions.len() > 3 { format!(" +{}", self.substitutions.len() - 3) } else { String::new() };
            format!("{}{}", previews.join(", "), more)
        }
    }

    pub fn apply_substitutions(&self, text: &str) -> String {
        crate::substitution::apply(&self.substitutions, text)
    }

    #[allow(dead_code)]
    pub fn add_substitution(&mut self, from: &str, to: &str, case_insensitive: bool) {
        if !from.is_empty() && !self.substitutions.iter().any(|s| s.from == from) {
            self.substitutions.push(Substitution { from: from.to_string(), to: to.to_string(), case_insensitive });
        }
    }

    #[allow(dead_code)]
    pub fn remove_substitution(&mut self, from: &str) {
        self.substitutions.retain(|s| s.from != from);
    }

    #[allow(dead_code)]
    pub fn clear_substitutions(&mut self) {
        self.substitutions.clear();
    }

    #[allow(dead_code)]
    pub fn toggle_beep(&mut self) -> &mut Self {
        self.beep_enabled = !self.beep_enabled;
        log::debug!("Beep : {}", if self.beep_enabled { "activé" } else { "désactivé" });
        self
    }

    #[allow(dead_code)]
    pub fn toggle_pause_media(&mut self) -> &mut Self {
        self.pause_media = !self.pause_media;
        log::debug!("Pause médias : {}", if self.pause_media { "activée" } else { "désactivée" });
        self
    }

    #[allow(dead_code)]
    pub fn set_max_history(&mut self, max: usize) -> &mut Self {
        self.max_history = max.clamp(1, 100);
        self
    }

    #[allow(dead_code)]
    pub fn set_max_record_secs(&mut self, secs: u64) -> &mut Self {
        self.max_record_secs = secs.max(5).min(300);
        self
    }

    #[allow(dead_code)]
    pub fn set_silence_threshold(&mut self, threshold: f32) -> &mut Self {
        self.silence_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    #[allow(dead_code)]
    pub fn set_threads(&mut self, threads: u32) -> &mut Self {
        self.whisper_threads = threads.min(32);
        self
    }

    #[allow(dead_code)]
    pub fn set_model(&mut self, model_path: std::path::PathBuf) -> &mut Self {
        self.model_path = model_path;
        self
    }

    #[allow(dead_code)]
    pub fn set_microphone(&mut self, mic: Option<String>) -> &mut Self {
        self.microphone = mic;
        self
    }

    #[allow(dead_code)]
    pub fn set_language(&mut self, lang: &str) -> &mut Self {
        self.language = lang.to_string();
        self.french_typography = lang == "fr";
        self
    }

    #[allow(dead_code)]
    pub fn set_hotkey(&mut self, key: &str, ctrl: bool, alt: bool, shift: bool) -> &mut Self {
        self.hotkey.key = key.to_string();
        self.hotkey.ctrl = ctrl;
        self.hotkey.alt = alt;
        self.hotkey.shift = shift;
        self
    }

    #[allow(dead_code)]
    pub fn max_history_display(&self) -> String {
        format!("{} entrée{}", self.max_history, if self.max_history > 1 { "s" } else { "" })
    }

    #[allow(dead_code)]
    pub fn record_config_display(&self) -> String {
        format!("max={}s min={}ms silence={}", self.max_record_secs, self.min_record_ms, self.silence_level_label())
    }

    pub fn microphone_display(&self) -> String {
        self.microphone.clone().unwrap_or_else(|| "défaut système".to_string())
    }

    #[allow(dead_code)]
    pub fn audio_input_display(&self) -> String {
        format!("micro=[{}] rec=[{}] silence=[{}]",
            self.microphone_display(),
            self.record_duration_label(),
            self.silence_level_label()
        )
    }

    #[allow(dead_code)]
    pub fn inject_delay_display(&self) -> String {
        match self.inject_delay_ms {
            d if d < 50  => format!("{}ms (rapide)", d),
            d if d < 150 => format!("{}ms (normal)", d),
            d            => format!("{}ms (lent)", d),
        }
    }

    #[allow(dead_code)]
    pub fn defaults() -> Self {
        Config::default()
    }

    #[allow(dead_code)]
    pub fn export_to_file(&self, path: &std::path::Path) -> Result<()> {
        self.save_to(path)
    }

    #[allow(dead_code)]
    pub fn export_to_string(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Fusionne les champs non-default d'une autre config dans celle-ci.
    #[allow(dead_code)]
    pub fn merge_substitutions_from(&mut self, other: &Config) {
        for sub in &other.substitutions {
            if !self.substitutions.iter().any(|s| s.from == sub.from) {
                self.substitutions.push(sub.clone());
            }
        }
        log::debug!("Fusion substitutions : {} règles au total", self.substitutions.len());
    }

    #[allow(dead_code)]
    pub fn from_json_string(json: &str) -> Result<Self> {
        let mut config: Config = serde_json::from_str(json)?;
        config.sanitize();
        Ok(config)
    }

    #[allow(dead_code)]
    pub fn to_json_string(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn log_level_display(&self) -> &str {
        match self.log_level.as_str() {
            "error" => "🔴 error",
            "warn"  => "🟡 warn",
            "info"  => "🟢 info",
            "debug" => "🔵 debug",
            "trace" => "⚪ trace",
            other   => other,
        }
    }

    pub fn config_version_display(&self) -> String {
        format!("config v{} | app v{}", self.config_version, Self::app_version())
    }

    pub fn language_display(&self) -> String {
        match self.language.as_str() {
            "auto" => "Auto-détection".to_string(),
            "fr" => "Français".to_string(),
            "en" => "English".to_string(),
            "de" => "Deutsch".to_string(),
            "es" => "Español".to_string(),
            "it" => "Italiano".to_string(),
            "pt" => "Português".to_string(),
            "ja" => "日本語".to_string(),
            "zh" => "中文".to_string(),
            "ko" => "한국어".to_string(),
            other => other.to_string(),
        }
    }

    pub fn model_name(&self) -> &str {
        self.model_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
    }

    pub fn hotkey_string(&self) -> String {
        format!("{}{}{}{}",
            if self.hotkey.ctrl  { "Ctrl+" } else { "" },
            if self.hotkey.alt   { "Alt+"  } else { "" },
            if self.hotkey.shift { "Shift+"} else { "" },
            self.hotkey.key
        )
    }
}
