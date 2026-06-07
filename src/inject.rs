use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use crate::config::Config;
use arboard;

/// Inject texte brut sans aucun traitement typographique ni capitalisation.
#[allow(dead_code)]
pub fn inject_raw(text: &str, delay_ms: u64) {
    if text.is_empty() { return; }
    std::thread::sleep(std::time::Duration::from_millis(delay_ms));
    let settings = enigo::Settings::default();
    match enigo::Enigo::new(&settings) {
        Ok(mut enigo) => { let _ = enigo.text(text); }
        Err(e) => log::error!("inject_raw échoué : {e}"),
    }
}

pub fn inject_text(text: &str, config: &Config) {
    let text = text.trim().to_string();
    if text.is_empty() { return; }
    log::debug!("Injection : {} chars, typo_fr={} capitalize={}", text.len(), config.french_typography, config.auto_capitalize);

    let text = if config.prefix_space {
        format!(" {}", text)
    } else {
        text
    };

    let text = if config.auto_capitalize {
        capitalize_first(&text)
    } else {
        text
    };

    let text = if config.french_typography {
        apply_french_typography(&text)
    } else {
        text
    };

    // Délai configurable pour laisser le système traiter le key-release
    let delay_ms = if config.inject_delay_ms > 0 { config.inject_delay_ms } else { 80 };
    std::thread::sleep(std::time::Duration::from_millis(delay_ms));

    let settings = Settings::default();
    match Enigo::new(&settings) {
        Ok(mut enigo) => {
            // Injecter ligne par ligne pour gérer les \n (Whisper peut retourner plusieurs segments)
            let lines: Vec<&str> = text.split('\n').collect();
            const CHUNK_SIZE: usize = 500;
            for (i, line) in lines.iter().enumerate() {
                if !line.is_empty() {
                    // Injection par chunks si ligne > 500 chars (évite timeouts enigo)
                    if line.len() > CHUNK_SIZE {
                        for chunk in line.as_bytes().chunks(CHUNK_SIZE) {
                            let s = String::from_utf8_lossy(chunk);
                            if let Err(e) = enigo.text(&s) {
                                log::error!("Injection chunk échouée : {e}");
                                return;
                            }
                            std::thread::sleep(std::time::Duration::from_millis(10));
                        }
                    } else if let Err(e) = enigo.text(line) {
                        log::error!("Injection échouée : {e}");
                        return;
                    }
                }
                if i < lines.len() - 1 {
                    let _ = enigo.key(Key::Return, Direction::Click);
                }
            }
            if config.auto_enter {
                log::debug!("Auto-Enter déclenché");
                let _ = enigo.key(Key::Return, Direction::Click);
            }
            log::info!("Texte injecté ({} chars)", text.len());
        }
        Err(e) => {
            log::error!("Enigo init échoué : {e} — fallback clipboard");
            clipboard_paste_fallback(&text);
        }
    }
}

/// Fallback : copie le texte dans le presse-papiers et simule Ctrl+V.
fn clipboard_paste_fallback(text: &str) {
    match arboard::Clipboard::new() {
        Ok(mut cb) => {
            if cb.set_text(text).is_err() {
                log::error!("Fallback clipboard : impossible d'écrire dans le presse-papiers");
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
            let settings = Settings::default();
            match Enigo::new(&settings) {
                Ok(mut enigo) => {
                    let _ = enigo.key(Key::Control, Direction::Press);
                    let _ = enigo.key(Key::Unicode('v'), Direction::Click);
                    let _ = enigo.key(Key::Control, Direction::Release);
                    log::info!("Texte injecté via fallback clipboard ({} chars)", text.len());
                }
                Err(e2) => log::error!("Fallback clipboard : Ctrl+V échoué : {e2}"),
            }
        }
        Err(e) => log::error!("Fallback clipboard inaccessible : {e}"),
    }
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn apply_french_typography(text: &str) -> String {
    // Points de suspension en premier (avant d'autres substitutions)
    let text = text.replace("...", "\u{2026}");
    // Apostrophe typographique
    let text = text.replace("'", "\u{2019}");
    // Guillemets français : "texte" → « texte »
    let text = replace_quotes_french(&text);
    // Espaces insécables avant ponctuation double (espace normale ET espace insécable déjà présente)
    let text = text
        .replace(" ?", "\u{00A0}?")
        .replace(" !", "\u{00A0}!")
        .replace(" :", "\u{00A0}:")
        .replace(" ;", "\u{00A0};");
    // Tiret demi-cadratin dans les incises (en début de phrase uniquement)
    let text = if text.starts_with("- ") {
        format!("\u{2013} {}", &text[2..])
    } else {
        text
    };
    text
}

fn replace_quotes_french(text: &str) -> String {
    // Remplace "texte" par « texte » (guillemets droits uniquement)
    let mut result = String::with_capacity(text.len());
    let mut open = false;
    for ch in text.chars() {
        if ch == '"' {
            if open {
                result.push_str("\u{00A0}»"); // espace insécable + »
            } else {
                result.push_str("«\u{00A0}"); // « + espace insécable
            }
            open = !open;
        } else {
            result.push(ch);
        }
    }
    result
}
