use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub text: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct History {
    entries: VecDeque<HistoryEntry>,
}

impl History {
    pub fn load() -> Result<Self> {
        let path = history_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&content).unwrap_or_default())
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = history_path();
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn push_with_limit(&mut self, text: String, max: usize) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        log::debug!("Historique : ajout entrée ({}/{} max)", self.entries.len() + 1, max);
        self.entries.push_front(HistoryEntry { text, timestamp });
        while self.entries.len() > max.max(1) {
            self.entries.pop_back();
        }
    }

    pub fn push(&mut self, text: String) {
        self.push_with_limit(text, 10);
    }

    pub fn entries(&self) -> &VecDeque<HistoryEntry> {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn last_text(&self) -> Option<String> {
        self.entries.front().map(|e| e.text.clone())
    }

    pub fn get_by_index(&self, idx: usize) -> Option<&HistoryEntry> {
        self.entries.get(idx)
    }

    pub fn all_texts(&self) -> Vec<String> {
        self.entries.iter().map(|e| e.text.clone()).collect()
    }

    #[allow(dead_code)]
    pub fn total_chars(&self) -> usize {
        self.entries.iter().map(|e| e.text.len()).sum()
    }

    pub fn average_length(&self) -> usize {
        if self.entries.is_empty() { return 0; }
        self.total_chars() / self.entries.len()
    }

    #[allow(dead_code)]
    pub fn search(&self, query: &str) -> Vec<&HistoryEntry> {
        let query_lower = query.to_lowercase();
        self.entries
            .iter()
            .filter(|e| e.text.to_lowercase().contains(&query_lower))
            .collect()
    }

    #[allow(dead_code)]
    pub fn last_timestamp(&self) -> Option<u64> {
        self.entries.front().map(|e| e.timestamp)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Exporte l'historique complet dans un fichier texte.
    pub fn export_to_file(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let header = format!("# Historique Dictum — {} entrée(s)\n\n", self.entries.len());
        let content = self.entries
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let time = format_timestamp(e.timestamp);
                format!("## {} — [{}]\n{}\n", i + 1, time, e.text)
            })
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(path, format!("{}{}", header, content))?;
        Ok(())
    }

    pub fn as_display_string(&self) -> String {
        if self.entries.is_empty() {
            return "Aucun historique.".to_string();
        }
        self.entries
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let time = format_timestamp(e.timestamp);
                let text = if e.text.len() > 120 {
                    format!("{}...", &e.text[..117])
                } else {
                    e.text.clone()
                };
                format!("{}. [{}]  {}", i + 1, time, text)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn format_timestamp(ts: u64) -> String {
    // Calculer l'heure locale à partir du timestamp UTC
    let secs = ts;
    // Approximation UTC+0 (pas de lib timezone pour rester léger)
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    // Afficher aussi la date si l'entrée date de plus de 24h
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if now_secs.saturating_sub(ts) > 86400 {
        let days_ago = now_secs.saturating_sub(ts) / 86400;
        format!("J-{} {:02}:{:02}", days_ago, h, m)
    } else {
        format!("{:02}:{:02}", h, m)
    }
}

fn history_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Dictum")
        .join("history.json")
}
