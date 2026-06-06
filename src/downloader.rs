use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;
use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};

/// URL du manifest hébergé — changer cette URL sans recompiler le logiciel.
pub const MANIFEST_URL: &str = "https://cdn.breizhzion.com/dictum/manifest.json";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelEntry {
    pub url: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BinaryEntry {
    pub url: String,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Manifest {
    pub version: String,
    pub models: HashMap<String, ModelEntry>,
    pub binaries: HashMap<String, BinaryEntry>,
}

pub fn fetch_manifest() -> Result<Manifest> {
    let resp = reqwest::blocking::get(MANIFEST_URL)
        .map_err(|e| anyhow!("Impossible de récupérer le manifest : {e}"))?;

    if !resp.status().is_success() {
        return Err(anyhow!("Manifest HTTP {} — URL : {}", resp.status(), MANIFEST_URL));
    }

    resp.json::<Manifest>()
        .map_err(|e| anyhow!("Format manifest invalide : {e}"))
}

/// Télécharge un modèle avec callback de progression (downloaded, total).
/// Vérifie le SHA256 après téléchargement. Supprime le fichier si invalide.
pub fn download_model<F>(entry: &ModelEntry, dest: &Path, mut on_progress: F) -> Result<()>
where
    F: FnMut(u64, u64),
{
    let mut resp = reqwest::blocking::get(&entry.url)
        .map_err(|e| anyhow!("Téléchargement échoué : {e}"))?;

    if !resp.status().is_success() {
        return Err(anyhow!("Erreur HTTP {} lors du téléchargement", resp.status()));
    }

    std::fs::create_dir_all(dest.parent().unwrap())?;
    let mut file = std::fs::File::create(dest)
        .map_err(|e| anyhow!("Impossible de créer le fichier : {e}"))?;

    let mut hasher = Sha256::new();
    let mut downloaded = 0u64;
    let total = entry.size_bytes;
    let mut buf = vec![0u8; 65_536];

    loop {
        let n = resp.read(&mut buf).map_err(|e| anyhow!("Erreur lecture : {e}"))?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).map_err(|e| anyhow!("Erreur écriture : {e}"))?;
        hasher.update(&buf[..n]);
        downloaded += n as u64;
        on_progress(downloaded, total);
    }

    let actual_hash = hex::encode(hasher.finalize());
    if !entry.sha256.is_empty() && actual_hash != entry.sha256 {
        std::fs::remove_file(dest).ok();
        return Err(anyhow!(
            "Checksum invalide !\nAttendu  : {}\nObtenu   : {}",
            entry.sha256,
            actual_hash
        ));
    }

    Ok(())
}

/// Télécharge un fichier binaire (même logique que download_model sans description).
pub fn download_file<F>(url: &str, sha256: &str, size_bytes: u64, dest: &Path, on_progress: F) -> Result<()>
where
    F: FnMut(u64, u64),
{
    let entry = ModelEntry {
        url: url.to_string(),
        sha256: sha256.to_string(),
        size_bytes,
        description: String::new(),
    };
    download_model(&entry, dest, on_progress)
}

/// Télécharge tous les binaires whisper-cli + DLLs dans data_dir.
pub fn download_all_binaries<F>(manifest: &Manifest, data_dir: &Path, mut on_progress: F) -> Result<()>
where
    F: FnMut(&str, u64, u64), // (filename, downloaded, total)
{
    for (name, entry) in &manifest.binaries {
        let dest = data_dir.join(name);
        if dest.exists() {
            continue;
        }
        let name_clone = name.clone();
        download_file(&entry.url, &entry.sha256, entry.size_bytes, &dest, |dl, total| {
            on_progress(&name_clone, dl, total);
        })?;
    }
    Ok(())
}

// ── Détection GPU NVIDIA ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct NvidiaInfo {
    pub name: String,
    pub vram_mb: u32,
    /// True si la carte est assez récente et puissante pour Whisper large
    pub capable: bool,
}

/// Appelle nvidia-smi pour détecter le GPU. Retourne None si absent ou trop vieux.
pub fn detect_nvidia() -> Option<NvidiaInfo> {
    let out = std::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,memory.total",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()?;

    if !out.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&out.stdout);
    let line = text.lines().next()?;
    let parts: Vec<&str> = line.splitn(2, ',').map(str::trim).collect();
    if parts.len() < 2 {
        return None;
    }

    let name = parts[0].to_string();
    let vram_mb: u32 = parts[1].parse().unwrap_or(0);

    let is_modern = name.contains("RTX")
        || name.contains("GTX 16")
        || name.contains("GTX 10")
        || name.contains("A100")
        || name.contains("A4000")
        || name.contains("A5000")
        || name.contains("A6000")
        || name.contains("Tesla");

    let capable = is_modern && vram_mb >= 4096;

    Some(NvidiaInfo { name, vram_mb, capable })
}
