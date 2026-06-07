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
    pub fn keep_recent(&mut self, n: usize) {
        while self.entries.len() > n {
            self.entries.pop_back();
        }
        log::debug!("Historique tronqué à {} entrée(s)", self.entries.len());
    }

    #[allow(dead_code)]
    pub fn deduplicate(&mut self) {
        let mut seen = std::collections::HashSet::new();
        self.entries.retain(|e| seen.insert(e.text.clone()));
        log::debug!("Historique dédupliqué : {} entrée(s) restantes", self.entries.len());
    }

    #[allow(dead_code)]
    pub fn contains(&self, text: &str) -> bool {
        self.entries.iter().any(|e| e.text == text)
    }

    #[allow(dead_code)]
    pub fn count_containing(&self, query: &str) -> usize {
        let q = query.to_lowercase();
        self.entries.iter().filter(|e| e.text.to_lowercase().contains(&q)).count()
    }

    #[allow(dead_code)]
    pub fn sorted_by_length(&self) -> Vec<&HistoryEntry> {
        let mut v: Vec<&HistoryEntry> = self.entries.iter().collect();
        v.sort_by(|a, b| b.text.len().cmp(&a.text.len()));
        v
    }

    #[allow(dead_code)]
    pub fn sorted_by_date(&self) -> Vec<&HistoryEntry> {
        let mut v: Vec<&HistoryEntry> = self.entries.iter().collect();
        v.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        v
    }

    #[allow(dead_code)]
    pub fn longest_entry(&self) -> Option<&HistoryEntry> {
        self.entries.iter().max_by_key(|e| e.text.len())
    }

    #[allow(dead_code)]
    pub fn shortest_entry(&self) -> Option<&HistoryEntry> {
        self.entries.iter().min_by_key(|e| e.text.len())
    }

    #[allow(dead_code)]
    pub fn time_since_last(&self) -> Option<u64> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default().as_secs();
        self.entries.front().map(|e| now.saturating_sub(e.timestamp))
    }

    #[allow(dead_code)]
    pub fn is_empty_or_old(&self, max_age_secs: u64) -> bool {
        self.is_empty() || self.time_since_last().map(|t| t > max_age_secs).unwrap_or(true)
    }

    #[allow(dead_code)]
    pub fn recent_texts(&self, n: usize) -> Vec<String> {
        self.entries.iter().take(n).map(|e| e.text.clone()).collect()
    }

    #[allow(dead_code)]
    pub fn oldest_entry(&self) -> Option<&HistoryEntry> {
        self.entries.back()
    }

    #[allow(dead_code)]
    pub fn entry_at_index(&self, idx: usize) -> Option<&HistoryEntry> {
        self.entries.get(idx)
    }

    #[allow(dead_code)]
    pub fn longest_text_length(&self) -> usize {
        self.entries.iter().map(|e| e.text.len()).max().unwrap_or(0)
    }

    #[allow(dead_code)]
    pub fn shortest_text_length(&self) -> usize {
        self.entries.iter().map(|e| e.text.len()).min().unwrap_or(0)
    }

    #[allow(dead_code)]
    pub fn is_growing(&self) -> bool { self.len() > 0 }

    #[allow(dead_code)]
    pub fn percentage_full(&self, max: usize) -> f32 {
        if max == 0 { return 100.0; }
        (self.len() as f32 / max as f32 * 100.0).min(100.0)
    }

    #[allow(dead_code)]
    pub fn first_entry(&self) -> Option<&HistoryEntry> {
        self.entries.back()
    }

    #[allow(dead_code)]
    pub fn text_at(&self, idx: usize) -> Option<String> {
        self.entries.get(idx).map(|e| e.text.clone())
    }

    #[allow(dead_code)]
    pub fn timestamp_at(&self, idx: usize) -> Option<u64> {
        self.entries.get(idx).map(|e| e.timestamp)
    }

    #[allow(dead_code)]
    pub fn count_words_in(&self, idx: usize) -> usize {
        self.entries.get(idx).map(|e| e.text.split_whitespace().count()).unwrap_or(0)
    }

    #[allow(dead_code)]
    pub fn longest_word(&self) -> Option<String> {
        let mut best = String::new();
        for e in &self.entries {
            for w in e.text.split_whitespace() {
                if w.len() > best.len() { best = w.to_string(); }
            }
        }
        if best.is_empty() { None } else { Some(best) }
    }

    #[allow(dead_code)]
    pub fn char_frequency(&self) -> std::collections::HashMap<char, usize> {
        let mut freq = std::collections::HashMap::new();
        for e in &self.entries {
            for c in e.text.chars().filter(|c| c.is_alphabetic()) {
                let lc = c.to_lowercase().next().unwrap_or(c);
                *freq.entry(lc).or_insert(0) += 1;
            }
        }
        freq
    }

    #[allow(dead_code)]
    pub fn average_words_per_char(&self) -> f32 {
        let chars = self.total_chars();
        let words = self.words_count();
        if words == 0 || chars == 0 { return 0.0; }
        words as f32 / chars as f32
    }

    #[allow(dead_code)]
    pub fn entries_in_range(&self, from: u64, to: u64) -> Vec<&HistoryEntry> {
        self.entries.iter().filter(|e| e.timestamp >= from && e.timestamp <= to).collect()
    }

    #[allow(dead_code)]
    pub fn push_unique(&mut self, text: String, max: usize) -> bool {
        if self.contains(&text) { return false; }
        self.push_with_limit(text, max);
        true
    }

    #[allow(dead_code)]
    pub fn total_sentences(&self) -> usize {
        self.entries.iter().map(|e| {
            e.text.chars().filter(|&c| ".!?".contains(c)).count().max(1)
        }).sum()
    }

    #[allow(dead_code)]
    pub fn export_csv(&self) -> String {
        let mut out = String::from("timestamp,chars,words,text\n");
        for e in &self.entries {
            let words = e.text.split_whitespace().count();
            let text_escaped = e.text.replace('"', "\"\"");
            out.push_str(&format!("{},{},{},\"{}\"\n", e.timestamp, e.text.len(), words, text_escaped));
        }
        out
    }

    #[allow(dead_code)]
    pub fn percentile_length(&self, pct: f32) -> usize {
        if self.is_empty() { return 0; }
        let mut lengths: Vec<usize> = self.entries.iter().map(|e| e.text.len()).collect();
        lengths.sort();
        let idx = ((pct / 100.0) * (lengths.len() - 1) as f32).round() as usize;
        lengths[idx.min(lengths.len() - 1)]
    }

    #[allow(dead_code)]
    pub fn median_length(&self) -> usize {
        self.percentile_length(50.0)
    }

    #[allow(dead_code)]
    pub fn texts_starting_with(&self, prefix: &str) -> Vec<&HistoryEntry> {
        let p = prefix.to_lowercase();
        self.entries.iter().filter(|e| e.text.to_lowercase().starts_with(&p)).collect()
    }

    #[allow(dead_code)]
    pub fn remove_at(&mut self, idx: usize) -> Option<HistoryEntry> {
        self.entries.remove(idx)
    }

    #[allow(dead_code)]
    pub fn swap(&mut self, i: usize, j: usize) {
        if i < self.entries.len() && j < self.entries.len() {
            self.entries.swap(i, j);
        }
    }

    #[allow(dead_code)]
    pub fn retain_matching(&mut self, query: &str) {
        let q = query.to_lowercase();
        self.entries.retain(|e| e.text.to_lowercase().contains(&q));
    }

    #[allow(dead_code)]
    pub fn clone_entries(&self) -> Vec<HistoryEntry> {
        self.entries.iter().cloned().collect()
    }

    #[allow(dead_code)]
    pub fn count_chars_total(&self) -> usize {
        self.entries.iter().map(|e| e.text.chars().count()).sum()
    }

    #[allow(dead_code)]
    pub fn texts_ending_with(&self, suffix: &str) -> Vec<&HistoryEntry> {
        let s = suffix.to_lowercase();
        self.entries.iter().filter(|e| e.text.to_lowercase().ends_with(&s)).collect()
    }

    #[allow(dead_code)]
    pub fn has_entry_with_min_chars(&self, min: usize) -> bool {
        self.entries.iter().any(|e| e.text.len() >= min)
    }

    #[allow(dead_code)]
    pub fn last_n(&self, n: usize) -> Vec<&HistoryEntry> {
        self.entries.iter().take(n).collect()
    }

    #[allow(dead_code)]
    pub fn oldest_n(&self, n: usize) -> Vec<&HistoryEntry> {
        self.entries.iter().rev().take(n).collect()
    }

    #[allow(dead_code)]
    pub fn average_timestamp_gap(&self) -> Option<u64> {
        if self.entries.len() < 2 { return None; }
        let ts: Vec<u64> = self.entries.iter().map(|e| e.timestamp).collect();
        let gaps: Vec<u64> = ts.windows(2).map(|w| w[0].saturating_sub(w[1])).collect();
        let sum: u64 = gaps.iter().sum();
        Some(sum / gaps.len() as u64)
    }

    #[allow(dead_code)]
    pub fn to_plain_text(&self) -> String {
        self.entries.iter().map(|e| e.text.as_str()).collect::<Vec<_>>().join("\n")
    }

    #[allow(dead_code)]
    pub fn truncate_to(&mut self, n: usize) {
        while self.entries.len() > n {
            self.entries.pop_back();
        }
    }

    #[allow(dead_code)]
    pub fn entry_word_count(&self, idx: usize) -> usize {
        self.entries.get(idx).map(|e| e.text.split_whitespace().count()).unwrap_or(0)
    }

    #[allow(dead_code)]
    pub fn richest_entry(&self) -> Option<&HistoryEntry> {
        self.entries.iter().max_by_key(|e| e.text.split_whitespace().count())
    }

    #[allow(dead_code)]
    pub fn is_full(&self, max: usize) -> bool {
        self.entries.len() >= max
    }

    #[allow(dead_code)]
    pub fn count_longer_than(&self, chars: usize) -> usize {
        self.entries.iter().filter(|e| e.text.len() > chars).count()
    }

    #[allow(dead_code)]
    pub fn count_shorter_than(&self, chars: usize) -> usize {
        self.entries.iter().filter(|e| e.text.len() < chars).count()
    }

    #[allow(dead_code)]
    pub fn latest_timestamp(&self) -> Option<u64> {
        self.entries.iter().map(|e| e.timestamp).max()
    }

    #[allow(dead_code)]
    pub fn earliest_timestamp(&self) -> Option<u64> {
        self.entries.iter().map(|e| e.timestamp).min()
    }

    #[allow(dead_code)]
    pub fn age_secs(&self) -> Option<u64> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default().as_secs();
        self.earliest_timestamp().map(|t| now.saturating_sub(t))
    }

    #[allow(dead_code)]
    pub fn words_in_last_n(&self, n: usize) -> usize {
        self.entries.iter().take(n)
            .map(|e| e.text.split_whitespace().count())
            .sum()
    }

    #[allow(dead_code)]
    pub fn summary_line(&self) -> String {
        if self.is_empty() {
            "Aucune dictée".to_string()
        } else {
            format!("{} dictée(s) | {} mots | {} chars",
                self.len(), self.words_count(), self.total_chars())
        }
    }

    #[allow(dead_code)]
    pub fn top_words(&self, n: usize) -> Vec<(String, usize)> {
        let freq = self.word_frequency();
        let mut list: Vec<(String, usize)> = freq.into_iter().collect();
        list.sort_by(|a, b| b.1.cmp(&a.1));
        list.truncate(n);
        list
    }

    #[allow(dead_code)]
    pub fn has_entry_older_than(&self, secs: u64) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default().as_secs();
        self.entries.iter().any(|e| now.saturating_sub(e.timestamp) > secs)
    }

    #[allow(dead_code)]
    pub fn entries_with_char(&self, ch: char) -> Vec<&HistoryEntry> {
        self.entries.iter().filter(|e| e.text.contains(ch)).collect()
    }

    #[allow(dead_code)]
    pub fn entries_without_char(&self, ch: char) -> Vec<&HistoryEntry> {
        self.entries.iter().filter(|e| !e.text.contains(ch)).collect()
    }

    #[allow(dead_code)]
    pub fn average_word_length(&self) -> f32 {
        let words = self.words_count();
        if words == 0 { return 0.0; }
        let chars: usize = self.entries.iter().flat_map(|e| e.text.split_whitespace()).map(|w| w.len()).sum();
        chars as f32 / words as f32
    }

    #[allow(dead_code)]
    pub fn has_duplicate(&self) -> bool {
        let mut seen = std::collections::HashSet::new();
        self.entries.iter().any(|e| !seen.insert(&e.text))
    }

    #[allow(dead_code)]
    pub fn count_duplicates(&self) -> usize {
        let mut seen = std::collections::HashSet::new();
        self.entries.iter().filter(|e| !seen.insert(&e.text)).count()
    }

    #[allow(dead_code)]
    pub fn texts_matching(&self, predicate: impl Fn(&str) -> bool) -> Vec<&HistoryEntry> {
        self.entries.iter().filter(|e| predicate(&e.text)).collect()
    }

    #[allow(dead_code)]
    pub fn word_count_histogram(&self) -> Vec<(usize, usize)> {
        let mut bins: std::collections::BTreeMap<usize, usize> = std::collections::BTreeMap::new();
        for e in &self.entries {
            let bucket = (e.text.split_whitespace().count() / 10) * 10;
            *bins.entry(bucket).or_insert(0) += 1;
        }
        bins.into_iter().collect()
    }

    #[allow(dead_code)]
    pub fn timestamps(&self) -> Vec<u64> {
        self.entries.iter().map(|e| e.timestamp).collect()
    }

    #[allow(dead_code)]
    pub fn at_timestamp(&self, ts: u64) -> Option<&HistoryEntry> {
        self.entries.iter().find(|e| e.timestamp == ts)
    }

    #[allow(dead_code)]
    pub fn merge(&mut self, other: &History, max: usize) {
        for entry in other.entries.iter() {
            if !self.contains(&entry.text) {
                self.push_with_limit(entry.text.clone(), max);
            }
        }
    }

    #[allow(dead_code)]
    pub fn chars_per_word_ratio(&self) -> f32 {
        let words = self.words_count();
        if words == 0 { return 0.0; }
        self.total_chars() as f32 / words as f32
    }

    #[allow(dead_code)]
    pub fn contains_all(&self, queries: &[&str]) -> Vec<&HistoryEntry> {
        self.entries.iter().filter(|e| {
            let t = e.text.to_lowercase();
            queries.iter().all(|q| t.contains(&q.to_lowercase()))
        }).collect()
    }

    #[allow(dead_code)]
    pub fn contains_any(&self, queries: &[&str]) -> Vec<&HistoryEntry> {
        self.entries.iter().filter(|e| {
            let t = e.text.to_lowercase();
            queries.iter().any(|q| t.contains(&q.to_lowercase()))
        }).collect()
    }

    #[allow(dead_code)]
    pub fn entry_index(&self, text: &str) -> Option<usize> {
        self.entries.iter().position(|e| e.text == text)
    }

    #[allow(dead_code)]
    pub fn save_to(&self, path: &std::path::Path) -> anyhow::Result<()> {
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn load_from(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content).unwrap_or_default())
    }

    #[allow(dead_code)]
    pub fn words_today(&self) -> usize {
        self.entries_today().iter().map(|e| e.text.split_whitespace().count()).sum()
    }

    #[allow(dead_code)]
    pub fn chars_today(&self) -> usize {
        self.entries_today().iter().map(|e| e.text.len()).sum()
    }

    #[allow(dead_code)]
    pub fn most_recent_n_words(&self, n: usize) -> String {
        self.entries.iter().take(n)
            .flat_map(|e| e.text.split_whitespace().map(str::to_string))
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[allow(dead_code)]
    pub fn random_entry_index(&self) -> Option<usize> {
        if self.is_empty() { return None; }
        // Déterministe : basé sur timestamp de la dernière entrée
        Some(self.entries.front()?.timestamp as usize % self.entries.len())
    }

    #[allow(dead_code)]
    pub fn json_string(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    #[allow(dead_code)]
    pub fn avg_words_per_entry(&self) -> usize {
        if self.is_empty() { return 0; }
        self.words_count() / self.len()
    }

    #[allow(dead_code)]
    pub fn entries_after(&self, timestamp: u64) -> Vec<&HistoryEntry> {
        self.entries.iter().filter(|e| e.timestamp >= timestamp).collect()
    }

    #[allow(dead_code)]
    pub fn sentences_count(&self) -> usize {
        self.entries.iter().map(|e| {
            e.text.chars().filter(|&c| c == '.' || c == '!' || c == '?').count().max(1)
        }).sum()
    }

    #[allow(dead_code)]
    pub fn word_frequency(&self) -> std::collections::HashMap<String, usize> {
        let mut freq = std::collections::HashMap::new();
        for e in &self.entries {
            for word in e.text.split_whitespace() {
                let w = word.to_lowercase().trim_matches(|c: char| !c.is_alphabetic()).to_string();
                if w.len() >= 3 { *freq.entry(w).or_insert(0) += 1; }
            }
        }
        freq
    }

    #[allow(dead_code)]
    pub fn words_count(&self) -> usize {
        self.entries.iter().map(|e| e.text.split_whitespace().count()).sum()
    }

    #[allow(dead_code)]
    pub fn has_recent_entry(&self, within_secs: u64) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default().as_secs();
        self.entries.front().map(|e| now.saturating_sub(e.timestamp) <= within_secs).unwrap_or(false)
    }

    #[allow(dead_code)]
    pub fn entries_today(&self) -> Vec<&HistoryEntry> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default().as_secs();
        let start_of_day = now - (now % 86400);
        self.entries.iter().filter(|e| e.timestamp >= start_of_day).collect()
    }

    #[allow(dead_code)]
    pub fn stats_summary(&self) -> String {
        if self.is_empty() {
            return "Historique vide".to_string();
        }
        format!("{} entrée(s) | {} chars total | moy.{} | {} mots uniques",
            self.len(), self.total_chars(), self.average_length(), self.unique_words_count())
    }

    #[allow(dead_code)]
    pub fn unique_words_count(&self) -> usize {
        let mut words = std::collections::HashSet::new();
        for e in &self.entries {
            for word in e.text.split_whitespace() {
                let w = word.to_lowercase().trim_matches(|c: char| !c.is_alphabetic()).to_string();
                if w.len() >= 3 { words.insert(w); }
            }
        }
        words.len()
    }

    #[allow(dead_code)]
    pub fn most_common_word(&self) -> Option<String> {
        let mut freq: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for e in &self.entries {
            for word in e.text.split_whitespace() {
                let w = word.to_lowercase().trim_matches(|c: char| !c.is_alphabetic()).to_string();
                if w.len() >= 3 {
                    *freq.entry(w).or_insert(0) += 1;
                }
            }
        }
        freq.into_iter().max_by_key(|(_, v)| *v).map(|(w, _)| w)
    }

    #[allow(dead_code)]
    pub fn filter_by_min_length(&self, min_chars: usize) -> Vec<&HistoryEntry> {
        self.entries.iter().filter(|e| e.text.len() >= min_chars).collect()
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
    pub fn to_markdown(&self) -> String {
        let header = format!("# Historique Dictum — {}\n\n", self.stats_summary());
        let content = self.entries
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let time = format_timestamp(e.timestamp);
                format!("## {} — [{}] ({} chars)\n\n{}\n", i + 1, time, e.text.len(), e.text)
            })
            .collect::<Vec<_>>()
            .join("\n---\n\n");
        format!("{}{}", header, content)
    }

    pub fn export_to_file(&self, path: &std::path::Path) -> anyhow::Result<()> {
        std::fs::write(path, self.to_markdown())?;
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
