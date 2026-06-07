/// Fenêtre paramètres graphique Dictum (egui/eframe).
use eframe::egui::{self, Color32, RichText, Vec2};
use crate::config::Config;

#[derive(PartialEq, Clone)]
enum Tab { General, Substitutions }

struct SettingsWindow {
    cfg: Config,
    saved: bool,
    save_error: Option<String>,
    dirty: bool,
    tab: Tab,
    new_sub_from: String,
    new_sub_to: String,
    new_sub_case: bool,
}

impl SettingsWindow {
    fn new(cfg: Config) -> Self {
        Self {
            cfg, saved: false, save_error: None, dirty: false,
            tab: Tab::General,
            new_sub_from: String::new(), new_sub_to: String::new(), new_sub_case: true,
        }
    }
}

impl eframe::App for SettingsWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let frame_style = egui::Frame::central_panel(&ctx.style())
            .fill(Color32::from_rgb(18, 18, 24))
            .inner_margin(egui::Margin::same(24.0));

        egui::CentralPanel::default().frame(frame_style).show(ctx, |ui| {
            ui.set_min_size(Vec2::new(500.0, 500.0));
            ui.label(RichText::new("Paramètres Dictum").size(24.0).color(Color32::WHITE).strong());
            ui.add_space(8.0);

            // Onglets
            ui.horizontal(|ui| {
                let tab = self.tab.clone();
                if ui.selectable_label(tab == Tab::General, "⚙ Général").clicked() {
                    self.tab = Tab::General;
                }
                if ui.selectable_label(tab == Tab::Substitutions, "🔄 Substitutions").clicked() {
                    self.tab = Tab::Substitutions;
                }
            });
            ui.separator();
            ui.add_space(8.0);

            match self.tab.clone() {
                Tab::General => {
                    // Langue
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Langue :").color(Color32::GRAY));
                        for lang in &["fr", "en", "de", "es", "it", "pt", "auto"] {
                            if ui.selectable_label(self.cfg.language == *lang, RichText::new(*lang).monospace()).clicked() {
                                self.cfg.language = lang.to_string();
                                self.cfg.french_typography = *lang == "fr";
                                self.dirty = true;
                            }
                        }
                    });
                    ui.add_space(8.0);

                    // Hotkey
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Hotkey :").color(Color32::GRAY));
                        if ui.checkbox(&mut self.cfg.hotkey.ctrl, "Ctrl").changed() { self.dirty = true; }
                        if ui.checkbox(&mut self.cfg.hotkey.alt, "Alt").changed() { self.dirty = true; }
                        if ui.checkbox(&mut self.cfg.hotkey.shift, "Shift").changed() { self.dirty = true; }
                        let keys = ["F1","F2","F3","F4","F5","F6","F7","F8","F9","F10","F11","F12","Insert","Home","End"];
                        let cur = self.cfg.hotkey.key.clone();
                        egui::ComboBox::from_id_source("hotkey_key")
                            .selected_text(&cur)
                            .show_ui(ui, |ui| {
                                for k in &keys {
                                    if ui.selectable_label(cur == *k, *k).clicked() {
                                        self.cfg.hotkey.key = k.to_string();
                                        self.dirty = true;
                                    }
                                }
                            });
                    });
                    ui.label(RichText::new(format!("→ {}", self.cfg.hotkey_string())).color(Color32::from_rgb(100,160,255)).small());
                    ui.add_space(8.0);

                    // Options booléennes
                    let mut changed = false;
                    changed |= ui.checkbox(&mut self.cfg.beep_enabled, "Beep au début/fin").changed();
                    changed |= ui.checkbox(&mut self.cfg.auto_capitalize, "Majuscule automatique").changed();
                    changed |= ui.checkbox(&mut self.cfg.french_typography, "Typographie française").changed();
                    changed |= ui.checkbox(&mut self.cfg.auto_enter, "Entrée automatique après dictée").changed();
                    changed |= ui.checkbox(&mut self.cfg.prefix_space, "Espace avant le texte").changed();
                    changed |= ui.checkbox(&mut self.cfg.pause_media, "Pause médias pendant dictée").changed();
                    if changed { self.dirty = true; }
                    ui.add_space(8.0);

                    // Seuil silence
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Seuil silence :").color(Color32::GRAY));
                        if ui.add(egui::Slider::new(&mut self.cfg.silence_threshold, 0.0..=0.1).step_by(0.001)).changed() {
                            self.dirty = true;
                        }
                        ui.label(RichText::new(self.cfg.silence_level_label()).color(Color32::LIGHT_GRAY).small());
                    });

                    // Max historique
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Historique max :").color(Color32::GRAY));
                        let mut n = self.cfg.max_history as i32;
                        if ui.add(egui::DragValue::new(&mut n).clamp_range(1..=1000)).changed() {
                            self.cfg.max_history = n as usize;
                            self.dirty = true;
                        }
                        ui.label(RichText::new("entrées").color(Color32::GRAY).small());
                    });
                    ui.add_space(8.0);
                }
                Tab::Substitutions => {
                    // Liste des substitutions existantes
                    let count = self.cfg.substitutions.len();
                    ui.label(RichText::new(format!("{} substitution(s) :", count)).color(Color32::GRAY));
                    ui.add_space(4.0);

                    let mut to_remove: Option<usize> = None;
                    egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                        for (i, sub) in self.cfg.substitutions.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&sub.from).color(Color32::from_rgb(220,120,50)).monospace());
                                ui.label(RichText::new("→").color(Color32::GRAY));
                                ui.label(RichText::new(&sub.to).color(Color32::WHITE).monospace());
                                if sub.case_insensitive {
                                    ui.label(RichText::new("(i)").color(Color32::GRAY).small());
                                }
                                if ui.small_button("✕").clicked() { to_remove = Some(i); }
                            });
                        }
                    });
                    if let Some(idx) = to_remove {
                        self.cfg.substitutions.remove(idx);
                        self.dirty = true;
                    }

                    ui.separator();
                    ui.add_space(4.0);
                    ui.label(RichText::new("Ajouter :").color(Color32::GRAY).small());
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.new_sub_from);
                        ui.label(RichText::new("→").color(Color32::GRAY));
                        ui.text_edit_singleline(&mut self.new_sub_to);
                        ui.checkbox(&mut self.new_sub_case, "Insensible");
                    });
                    if ui.small_button("+ Ajouter").clicked()
                        && !self.new_sub_from.is_empty() && !self.new_sub_to.is_empty()
                    {
                        self.cfg.add_substitution(&self.new_sub_from.clone(), &self.new_sub_to.clone(), self.new_sub_case);
                        self.new_sub_from.clear();
                        self.new_sub_to.clear();
                        self.dirty = true;
                    }
                }
            }

            ui.add_space(12.0);

            // Score
            let score = self.cfg.score();
            let color = if score >= 80 { Color32::from_rgb(80,200,120) } else if score >= 50 { Color32::from_rgb(220,160,50) } else { Color32::from_rgb(220,80,80) };
            ui.label(RichText::new(format!("Score : {}/100 ({})", score, self.cfg.score_label())).color(color).small());
            ui.add_space(8.0);

            // Boutons
            ui.horizontal(|ui| {
                let save_btn = egui::Button::new(RichText::new("💾 Sauvegarder").color(Color32::WHITE).strong())
                    .fill(if self.dirty { Color32::from_rgb(70,120,200) } else { Color32::from_rgb(40,40,50) })
                    .min_size(Vec2::new(140.0, 34.0));
                if ui.add(save_btn).clicked() && self.dirty {
                    match self.cfg.save() {
                        Ok(_) => { self.saved = true; self.dirty = false; self.save_error = None; }
                        Err(e) => { self.save_error = Some(e.to_string()); }
                    }
                }
                ui.add_space(8.0);
                if ui.add(egui::Button::new(RichText::new("✕ Fermer").color(Color32::GRAY))
                    .fill(Color32::from_rgb(30,30,40)).min_size(Vec2::new(100.0, 34.0))).clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            if self.saved {
                ui.label(RichText::new("✓ Sauvegardé — redémarrer pour appliquer le hotkey.").color(Color32::from_rgb(80,200,120)).small());
            }
            if let Some(ref err) = self.save_error {
                ui.label(RichText::new(format!("Erreur : {}", err)).color(Color32::RED).small());
            }
        });
    }
}

pub fn open(cfg: Config) {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Dictum — Paramètres")
            .with_inner_size([520.0, 540.0])
            .with_resizable(false),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Dictum — Paramètres",
        options,
        Box::new(move |_cc| Box::new(SettingsWindow::new(cfg))),
    );
}
