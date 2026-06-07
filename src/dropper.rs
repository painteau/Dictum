/// Fenêtre drag & drop pour transcrire des fichiers audio.
/// Glisser un fichier WAV/MP3/M4A → transcription → résultat affiché + copié.

use eframe::egui::{self, Color32, RichText, Vec2};
use crate::config::Config;

struct DropWindow {
    cfg: Config,
    result: Option<String>,
    error: Option<String>,
    processing: bool,
    dropped_file: Option<std::path::PathBuf>,
}

impl DropWindow {
    fn new(cfg: Config) -> Self {
        Self {
            cfg,
            result: None,
            error: None,
            processing: false,
            dropped_file: None,
        }
    }

    fn process_file(&mut self, path: std::path::PathBuf) {
        self.result = None;
        self.error = None;
        self.processing = true;
        self.dropped_file = Some(path.clone());

        let cfg = self.cfg.clone();

        // On lance la transcription dans un thread, résultat via channel
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let result = crate::read_audio_file(&path)
                .and_then(|samples| crate::transcribe::transcribe(&samples, &cfg));
            let _ = tx.send(result);
        });

        // Stocker le receiver pour le poll dans update()
        // Pour simplifier : on bloque (fichier court typiquement)
        match rx.recv() {
            Ok(Ok(text)) => {
                self.result = Some(if text.is_empty() { "[Silence détecté]".to_string() } else { text });
                self.processing = false;
            }
            Ok(Err(e)) => {
                self.error = Some(e.to_string());
                self.processing = false;
            }
            Err(_) => {
                self.error = Some("Erreur interne".to_string());
                self.processing = false;
            }
        }
    }
}

impl eframe::App for DropWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Gérer les fichiers droppés
        let dropped: Vec<std::path::PathBuf> = ctx.input(|i| {
            i.raw.dropped_files.iter()
                .filter_map(|f| f.path.clone())
                .collect()
        });
        if let Some(path) = dropped.into_iter().next() {
            self.process_file(path);
        }

        let frame_style = egui::Frame::central_panel(&ctx.style())
            .fill(Color32::from_rgb(18, 18, 24))
            .inner_margin(egui::Margin::same(24.0));

        egui::CentralPanel::default().frame(frame_style).show(ctx, |ui| {
            ui.set_min_size(Vec2::new(520.0, 400.0));
            ui.label(RichText::new("Transcription fichier").size(22.0).color(Color32::WHITE).strong());
            ui.add_space(8.0);
            ui.label(RichText::new("Glisser un fichier WAV ici pour le transcrire.").color(Color32::GRAY));
            ui.add_space(16.0);

            // Zone drop visuelle
            let drop_zone = egui::Frame::none()
                .fill(Color32::from_rgb(26, 26, 36))
                .stroke(egui::Stroke::new(2.0, Color32::from_rgb(60, 80, 120)))
                .rounding(12.0)
                .inner_margin(egui::Margin::same(32.0));

            drop_zone.show(ui, |ui| {
                ui.set_min_size(Vec2::new(460.0, 80.0));
                ui.centered_and_justified(|ui| {
                    if self.processing {
                        ui.label(RichText::new("Transcription en cours...").color(Color32::from_rgb(220,140,20)));
                    } else if let Some(ref path) = self.dropped_file {
                        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                        ui.label(RichText::new(format!("📄 {}", name)).color(Color32::LIGHT_GRAY));
                    } else {
                        ui.label(RichText::new("⬇  Déposer un fichier WAV").size(18.0).color(Color32::from_rgb(80, 120, 180)));
                    }
                });
            });

            ui.add_space(16.0);

            if let Some(ref text) = self.result.clone() {
                ui.label(RichText::new("Résultat :").color(Color32::GRAY));
                ui.add_space(4.0);

                egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                    ui.label(RichText::new(text).color(Color32::WHITE));
                });

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("📋 Copier").clicked() {
                        if let Ok(mut cb) = arboard::Clipboard::new() {
                            cb.set_text(text).ok();
                        }
                    }
                    if ui.button("🗑 Effacer").clicked() {
                        self.result = None;
                        self.dropped_file = None;
                    }
                });
            }

            if let Some(ref err) = self.error {
                ui.add_space(8.0);
                ui.label(RichText::new(format!("Erreur : {}", err)).color(Color32::RED));
            }

            ui.add_space(16.0);
            ui.label(RichText::new(format!("Modèle : {} | Langue : {}", self.cfg.model_name(), self.cfg.language_name())).color(Color32::GRAY).small());

            if ui.add(egui::Button::new(RichText::new("Fermer").color(Color32::GRAY))
                .fill(Color32::from_rgb(30,30,40))).clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    }
}

pub fn open(cfg: Config) {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Dictum — Transcription fichier")
            .with_inner_size([560.0, 460.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Dictum — Transcription fichier",
        options,
        Box::new(move |_cc| Box::new(DropWindow::new(cfg))),
    );
}
