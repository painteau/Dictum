use std::sync::{Arc, Mutex};
use eframe::egui::{self, Color32, RichText, Vec2};
use crate::config::{Config, HotkeyConfig};
use crate::downloader::{self, Manifest, NvidiaInfo};
use anyhow::Result;

#[derive(Debug, Clone, PartialEq)]
enum Step {
    Welcome,
    ModelChoice,
    Language,
    Hotkey,
    Downloading,
    Done,
}

#[derive(Debug, Clone, PartialEq)]
enum ModelKey {
    Medium,
    Large,
}

impl ModelKey {
    fn as_str(&self) -> &'static str {
        match self {
            ModelKey::Medium => "medium",
            ModelKey::Large  => "large-v3",
        }
    }
}

#[derive(Default)]
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: u64,
    pub finished: bool,
    pub error: Option<String>,
    pub current_file: String,
}

impl DownloadProgress {
    pub fn percent(&self) -> f32 {
        if self.total == 0 { return 0.0; }
        (self.downloaded as f32 / self.total as f32).clamp(0.0, 1.0)
    }
    pub fn mb_display(&self) -> String {
        format!(
            "{:.0} MB / {:.0} MB",
            self.downloaded as f32 / 1_048_576.0,
            self.total as f32 / 1_048_576.0
        )
    }
}

struct SetupWizard {
    step: Step,
    nvidia: Option<NvidiaInfo>,
    manifest: Option<Manifest>,
    manifest_error: Option<String>,

    model_choice: ModelKey,
    language: String,
    hotkey_key: String,
    hotkey_ctrl: bool,
    hotkey_alt: bool,
    hotkey_shift: bool,

    progress: Arc<Mutex<DownloadProgress>>,
    config: Option<Config>,
}

impl SetupWizard {
    fn new() -> Self {
        let nvidia = downloader::detect_nvidia();
        let model_choice = if nvidia.as_ref().map(|g| g.capable).unwrap_or(false) {
            ModelKey::Large
        } else {
            ModelKey::Medium
        };

        Self {
            step: Step::Welcome,
            nvidia,
            manifest: None,
            manifest_error: None,
            model_choice,
            language: "fr".to_string(),
            hotkey_key: "F9".to_string(),
            hotkey_ctrl: false,
            hotkey_alt: false,
            hotkey_shift: false,
            progress: Arc::new(Mutex::new(DownloadProgress::default())),
            config: None,
        }
    }

    fn start_download(&mut self) {
        if !downloader::has_internet() {
            self.manifest_error = Some("Pas de connexion internet. Vérifier le réseau.".to_string());
            return;
        }

        let manifest = match &self.manifest {
            Some(m) => m.clone(),
            None => {
                self.manifest_error = Some("Manifest non chargé.".to_string());
                return;
            }
        };

        let key = self.model_choice.as_str().to_string();
        let entry = match manifest.models.get(&key) {
            Some(e) => e.clone(),
            None => {
                self.manifest_error = Some(format!("Modèle '{}' absent du manifest.", key));
                return;
            }
        };

        let dest = Config::models_dir().join(format!("ggml-{}.bin", key));
        let progress = self.progress.clone();

        let data_dir = Config::data_dir();
        std::thread::spawn(move || {
            // 1. Télécharger binaires whisper-cli + DLLs
            let bin_result = downloader::download_all_binaries(&manifest, &data_dir, |name, dl, total| {
                if let Ok(mut p) = progress.lock() {
                    p.current_file = name.to_string();
                    p.downloaded = dl;
                    p.total = total;
                }
            });
            if let Err(e) = bin_result {
                if let Ok(mut p) = progress.lock() { p.error = Some(e.to_string()); }
                return;
            }

            // 2. Télécharger le modèle
            let result = downloader::download_model(&entry, &dest, |dl, total| {
                if let Ok(mut p) = progress.lock() {
                    p.current_file = format!("ggml-{}.bin", key);
                    p.downloaded = dl;
                    p.total = total;
                }
            });

            if let Ok(mut p) = progress.lock() {
                match result {
                    Ok(_) => {
                        log::info!("Wizard : téléchargement modèle terminé");
                        p.finished = true;
                    }
                    Err(e) => {
                        log::error!("Wizard : erreur téléchargement — {e}");
                        p.error = Some(e.to_string());
                    }
                }
            }
        });

        self.step = Step::Downloading;
    }

    fn build_config(&self) -> Config {
        let key = self.model_choice.as_str();
        let model_path = Config::models_dir()
            .join(format!("ggml-{}.bin", key));

        Config {
            model_path,
            language: self.language.clone(),
            hotkey: HotkeyConfig {
                ctrl: self.hotkey_ctrl,
                alt: self.hotkey_alt,
                shift: self.hotkey_shift,
                key: self.hotkey_key.clone(),
            },
            french_typography: self.language == "fr",
            auto_capitalize: true,
            auto_enter: false,
            substitutions: vec![],
            microphone: None,
            config_version: 1,
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

    fn fetch_manifest_sync(&mut self) {
        match downloader::fetch_manifest() {
            Ok(m)  => { self.manifest = Some(m); self.manifest_error = None; }
            Err(e) => { self.manifest_error = Some(e.to_string()); }
        }
    }
}

impl eframe::App for SetupWizard {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Force repaint while downloading
        if self.step == Step::Downloading {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
        }

        let frame_style = egui::Frame::central_panel(&ctx.style())
            .fill(Color32::from_rgb(18, 18, 24))
            .inner_margin(egui::Margin::same(32.0));

        egui::CentralPanel::default().frame(frame_style).show(ctx, |ui| {
            ui.set_min_size(Vec2::new(520.0, 380.0));

            match self.step.clone() {
                Step::Welcome    => self.ui_welcome(ui),
                Step::ModelChoice => self.ui_model(ui),
                Step::Language   => self.ui_language(ui),
                Step::Hotkey     => self.ui_hotkey(ui),
                Step::Downloading => self.ui_downloading(ui),
                Step::Done       => {
                    let cfg = self.build_config();
                    if let Err(e) = cfg.save() {
                        log::error!("Impossible de sauvegarder la config : {e}");
                    }
                    self.config = Some(cfg);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        });
    }
}

impl SetupWizard {
    fn ui_welcome(&mut self, ui: &mut egui::Ui) {
        ui.add_space(24.0);
        ui.label(RichText::new("Dictum").size(36.0).color(Color32::WHITE).strong());
        ui.add_space(4.0);
        ui.label(RichText::new("Dictée vocale locale — zéro cloud, zéro abonnement.").size(14.0).color(Color32::GRAY));
        ui.add_space(32.0);

        if let Some(gpu) = &self.nvidia {
            let color = if gpu.capable { Color32::from_rgb(80, 200, 120) } else { Color32::from_rgb(200, 160, 80) };
            ui.label(RichText::new(format!("GPU détecté : {} ({} MB VRAM)", gpu.name, gpu.vram_mb)).color(color));
        } else {
            ui.label(RichText::new("Aucun GPU NVIDIA détecté — mode CPU").color(Color32::GRAY));
        }

        ui.add_space(32.0);
        ui.label(RichText::new("Configuration rapide : 2 questions.").color(Color32::LIGHT_GRAY));
        ui.add_space(24.0);

        if ui.add(styled_button("Commencer →")).clicked() {
            self.fetch_manifest_sync();
            self.step = Step::ModelChoice;
        }

        if let Some(err) = &self.manifest_error {
            ui.add_space(8.0);
            ui.label(RichText::new(format!("Erreur : {}", err)).color(Color32::RED).small());
            ui.label(RichText::new(format!("URL manifest : {}", downloader::MANIFEST_URL)).color(Color32::GRAY).small());
        }
    }

    fn ui_model(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Qualité de reconnaissance").size(22.0).color(Color32::WHITE).strong());
        ui.add_space(8.0);
        ui.label(RichText::new("Plus la qualité est haute, plus le traitement est lent.").color(Color32::GRAY));
        ui.add_space(24.0);

        let options = [
            (ModelKey::Medium, "Rapide", "1.5 GB", "Excellent français, rapide — recommandé"),
            (ModelKey::Large,  "Maximum", "3 GB",  "Meilleure précision — nécessite un bon CPU ou GPU"),
        ];

        for (key, label, size, desc) in &options {
            let selected = self.model_choice == *key;
            let border = if selected { Color32::from_rgb(100, 160, 255) } else { Color32::from_rgb(50, 50, 60) };

            egui::Frame::none()
                .fill(Color32::from_rgb(26, 26, 36))
                .stroke(egui::Stroke::new(if selected { 2.0 } else { 1.0 }, border))
                .rounding(8.0)
                .inner_margin(egui::Margin::same(12.0))
                .show(ui, |ui| {
                    ui.set_min_width(440.0);
                    if ui.selectable_label(selected, RichText::new(format!("{} — {}", label, size)).color(Color32::WHITE).strong()).clicked() {
                        self.model_choice = key.clone();
                    }
                    ui.label(RichText::new(*desc).color(Color32::GRAY).small());
                });

            ui.add_space(8.0);
        }

        // Auto-select notice
        if let Some(gpu) = &self.nvidia {
            let rec = if gpu.capable { "large-v3 (GPU capable)" } else { "medium (VRAM insuffisante)" };
            ui.label(RichText::new(format!("Recommandation GPU : {}", rec)).color(Color32::GRAY).small());
        }

        ui.add_space(16.0);
        if ui.add(styled_button("Suivant →")).clicked() {
            self.step = Step::Language;
        }
    }

    fn ui_language(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Langue principale").size(22.0).color(Color32::WHITE).strong());
        ui.add_space(8.0);
        ui.label(RichText::new("Whisper supporte 99 langues. Ce réglage optimise la détection.").color(Color32::GRAY));
        ui.add_space(24.0);

        let langs = [
            ("fr",   "Français"),
            ("en",   "English"),
            ("auto", "Détection automatique"),
        ];

        for (code, name) in &langs {
            let selected = self.language == *code;
            if ui.selectable_label(selected, RichText::new(*name).color(Color32::WHITE)).clicked() {
                self.language = code.to_string();
            }
        }

        ui.add_space(24.0);
        if ui.add(styled_button("Suivant →")).clicked() {
            self.step = Step::Hotkey;
        }
    }

    fn ui_hotkey(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Touche de dictée").size(22.0).color(Color32::WHITE).strong());
        ui.add_space(8.0);
        ui.label(RichText::new("Maintenez cette touche pour dicter, relâchez pour envoyer.").color(Color32::GRAY));
        ui.add_space(24.0);

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.hotkey_ctrl,  "Ctrl");
            ui.checkbox(&mut self.hotkey_alt,   "Alt");
            ui.checkbox(&mut self.hotkey_shift, "Shift");
        });

        ui.add_space(8.0);

        let keys = ["F1","F2","F3","F4","F5","F6","F7","F8","F9","F10","F11","F12",
                    "Space","Insert","Home","End","PageUp","PageDown",
                    "ScrollLock","Pause","NumLock",
                    "KpMinus","KpPlus","KpMultiply","KpDivide","KpReturn",
                    "Num0","Num1","Num2","Num3","Num4","Num5","Num6","Num7","Num8","Num9"];
        ui.horizontal_wrapped(|ui| {
            for k in &keys {
                let selected = self.hotkey_key == *k;
                if ui.selectable_label(selected, RichText::new(*k).monospace()).clicked() {
                    self.hotkey_key = k.to_string();
                }
            }
        });

        ui.add_space(8.0);
        let combo = format!(
            "{}{}{}{}",
            if self.hotkey_ctrl  { "Ctrl+" } else { "" },
            if self.hotkey_alt   { "Alt+"  } else { "" },
            if self.hotkey_shift { "Shift+"} else { "" },
            self.hotkey_key
        );
        ui.label(RichText::new(format!("Raccourci : {}", combo)).color(Color32::from_rgb(100, 160, 255)));

        ui.add_space(24.0);
        if ui.add(styled_button("Télécharger le modèle →")).clicked() {
            self.start_download();
        }
    }

    fn ui_downloading(&mut self, ui: &mut egui::Ui) {
        let (pct, label, finished, error) = {
            let p = self.progress.lock().unwrap();
            (p.percent(), p.mb_display(), p.finished, p.error.clone())
        };

        let title = {
            let p = self.progress.lock().unwrap();
            if p.current_file.contains("ggml") {
                "Téléchargement du modèle Whisper"
            } else if p.current_file.is_empty() {
                "Préparation..."
            } else {
                "Téléchargement des outils"
            }
        };
        ui.label(RichText::new(title).size(22.0).color(Color32::WHITE).strong());
        ui.add_space(24.0);

        let file = self.progress.lock().unwrap().current_file.clone();
        if !file.is_empty() {
            ui.label(RichText::new(format!("Fichier : {}", file)).color(Color32::GRAY).small());
        }
        ui.label(RichText::new(&label).color(Color32::LIGHT_GRAY));
        ui.add_space(8.0);

        let bar = egui::ProgressBar::new(pct)
            .show_percentage()
            .animate(true);
        ui.add(bar);

        ui.add_space(16.0);

        if let Some(err) = error {
            ui.label(RichText::new(format!("Erreur : {}", err)).color(Color32::RED));
            if ui.add(styled_button("Réessayer")).clicked() {
                *self.progress.lock().unwrap() = DownloadProgress::default();
                self.start_download();
            }
        } else if finished {
            ui.label(RichText::new("Modèle téléchargé et vérifié.").color(Color32::from_rgb(80, 200, 120)));
            ui.add_space(16.0);
            if ui.add(styled_button("Terminer")).clicked() {
                self.step = Step::Done;
            }
        } else {
            ui.label(RichText::new("Ne pas fermer cette fenêtre...").color(Color32::GRAY).small());
        }
    }
}

fn styled_button(text: &str) -> egui::Button<'_> {
    egui::Button::new(RichText::new(text).color(Color32::WHITE).strong())
        .fill(Color32::from_rgb(70, 120, 200))
        .min_size(Vec2::new(160.0, 36.0))
}

// ── Point d'entrée public ─────────────────────────────────────────────────────

pub fn needs_setup(config: &Config) -> bool {
    let model_missing = !config.is_model_ready();
    let cli_missing = !Config::is_whisper_cli_ready();
    if model_missing { log::info!("Setup requis : modèle absent"); }
    if cli_missing { log::info!("Setup requis : whisper-cli.exe absent"); }
    model_missing || cli_missing
}

/// Lance le wizard et retourne la config finale. Bloque jusqu'à fermeture.
pub fn run_wizard() -> Result<Config> {
    let wizard = SetupWizard::new();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Dictum — Configuration")
            .with_inner_size([540.0, 420.0])
            .with_resizable(false)
            .with_maximize_button(false),
        ..Default::default()
    };

    // We can't easily pass the result out of run_native closure in egui 0.27,
    // so we write config to file inside the wizard and reload it here.
    eframe::run_native(
        "Dictum Setup",
        options,
        Box::new(|_cc| Box::new(wizard)),
    )
    .map_err(|e| anyhow::anyhow!("Wizard échoué : {e}"))?;

    // After wizard exits, config file was saved — reload it
    Config::load()
}
