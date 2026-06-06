use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use anyhow::Result;

const MAX_ENTRIES: usize = 10;

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

    pub fn push(&mut self, text: String) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.entries.push_front(HistoryEntry { text, timestamp });
        while self.entries.len() > MAX_ENTRIES {
            self.entries.pop_back();
        }
    }

    pub fn entries(&self) -> &VecDeque<HistoryEntry> {
        &self.entries
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn as_display_string(&self) -> String {
        if self.entries.is_empty() {
            return "Aucun historique.".to_string();
        }
        self.entries
            .iter()
            .enumerate()
            .map(|(i, e)| format!("{}. {}", i + 1, e.text))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn history_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Dictum")
        .join("history.json")
}
