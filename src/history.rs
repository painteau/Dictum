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
            .map(|(i, e)| {
                let time = format_timestamp(e.timestamp);
                format!("{}. [{}]  {}", i + 1, time, e.text)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn format_timestamp(ts: u64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH, Duration};
    let dt = SystemTime::UNIX_EPOCH + Duration::from_secs(ts);
    match dt.duration_since(UNIX_EPOCH) {
        Ok(d) => {
            let secs = d.as_secs();
            let h = (secs % 86400) / 3600;
            let m = (secs % 3600) / 60;
            format!("{:02}:{:02}", h, m)
        }
        Err(_) => "??:??".to_string(),
    }
}

fn history_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Dictum")
        .join("history.json")
}
