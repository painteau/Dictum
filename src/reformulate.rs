/// Reformulation de texte via Ollama local.
/// Aucun cloud — tout tourne sur la machine.

use anyhow::{anyhow, Result};

pub const STYLES: &[(&str, &str, &str)] = &[
    ("formel",      "Formel",           "Reformule ce texte de manière formelle et professionnelle"),
    ("informel",    "Informel",         "Reformule ce texte de manière décontractée et amicale"),
    ("resume",      "Résumé",           "Résume ce texte en 1-2 phrases concises"),
    ("bullets",     "Bullet points",    "Transforme ce texte en liste à puces claire"),
    ("email",       "Email pro",        "Transforme ce texte en email professionnel avec objet et formules de politesse"),
    ("tweet",       "Tweet",            "Condense ce texte en 280 caractères maximum pour Twitter/X"),
    ("correction",  "Correction",       "Corrige uniquement l'orthographe et la grammaire sans changer le sens ni le style"),
];

/// Vérifie si Ollama est disponible à l'URL donnée.
pub fn is_available(url: &str) -> bool {
    let check_url = format!("{}/api/tags", url.trim_end_matches('/'));
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .map(|c| c.get(&check_url).send().map(|r| r.status().is_success()).unwrap_or(false))
        .unwrap_or(false)
}

/// Liste les modèles disponibles dans Ollama.
pub fn list_models(url: &str) -> Vec<String> {
    let check_url = format!("{}/api/tags", url.trim_end_matches('/'));
    let resp = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .ok()
        .and_then(|c| c.get(&check_url).send().ok())
        .and_then(|r| r.json::<serde_json::Value>().ok());

    resp.and_then(|v| {
        v["models"].as_array().map(|arr| {
            arr.iter()
                .filter_map(|m| m["name"].as_str().map(String::from))
                .collect()
        })
    }).unwrap_or_default()
}

/// Reformule un texte via Ollama avec le style donné.
pub fn reformulate(text: &str, style_key: &str, model: &str, url: &str) -> Result<String> {
    let instruction = STYLES.iter()
        .find(|(k, _, _)| *k == style_key)
        .map(|(_, _, instr)| *instr)
        .unwrap_or("Reformule ce texte");

    let prompt = format!("{} :\n\n{}\n\nRéponds uniquement avec le texte reformulé, sans commentaire ni explication.", instruction, text);

    let api_url = format!("{}/api/generate", url.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": false,
        "options": {
            "temperature": 0.7,
            "num_predict": 500
        }
    });

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| anyhow!("Client HTTP : {e}"))?;

    let resp = client.post(&api_url)
        .json(&body)
        .send()
        .map_err(|e| anyhow!("Ollama inaccessible ({}) : {e}", url))?;

    if !resp.status().is_success() {
        return Err(anyhow!("Ollama HTTP {} — modèle '{}' disponible ?", resp.status(), model));
    }

    let json: serde_json::Value = resp.json()
        .map_err(|e| anyhow!("Réponse Ollama invalide : {e}"))?;

    json["response"].as_str()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("Réponse vide d'Ollama"))
}

/// Récupère le texte sélectionné via le presse-papiers (simule Ctrl+C).
pub fn get_selected_text() -> Option<String> {
    use enigo::{Enigo, Key, Direction, Keyboard, Settings};

    // Sauvegarder le clipboard actuel
    let prev = arboard::Clipboard::new().ok()?.get_text().ok();

    // Vider le clipboard
    if let Ok(mut cb) = arboard::Clipboard::new() {
        cb.set_text("").ok();
    }

    // Simuler Ctrl+C
    std::thread::sleep(std::time::Duration::from_millis(50));
    if let Ok(mut enigo) = Enigo::new(&Settings::default()) {
        let _ = enigo.key(Key::Control, Direction::Press);
        let _ = enigo.key(Key::Unicode('c'), Direction::Click);
        let _ = enigo.key(Key::Control, Direction::Release);
    }
    std::thread::sleep(std::time::Duration::from_millis(150));

    // Lire le nouveau clipboard
    let selected = arboard::Clipboard::new().ok()?.get_text().ok()
        .filter(|s| !s.is_empty() && s != prev.as_deref().unwrap_or(""));

    // Restaurer l'ancien clipboard si on n'a rien capturé
    if selected.is_none() {
        if let Some(prev_text) = prev {
            if let Ok(mut cb) = arboard::Clipboard::new() {
                cb.set_text(&prev_text).ok();
            }
        }
    }

    selected
}
