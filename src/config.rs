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
        log::info!("Config [{}|{}] | {} | flags=[{}]", self.profile_name(), self.performance_label(), self.changes_summary_display(), flags_str);
        log::info!("Détails : lang={} model=[{}|{}] hotkey={} threads={} rec=[{}] silence={} inject=[{}]",
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
            format!("Dictum {} prêt [score:{}/100 — {}] — {}",
                Self::app_version(), self.score(), self.score_label(), self.model_name())
        } else {
            format!("score:{}/100 — {} problème(s) : {}", self.score(), issues.len(), issues.join(", "))
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
            "Dictum v{} | Modèle: {} [{}|~{}MB|{}] | Langue: {} | Hotkey: {}\nWhisper: threads={} temp={:.1} | Enregistrement: {} | Silence: {}\nInjection: [{}] | Beep: {}",
            Self::app_version(),
            self.model_name(), if self.is_model_ready() { "✓" } else { "✗" },
            self.model_size_mb_estimate(), self.performance_label(),
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
    pub fn with_model_name(&mut self, name: &str) -> &mut Self {
        let filename = if name.ends_with(".bin") {
            name.to_string()
        } else {
            format!("ggml-{}.bin", name)
        };
        self.model_path = Self::models_dir().join(filename);
        self
    }

    #[allow(dead_code)]
    pub fn set_beep_freqs(&mut self, start: u32, end: u32, duration_ms: u32) -> &mut Self {
        self.beep_start_freq = start.clamp(100, 20000);
        self.beep_end_freq = end.clamp(100, 20000);
        self.beep_duration_ms = duration_ms.clamp(1, 2000);
        self
    }

    #[allow(dead_code)]
    pub fn set_prefix_space(&mut self, val: bool) -> &mut Self {
        self.prefix_space = val;
        self
    }

    #[allow(dead_code)]
    pub fn set_whisper_temperature(&mut self, temp: f32) -> &mut Self {
        self.whisper_temperature = temp.clamp(0.0, 1.0);
        self
    }

    #[allow(dead_code)]
    pub fn set_no_speech(&mut self, val: bool) -> &mut Self {
        self.whisper_no_speech = val;
        self
    }

    #[allow(dead_code)]
    pub fn set_log_level(&mut self, level: &str) -> &mut Self {
        if ["error","warn","info","debug","trace"].contains(&level) {
            self.log_level = level.to_string();
        } else {
            log::warn!("Niveau log '{}' invalide, ignoré", level);
        }
        self
    }

    #[allow(dead_code)]
    pub fn set_inject_delay(&mut self, ms: u64) -> &mut Self {
        self.inject_delay_ms = ms.min(1000);
        self
    }

    #[allow(dead_code)]
    pub fn set_auto_capitalize(&mut self, val: bool) -> &mut Self {
        self.auto_capitalize = val;
        self
    }

    #[allow(dead_code)]
    pub fn set_auto_enter(&mut self, val: bool) -> &mut Self {
        self.auto_enter = val;
        self
    }

    #[allow(dead_code)]
    pub fn set_french_typography(&mut self, val: bool) -> &mut Self {
        self.french_typography = val;
        self
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

    pub fn is_using_cuda(&self) -> bool { false }
    pub fn needs_wizard(&self) -> bool { !self.is_fully_ready() }

    pub fn changes_summary_display(&self) -> String {
        let changes = self.changes_from_default();
        if changes.is_empty() {
            "configuration par défaut".to_string()
        } else {
            format!("{} modification(s) : {}", changes.len(), changes.join(", "))
        }
    }

    pub fn changes_from_default(&self) -> Vec<String> {
        let default = Config::default();
        let mut changes = Vec::new();
        if self.language != default.language { changes.push(format!("langue: {} → {}", default.language, self.language)); }
        if self.beep_enabled != default.beep_enabled { changes.push(format!("beep: {} → {}", default.beep_enabled, self.beep_enabled)); }
        if self.pause_media != default.pause_media { changes.push(format!("pause_media: {} → {}", default.pause_media, self.pause_media)); }
        if self.silence_threshold != default.silence_threshold { changes.push(format!("silence: {:.3} → {:.3}", default.silence_threshold, self.silence_threshold)); }
        if self.max_record_secs != default.max_record_secs { changes.push(format!("max_rec: {}s → {}s", default.max_record_secs, self.max_record_secs)); }
        if !self.substitutions.is_empty() { changes.push(format!("substitutions: {} règle(s)", self.substitutions.len())); }
        changes
    }

    #[allow(dead_code)]
    pub fn apply_profile_french_standard(&mut self) -> &mut Self {
        self.language = "fr".to_string();
        self.french_typography = true;
        self.auto_capitalize = true;
        self.beep_enabled = true;
        self.silence_threshold = 0.005;
        self
    }

    #[allow(dead_code)]
    pub fn apply_profile_minimal(&mut self) -> &mut Self {
        self.french_typography = false;
        self.auto_capitalize = false;
        self.beep_enabled = false;
        self.pause_media = false;
        self
    }

    #[allow(dead_code)]
    pub fn apply_profile_performance(&mut self) -> &mut Self {
        self.whisper_threads = 0; // auto
        self.whisper_temperature = 0.0;
        self.silence_threshold = 0.003;
        self.min_record_ms = 200;
        self
    }

    #[allow(dead_code)]
    pub fn apply_profile_quiet(&mut self) -> &mut Self {
        self.beep_enabled = false;
        self.pause_media = false;
        self.auto_enter = false;
        self.prefix_space = false;
        self
    }

    #[allow(dead_code)]
    pub fn apply_profile_english(&mut self) -> &mut Self {
        self.language = "en".to_string();
        self.french_typography = false;
        self.auto_capitalize = true;
        self
    }

    #[allow(dead_code)]
    pub fn apply_profile_dictaphone(&mut self) -> &mut Self {
        self.max_record_secs = 120;
        self.auto_enter = false;
        self.prefix_space = false;
        self.auto_capitalize = true;
        self.beep_enabled = true;
        self
    }

    pub fn diagnose(&self) -> String {
        let issues = self.validate();
        let ok = "✓".to_string();
        let nok = "✗".to_string();
        format!(
            "=== Dictum {} Diagnostic — Score: {}/100 ({}) ===\nModèle   : {} {}\nCLI      : {} {}\nConfig   : {} v{}\nProfil   : {}\nThreads  : {}\nSilence  : {}\nBeep     : {}\n{}\n===",
            Self::app_version(), self.score(), self.score_label(),
            if self.is_model_ready() { &ok } else { &nok }, self.model_name(),
            if Self::is_whisper_cli_ready() { &ok } else { &nok }, "whisper-cli.exe",
            ok, self.config_version,
            self.profile_name(),
            self.threads_display(),
            self.silence_level_label(),
            self.beep_description(),
            if issues.is_empty() { "✓ Tout est OK".to_string() } else { format!("⚠ {} problème(s) :\n{}", issues.len(), issues.iter().map(|i| format!("  • {i}")).collect::<Vec<_>>().join("\n")) }
        )
    }
    pub fn can_transcribe(&self) -> bool { self.is_fully_ready() }
    pub fn is_configured_for_french_dictation(&self) -> bool {
        self.is_french() && self.french_typography && self.auto_capitalize
    }

    pub fn min_disk_space_mb(&self) -> u32 {
        self.model_size_mb_estimate() + 100
    }

    pub fn config_age_display(&self) -> String {
        format!("config_version={}", self.config_version)
    }

    pub fn is_compatible_with_version(&self, min_version: u32) -> bool {
        self.config_version >= min_version
    }

    pub fn total_config_fields() -> usize { 23 }

    #[allow(dead_code)]
    pub fn copy_with_model(&self, model_name: &str) -> Self {
        let mut c = self.clone();
        c.with_model_name(model_name);
        c
    }

    #[allow(dead_code)]
    pub fn diff_fields(&self, other: &Config) -> usize {
        let mut diffs = 0;
        if self.language != other.language { diffs += 1; }
        if self.model_path != other.model_path { diffs += 1; }
        if self.hotkey.key != other.hotkey.key { diffs += 1; }
        if self.silence_threshold != other.silence_threshold { diffs += 1; }
        if self.whisper_threads != other.whisper_threads { diffs += 1; }
        if self.max_record_secs != other.max_record_secs { diffs += 1; }
        diffs
    }

    pub fn required_fields_present(&self) -> bool {
        !self.hotkey.key.is_empty() && !self.language.is_empty()
    }

    pub fn optional_fields_count(&self) -> usize {
        let mut count = 0;
        if self.microphone.is_some() { count += 1; }
        if !self.substitutions.is_empty() { count += self.substitutions.len(); }
        count
    }

    pub fn recommend_model(has_fast_cpu: bool, vram_mb: u32) -> &'static str {
        if vram_mb >= 4096 { "large-v3" }
        else if has_fast_cpu { "medium" }
        else { "medium" }
    }

    pub fn recommend_threads(cpu_cores: usize) -> u32 {
        (cpu_cores.min(8) as u32).max(2)
    }

    pub fn model_size_mb_estimate(&self) -> u32 {
        let name = self.model_name();
        if name.contains("large") { 3000 }
        else if name.contains("medium") { 1500 }
        else if name.contains("small") { 500 }
        else if name.contains("base") { 150 }
        else if name.contains("tiny") { 75 }
        else { 1500 }
    }

    pub fn performance_label(&self) -> &'static str {
        if self.is_optimized_for_speed() { "Rapide" }
        else if self.is_optimized_for_quality() { "Haute qualité" }
        else if self.uses_large_model() { "Qualité" }
        else { "Standard" }
    }

    pub fn is_optimized_for_speed(&self) -> bool {
        self.uses_medium_model() && self.effective_threads() >= 4 && self.inject_delay_ms <= 80
    }
    pub fn is_optimized_for_quality(&self) -> bool {
        self.uses_large_model() && self.whisper_temperature == 0.0
    }
    pub fn estimated_transcription_time(&self, audio_secs: f32) -> f32 {
        let factor: f32 = if self.uses_large_model() { 3.0 } else { 1.5 };
        let thread_factor = (self.effective_threads() as f32 / 4.0).min(2.0);
        audio_secs * factor / thread_factor
    }
    pub fn recording_is_limited(&self) -> bool { self.max_record_secs < 60 }
    pub fn silence_detection_active(&self) -> bool { self.silence_threshold > 0.001 }
    pub fn has_min_record_limit(&self) -> bool { self.min_record_ms > 0 }
    pub fn is_history_full(&self, current_count: usize) -> bool { current_count >= self.max_history }
    pub fn history_capacity_remaining(&self, current_count: usize) -> usize {
        self.max_history.saturating_sub(current_count)
    }
    pub fn effective_threads(&self) -> usize {
        if self.whisper_threads == 0 {
            std::thread::available_parallelism().map(|n| n.get().min(8)).unwrap_or(4)
        } else {
            self.whisper_threads as usize
        }
    }
    pub fn effective_timeout(&self, audio_secs: u64) -> u64 {
        (audio_secs * 10).max(60).min(300)
    }
    pub fn substitution_index(&self, from: &str) -> Option<usize> {
        self.substitutions.iter().position(|s| s.from == from)
    }
    pub fn uses_large_model(&self) -> bool { self.model_name().contains("large") }
    pub fn uses_medium_model(&self) -> bool { self.model_name().contains("medium") }
    pub fn is_low_latency(&self) -> bool {
        self.inject_delay_ms <= 60 && self.min_record_ms <= 200
    }
    pub fn is_verbose_mode(&self) -> bool { self.log_level == "debug" || self.log_level == "trace" }
    pub fn is_production_ready(&self) -> bool {
        self.is_fully_ready() && self.validate().is_empty()
    }
    pub fn has_whisper_optimizations(&self) -> bool {
        self.whisper_threads > 0 || self.whisper_temperature < 0.1
    }
    pub fn is_using_default_microphone(&self) -> bool { self.microphone.is_none() }
    pub fn has_custom_model_path(&self) -> bool {
        self.model_path != Self::default().model_path
    }
    pub fn effective_language(&self) -> &str {
        if self.language == "auto" { "auto-detect" } else { &self.language }
    }

    pub fn score(&self) -> u8 {
        let mut s = 0u8;
        if self.is_model_ready() { s += 30; }
        if Self::is_whisper_cli_ready() { s += 20; }
        if self.validate().is_empty() { s += 20; }
        if self.has_substitutions() { s += 10; }
        if self.language != "auto" { s += 5; }
        if self.beep_enabled { s += 5; }
        if self.whisper_threads > 0 || self.whisper_temperature < 0.1 { s += 10; }
        s
    }

    pub fn score_breakdown(&self) -> Vec<(&'static str, u8)> {
        vec![
            ("Modèle présent", if self.is_model_ready() { 30 } else { 0 }),
            ("whisper-cli présent", if Self::is_whisper_cli_ready() { 20 } else { 0 }),
            ("Config valide", if self.validate().is_empty() { 20 } else { 0 }),
            ("Substitutions configurées", if self.has_substitutions() { 10 } else { 0 }),
            ("Langue explicite", if self.language != "auto" { 5 } else { 0 }),
            ("Beep activé", if self.beep_enabled { 5 } else { 0 }),
            ("Whisper optimisé", if self.whisper_threads > 0 || self.whisper_temperature < 0.1 { 10 } else { 0 }),
        ]
    }

    pub fn score_breakdown_display(&self) -> String {
        self.score_breakdown().iter()
            .map(|(label, pts)| format!("  {} : {}/{}pts", if *pts > 0 { "✓" } else { "✗" }, pts, pts.max(&1)))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn score_label(&self) -> &'static str {
        match self.score() {
            90..=100 => "Excellent",
            70..=89  => "Très bon",
            50..=69  => "Bon",
            30..=49  => "Basique",
            _        => "Incomplet",
        }
    }
    pub fn is_french(&self) -> bool { self.language == "fr" }
    pub fn is_auto_detect(&self) -> bool { self.language == "auto" }
    pub fn is_beep_enabled(&self) -> bool { self.beep_enabled }
    pub fn is_pause_media_enabled(&self) -> bool { self.pause_media }
    pub fn is_prefix_space_enabled(&self) -> bool { self.prefix_space }
    pub fn is_auto_enter_enabled(&self) -> bool { self.auto_enter }
    pub fn is_french_typography_enabled(&self) -> bool { self.french_typography }
    pub fn is_auto_capitalize_enabled(&self) -> bool { self.auto_capitalize }
    pub fn is_no_speech_enabled(&self) -> bool { self.whisper_no_speech }
    pub fn is_debug_mode(&self) -> bool { self.log_level == "debug" || self.log_level == "trace" }

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

    #[allow(dead_code)]
    pub fn from_json_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }

    #[allow(dead_code)]
    pub fn substitutions_count(&self) -> usize {
        self.substitutions.len()
    }

    #[allow(dead_code)]
    pub fn record_max_secs_display(&self) -> String {
        if self.max_record_secs >= 60 {
            format!("{}min{}s", self.max_record_secs / 60, self.max_record_secs % 60)
        } else {
            format!("{}s", self.max_record_secs)
        }
    }

    #[allow(dead_code)]
    pub fn is_hotkey_with_modifier(&self) -> bool {
        self.hotkey.ctrl || self.hotkey.alt || self.hotkey.shift
    }

    #[allow(dead_code)]
    pub fn hotkey_key(&self) -> &str {
        &self.hotkey.key
    }

    #[allow(dead_code)]
    pub fn remove_substitution_by_from(&mut self, from: &str) -> bool {
        let before = self.substitutions.len();
        self.substitutions.retain(|s| s.from != from);
        self.substitutions.len() < before
    }

    #[allow(dead_code)]
    pub fn find_substitution(&self, from: &str) -> Option<&Substitution> {
        self.substitutions.iter().find(|s| s.from == from)
    }

    #[allow(dead_code)]
    pub fn min_record_ms_display(&self) -> String {
        format!("{}ms", self.min_record_ms)
    }

    #[allow(dead_code)]
    pub fn silence_threshold_display(&self) -> String {
        format!("{:.4}", self.silence_threshold)
    }

    #[allow(dead_code)]
    pub fn is_low_silence_threshold(&self) -> bool {
        self.silence_threshold < 0.003
    }

    #[allow(dead_code)]
    pub fn is_high_silence_threshold(&self) -> bool {
        self.silence_threshold > 0.02
    }

    #[allow(dead_code)]
    pub fn config_summary_oneliner(&self) -> String {
        format!("model={} lang={} hotkey={} beep={} typo_fr={}",
            self.model_name(), self.language, self.hotkey_string(),
            self.beep_enabled, self.french_typography)
    }

    #[allow(dead_code)]
    pub fn is_valid_language_code(&self) -> bool {
        const VALID: &[&str] = &[
            "af","ar","hy","az","be","bs","bg","ca","zh","hr","cs","da","nl","en",
            "et","fi","fr","gl","de","el","he","hi","hu","is","id","it","ja","kn",
            "kk","ko","lv","lt","mk","ms","mr","mi","ne","no","fa","pl","pt","ro",
            "ru","sr","sk","sl","es","sw","sv","tl","ta","th","tr","uk","ur","vi","cy",
            "auto",
        ];
        VALID.contains(&self.language.as_str())
    }

    #[allow(dead_code)]
    pub fn hotkey_modifier_count(&self) -> u8 {
        self.hotkey.ctrl as u8 + self.hotkey.alt as u8 + self.hotkey.shift as u8
    }

    #[allow(dead_code)]
    pub fn is_language(&self, code: &str) -> bool {
        self.language == code
    }

    #[allow(dead_code)]
    pub fn set_language_auto(&mut self) -> &mut Self {
        self.language = "auto".to_string();
        self.french_typography = false;
        self
    }

    #[allow(dead_code)]
    pub fn history_limit_display(&self) -> String {
        format!("{} entrées max", self.max_history)
    }

    #[allow(dead_code)]
    pub fn is_history_large(&self) -> bool {
        self.max_history >= 50
    }

    #[allow(dead_code)]
    pub fn model_exists_locally(&self) -> bool {
        self.model_path.exists()
    }

    #[allow(dead_code)]
    pub fn whisper_cli_exists(&self) -> bool {
        Self::data_dir().join("whisper-cli.exe").exists()
    }

    #[allow(dead_code)]
    pub fn data_dir_size_mb(&self) -> f64 {
        let dir = Self::data_dir();
        let size: u64 = std::fs::read_dir(&dir)
            .ok()
            .map(|entries| entries.flatten()
                .filter_map(|e| e.metadata().ok())
                .map(|m| m.len())
                .sum())
            .unwrap_or(0);
        size as f64 / 1_048_576.0
    }

    #[allow(dead_code)]
    pub fn model_size_mb(&self) -> f64 {
        std::fs::metadata(&self.model_path)
            .map(|m| m.len() as f64 / 1_048_576.0)
            .unwrap_or(0.0)
    }

    #[allow(dead_code)]
    pub fn inject_mode_is_keyboard(&self) -> bool { true }

    #[allow(dead_code)]
    pub fn is_windows(&self) -> bool { cfg!(windows) }

    #[allow(dead_code)]
    pub fn app_name() -> &'static str { "Dictum" }

    #[allow(dead_code)]
    pub fn is_english(&self) -> bool { self.language == "en" }

    #[allow(dead_code)]
    pub fn is_german(&self) -> bool { self.language == "de" }

    #[allow(dead_code)]
    pub fn is_spanish(&self) -> bool { self.language == "es" }

    #[allow(dead_code)]
    pub fn is_italian(&self) -> bool { self.language == "it" }

    #[allow(dead_code)]
    pub fn is_portuguese(&self) -> bool { self.language == "pt" }

    #[allow(dead_code)]
    pub fn is_chinese(&self) -> bool { self.language == "zh" }

    #[allow(dead_code)]
    pub fn is_japanese(&self) -> bool { self.language == "ja" }

    #[allow(dead_code)]
    pub fn is_arabic(&self) -> bool { self.language == "ar" }

    #[allow(dead_code)]
    pub fn is_russian(&self) -> bool { self.language == "ru" }

    #[allow(dead_code)]
    pub fn hotkey_is_function_key(&self) -> bool {
        self.hotkey.key.starts_with('F') && self.hotkey.key[1..].parse::<u8>().is_ok()
    }

    #[allow(dead_code)]
    pub fn full_description(&self) -> String {
        format!("{} | {} | score {}/100", self.description(), self.hotkey_string(), self.score())
    }

    #[allow(dead_code)]
    pub fn is_beep_disabled(&self) -> bool { !self.beep_enabled }

    #[allow(dead_code)]
    pub fn is_auto_enter_disabled(&self) -> bool { !self.auto_enter }

    #[allow(dead_code)]
    pub fn models_dir_display(&self) -> String {
        Self::models_dir().display().to_string()
    }

    #[allow(dead_code)]
    pub fn log_path_display(&self) -> String {
        Self::log_path().display().to_string()
    }

    #[allow(dead_code)]
    pub fn with_language(mut self, lang: &str) -> Self {
        self.language = lang.to_string();
        self.french_typography = lang == "fr";
        self
    }

    #[allow(dead_code)]
    pub fn with_beep(mut self, enabled: bool) -> Self {
        self.beep_enabled = enabled;
        self
    }

    #[allow(dead_code)]
    pub fn with_capitalize(mut self, enabled: bool) -> Self {
        self.auto_capitalize = enabled;
        self
    }

    #[allow(dead_code)]
    pub fn with_prefix_space(mut self, enabled: bool) -> Self {
        self.prefix_space = enabled;
        self
    }

    #[allow(dead_code)]
    pub fn with_auto_enter(mut self, enabled: bool) -> Self {
        self.auto_enter = enabled;
        self
    }

    #[allow(dead_code)]
    pub fn with_pause_media(mut self, enabled: bool) -> Self {
        self.pause_media = enabled;
        self
    }

    #[allow(dead_code)]
    pub fn with_silence_threshold(mut self, v: f32) -> Self {
        self.silence_threshold = v.clamp(0.0, 1.0);
        self
    }

    #[allow(dead_code)]
    pub fn with_max_history(mut self, n: usize) -> Self {
        self.max_history = n.max(1).min(1000);
        self
    }

    #[allow(dead_code)]
    pub fn with_hotkey_key(mut self, key: &str) -> Self {
        self.hotkey.key = key.to_string();
        self
    }

    #[allow(dead_code)]
    pub fn with_microphone(mut self, mic: Option<String>) -> Self {
        self.microphone = mic;
        self
    }

    #[allow(dead_code)]
    pub fn with_french_typography(mut self, enabled: bool) -> Self {
        self.french_typography = enabled;
        self
    }

    #[allow(dead_code)]
    pub fn with_whisper_threads(mut self, n: u32) -> Self {
        self.whisper_threads = n;
        self
    }

    #[allow(dead_code)]
    pub fn with_log_level(mut self, level: &str) -> Self {
        self.log_level = level.to_string();
        self
    }

    #[allow(dead_code)]
    pub fn with_inject_delay(mut self, ms: u64) -> Self {
        self.inject_delay_ms = ms;
        self
    }

    #[allow(dead_code)]
    pub fn with_min_record_ms(mut self, ms: u64) -> Self {
        self.min_record_ms = ms;
        self
    }

    #[allow(dead_code)]
    pub fn has_long_record_limit(&self) -> bool {
        self.max_record_secs > 60
    }

    #[allow(dead_code)]
    pub fn has_short_record_limit(&self) -> bool {
        self.max_record_secs <= 10
    }

    #[allow(dead_code)]
    pub fn is_whisper_temperature_zero(&self) -> bool {
        self.whisper_temperature == 0.0
    }

    #[allow(dead_code)]
    pub fn inject_delay_is_fast(&self) -> bool {
        self.inject_delay_ms <= 50
    }

    #[allow(dead_code)]
    pub fn inject_delay_category(&self) -> &'static str {
        if self.inject_delay_ms <= 50 { "rapide" }
        else if self.inject_delay_ms <= 150 { "normal" }
        else { "lent" }
    }

    #[allow(dead_code)]
    pub fn silence_category(&self) -> &'static str {
        if self.silence_threshold < 0.002 { "très sensible" }
        else if self.silence_threshold < 0.01 { "sensible" }
        else if self.silence_threshold < 0.05 { "normal" }
        else { "sourd" }
    }

    #[allow(dead_code)]
    pub fn is_privacy_mode(&self) -> bool {
        !self.beep_enabled && !self.pause_media
    }

    #[allow(dead_code)]
    pub fn total_feature_flags(&self) -> usize {
        [self.beep_enabled, self.auto_capitalize, self.french_typography,
         self.auto_enter, self.prefix_space, self.pause_media,
         self.whisper_no_speech, self.microphone.is_some()]
            .iter().filter(|&&b| b).count()
    }

    #[allow(dead_code)]
    pub fn is_minimal_config(&self) -> bool {
        !self.beep_enabled && !self.french_typography && !self.auto_capitalize && !self.auto_enter
    }

    #[allow(dead_code)]
    pub fn is_maximal_config(&self) -> bool {
        self.beep_enabled && self.french_typography && self.auto_capitalize && self.pause_media
    }

    #[allow(dead_code)]
    pub fn compatible_with_french(&self) -> bool {
        self.is_french() && self.french_typography && self.auto_capitalize
    }

    #[allow(dead_code)]
    pub fn thread_count_display(&self) -> String {
        if self.whisper_threads == 0 {
            "auto".to_string()
        } else {
            format!("{} threads", self.whisper_threads)
        }
    }

    #[allow(dead_code)]
    pub fn summary_for_about(&self) -> String {
        format!(
            "v{} | {} | {}/100",
            Self::app_version(), self.hotkey_string(), self.score()
        )
    }

    #[allow(dead_code)]
    pub fn is_same_language(&self, other: &Config) -> bool {
        self.language == other.language
    }

    #[allow(dead_code)]
    pub fn is_same_hotkey(&self, other: &Config) -> bool {
        self.hotkey.key == other.hotkey.key
            && self.hotkey.ctrl == other.hotkey.ctrl
            && self.hotkey.alt == other.hotkey.alt
            && self.hotkey.shift == other.hotkey.shift
    }

    #[allow(dead_code)]
    pub fn log_level_is_debug(&self) -> bool { self.log_level == "debug" }

    #[allow(dead_code)]
    pub fn log_level_is_info(&self) -> bool { self.log_level == "info" }

    #[allow(dead_code)]
    pub fn log_level_is_warn(&self) -> bool { self.log_level == "warn" }

    #[allow(dead_code)]
    pub fn set_log_debug(&mut self) -> &mut Self { self.log_level = "debug".into(); self }

    #[allow(dead_code)]
    pub fn set_log_info(&mut self) -> &mut Self { self.log_level = "info".into(); self }

    #[allow(dead_code)]
    pub fn beep_start_freq_display(&self) -> String {
        format!("{}Hz", self.beep_start_freq)
    }

    #[allow(dead_code)]
    pub fn beep_end_freq_display(&self) -> String {
        format!("{}Hz", self.beep_end_freq)
    }

    #[allow(dead_code)]
    pub fn beep_duration_display(&self) -> String {
        format!("{}ms", self.beep_duration_ms)
    }

    #[allow(dead_code)]
    pub fn is_production_config(&self) -> bool {
        self.is_fully_ready() && self.score() >= 80
    }

    #[allow(dead_code)]
    pub fn export_json_to(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn max_record_minutes(&self) -> f32 {
        self.max_record_secs as f32 / 60.0
    }

    #[allow(dead_code)]
    pub fn whisper_temperature_display(&self) -> String {
        format!("{:.2}", self.whisper_temperature)
    }

    #[allow(dead_code)]
    pub fn is_hotkey_ctrl_only(&self) -> bool {
        self.hotkey.ctrl && !self.hotkey.alt && !self.hotkey.shift
    }

    #[allow(dead_code)]
    pub fn is_hotkey_alt_only(&self) -> bool {
        !self.hotkey.ctrl && self.hotkey.alt && !self.hotkey.shift
    }

    #[allow(dead_code)]
    pub fn is_hotkey_no_modifier(&self) -> bool {
        !self.hotkey.ctrl && !self.hotkey.alt && !self.hotkey.shift
    }

    #[allow(dead_code)]
    pub fn clone_with_language(&self, lang: &str) -> Self {
        let mut c = self.clone();
        c.language = lang.to_string();
        c.french_typography = lang == "fr";
        c
    }

    #[allow(dead_code)]
    pub fn clone_with_model(&self, path: std::path::PathBuf) -> Self {
        let mut c = self.clone();
        c.model_path = path;
        c
    }

    #[allow(dead_code)]
    pub fn language_emoji(&self) -> &'static str {
        match self.language.as_str() {
            "fr" => "🇫🇷", "en" => "🇬🇧", "de" => "🇩🇪",
            "es" => "🇪🇸", "it" => "🇮🇹", "pt" => "🇵🇹",
            "ja" => "🇯🇵", "zh" => "🇨🇳", "ru" => "🇷🇺",
            "ar" => "🇸🇦", "auto" => "🌐", _ => "🌍",
        }
    }

    #[allow(dead_code)]
    pub fn has_high_thread_count(&self) -> bool {
        self.whisper_threads > 8
    }

    #[allow(dead_code)]
    pub fn has_custom_beep(&self) -> bool {
        self.beep_start_freq != 800 || self.beep_end_freq != 600
    }

    #[allow(dead_code)]
    pub fn record_limit_category(&self) -> &'static str {
        match self.max_record_secs {
            0..=10 => "très court",
            11..=30 => "court",
            31..=120 => "normal",
            _ => "long",
        }
    }

    #[allow(dead_code)]
    pub fn is_same_model(&self, other: &Config) -> bool {
        self.model_path == other.model_path
    }
}
