use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use crate::config::Config;

pub fn inject_text(text: &str, config: &Config) {
    let text = text.trim().to_string();
    if text.is_empty() { return; }

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

    // Délai pour laisser le système traiter le key-release du hotkey
    // Texte long → délai légèrement plus court (le système est prêt)
    let delay_ms = if text.len() > 200 { 60 } else { 80 };
    std::thread::sleep(std::time::Duration::from_millis(delay_ms));

    let settings = Settings::default();
    match Enigo::new(&settings) {
        Ok(mut enigo) => {
            // Injecter ligne par ligne pour gérer les \n (Whisper peut retourner plusieurs segments)
            let lines: Vec<&str> = text.split('\n').collect();
            for (i, line) in lines.iter().enumerate() {
                if !line.is_empty() {
                    if let Err(e) = enigo.text(line) {
                        log::error!("Injection échouée : {e}");
                        return;
                    }
                }
                if i < lines.len() - 1 {
                    let _ = enigo.key(Key::Return, Direction::Click);
                }
            }
            if config.auto_enter {
                let _ = enigo.key(Key::Return, Direction::Click);
            }
        }
        Err(e) => log::error!("Enigo init échoué : {e}"),
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
