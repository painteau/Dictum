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

/// Vérifie rapidement la connectivité internet (HEAD sur le manifest).
/// Retourne le User-Agent utilisé pour toutes les requêtes HTTP.
#[allow(dead_code)]
pub fn user_agent() -> String {
    format!("Dictum/{} (Windows)", env!("CARGO_PKG_VERSION"))
}

pub fn has_internet() -> bool {
    reqwest::blocking::Client::builder()
        .user_agent(format!("Dictum/{}", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map(|c| c.head(MANIFEST_URL).send().map(|r| r.status().is_success()).unwrap_or(false))
        .unwrap_or(false)
}

pub fn fetch_manifest() -> Result<Manifest> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(format!("Dictum/{}", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| anyhow!("Client HTTP manifest : {e}"))?;

    let resp = client.get(MANIFEST_URL).send()
        .map_err(|e| anyhow!("Impossible de récupérer le manifest : {e}"))?;

    if !resp.status().is_success() {
        return Err(anyhow!("Manifest HTTP {} — URL : {}", resp.status(), MANIFEST_URL));
    }

    resp.json::<Manifest>()
        .map_err(|e| anyhow!("Format manifest invalide : {e}"))
}

/// Télécharge un modèle avec callback de progression (downloaded, total).
/// Supporte la reprise via HTTP Range si le fichier partiel existe.
/// Vérifie le SHA256 après téléchargement complet.
pub fn download_model<F>(entry: &ModelEntry, dest: &Path, mut on_progress: F) -> Result<()>
where
    F: FnMut(u64, u64),
{
    std::fs::create_dir_all(dest.parent().unwrap())?;

    // Reprise si fichier partiel existant
    let filename = dest.file_name().and_then(|n| n.to_str()).unwrap_or("?");
    log::info!("Téléchargement : {}", filename);

    let existing_bytes = if dest.exists() {
        let size = std::fs::metadata(dest).map(|m| m.len()).unwrap_or(0);
        // Si le fichier est déjà complet, skip direct
        if entry.size_bytes > 0 && size >= entry.size_bytes {
            log::info!("Skip {} — déjà complet ({} MB)", filename, size / 1_048_576);
            on_progress(entry.size_bytes, entry.size_bytes);
            return Ok(());
        }
        size
    } else {
        0
    };

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| anyhow!("Client HTTP : {e}"))?;

    let mut request = client.get(&entry.url);
    if existing_bytes > 0 {
        log::info!("Reprise téléchargement depuis {} MB", existing_bytes / 1_048_576);
        request = request.header("Range", format!("bytes={}-", existing_bytes));
    }

    let mut resp = request.send()
        .map_err(|e| anyhow!("Téléchargement échoué : {e}"))?;

    if !resp.status().is_success() && resp.status().as_u16() != 206 {
        return Err(anyhow!("Erreur HTTP {} lors du téléchargement", resp.status()));
    }

    // Vérifier Content-Length si disponible
    if let Some(content_len) = resp.headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
    {
        let expected = if existing_bytes > 0 { entry.size_bytes.saturating_sub(existing_bytes) } else { entry.size_bytes };
        if expected > 0 && (content_len as i64 - expected as i64).abs() > 1024 {
            log::warn!("Content-Length ({}) différent de la taille attendue ({})", content_len, expected);
        }
    }

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(existing_bytes > 0)
        .write(true)
        .open(dest)
        .map_err(|e| anyhow!("Impossible d'ouvrir le fichier : {e}"))?;

    let mut hasher = Sha256::new();
    let mut downloaded = existing_bytes;
    let total = entry.size_bytes;
    let mut last_log_pct = 0u64;
    let dl_start = std::time::Instant::now();
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
        // Log tous les 10% avec débit
        if total > 0 {
            let pct = downloaded * 100 / total;
            if pct >= last_log_pct + 10 {
                last_log_pct = pct;
                let elapsed = dl_start.elapsed().as_secs_f64().max(0.001);
                let speed_mb = (downloaded - existing_bytes) as f64 / 1_048_576.0 / elapsed;
                log::info!("Téléchargement : {}% ({:.0}/{:.0} MB) à {:.1} MB/s",
                    pct,
                    downloaded as f64 / 1_048_576.0,
                    total as f64 / 1_048_576.0,
                    speed_mb
                );
            }
        }
    }

    let elapsed_secs = dl_start.elapsed().as_secs_f64().max(0.001);
    let total_mb = (downloaded - existing_bytes) as f64 / 1_048_576.0;
    let speed_mb = total_mb / elapsed_secs;
    log::info!("Téléchargement terminé : {:.1} MB en {:.0}s ({:.1} MB/s)", total_mb, elapsed_secs, speed_mb);

    let actual_hash = hex::encode(hasher.finalize());
    if !entry.sha256.is_empty() {
        if actual_hash != entry.sha256 {
            std::fs::remove_file(dest).ok();
            return Err(anyhow!(
                "Checksum invalide !\nAttendu  : {}\nObtenu   : {}",
                entry.sha256,
                actual_hash
            ));
        }
        log::info!("SHA256 vérifié ✓ : {}", &actual_hash[..16]);
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
