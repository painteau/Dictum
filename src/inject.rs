use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use crate::config::Config;

pub fn inject_text(text: &str, config: &Config) {
    let text = text.trim().to_string();
    if text.is_empty() { return; }

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

    // Small delay so the key-release event from hotkey doesn't interfere
    std::thread::sleep(std::time::Duration::from_millis(80));

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
    text
        // Espaces insécables avant ponctuation double
        .replace(" ?", "\u{00A0}?")
        .replace(" !", "\u{00A0}!")
        .replace(" :", "\u{00A0}:")
        .replace(" ;", "\u{00A0};")
        // Apostrophe typographique ' → '
        .replace("'", "\u{2019}")
        // Points de suspension ... → …
        .replace("...", "\u{2026}")
}
