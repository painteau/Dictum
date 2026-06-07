use serde::Deserialize;
use anyhow::{anyhow, Result};

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const GITHUB_REPO: &str = "painteau/Dictum";

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
    assets: Vec<GithubAsset>,
    prerelease: bool,
    draft: bool,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Debug)]
pub struct UpdateInfo {
    pub version: String,
    pub url: String,
    pub installer_url: String,
    pub installer_size: u64,
}

/// Vérifie si une nouvelle version est disponible sur GitHub.
/// Retourne None si déjà à jour ou en cas d'erreur réseau (silencieux).
pub fn check_update() -> Option<UpdateInfo> {
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        GITHUB_REPO
    );

    let client = reqwest::blocking::Client::builder()
        .user_agent(format!("Dictum/{}", CURRENT_VERSION))
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;

    // Retry 3 fois avec délai exponentiel
    let release: GithubRelease = {
        let mut result = None;
        for attempt in 0..3u32 {
            match client.get(&url).send().and_then(|r| r.json::<GithubRelease>()) {
                Ok(r) => { result = Some(r); break; }
                Err(e) => {
                    log::debug!("Tentative {}/3 check update échouée : {e}", attempt + 1);
                    if attempt < 2 {
                        std::thread::sleep(std::time::Duration::from_secs(2u64.pow(attempt)));
                    }
                }
            }
        }
        result?
    };

    // Ignorer les pre-releases et drafts
    if release.prerelease || release.draft {
        log::debug!("Release {} ignorée (prerelease/draft)", release.tag_name);
        return None;
    }

    let latest = release.tag_name.trim_start_matches('v');
    let current = CURRENT_VERSION;

    log::debug!("Version actuelle : v{} | Dernière : v{}", current, latest);

    if !is_newer(latest, current) {
        log::info!("Dictum à jour (v{})", current);
        return None;
    }

    log::info!("Mise à jour disponible : v{} → v{}", current, latest);

    // Cherche Dictum-Setup-*.exe dans les assets
    let installer = release.assets.iter().find(|a| {
        a.name.starts_with("Dictum-Setup") && a.name.ends_with(".exe")
    })?;

    Some(UpdateInfo {
        version: latest.to_string(),
        url: release.html_url,
        installer_url: installer.browser_download_url.clone(),
        installer_size: installer.size,
    })
}

/// Télécharge le nouvel installateur dans %TEMP% et le lance en /SILENT.
/// L'installateur Inno Setup gère la mise à jour par-dessus l'existant.
pub fn apply_update(info: &UpdateInfo) -> Result<()> {
    // Nettoyer les anciens installateurs dans %TEMP%
    if let Ok(entries) = std::fs::read_dir(std::env::temp_dir()) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let s = name.to_string_lossy();
            if s.starts_with("Dictum-Setup-") && s.ends_with(".exe") && !s.contains(&info.version) {
                std::fs::remove_file(entry.path()).ok();
                log::debug!("Ancienne install supprimée : {}", s);
            }
        }
    }

    let dest = std::env::temp_dir()
        .join(format!("Dictum-Setup-{}.exe", info.version));

    let size_mb = (info.installer_size as f64 / 1_048_576.0).ceil() as u64;
    log::info!("Téléchargement mise à jour v{} ({} MB) → {}", info.version, size_mb, dest.display());

    let mut resp = reqwest::blocking::get(&info.installer_url)
        .map_err(|e| anyhow!("Téléchargement échoué : {e}"))?;

    if !resp.status().is_success() {
        return Err(anyhow!("HTTP {} lors du téléchargement", resp.status()));
    }

    {
        use std::io::{Read, Write};
        let mut file = std::fs::File::create(&dest)
            .map_err(|e| anyhow!("Impossible de créer le fichier : {e}"))?;
        let mut buf = vec![0u8; 65_536];
        let mut written = 0u64;
        let mut last_pct = 0u64;
        loop {
            let n = resp.read(&mut buf).map_err(|e| anyhow!("Erreur lecture : {e}"))?;
            if n == 0 { break; }
            file.write_all(&buf[..n]).map_err(|e| anyhow!("Erreur écriture : {e}"))?;
            written += n as u64;
            if info.installer_size > 0 {
                let pct = written * 100 / info.installer_size;
                if pct >= last_pct + 25 {
                    last_pct = pct;
                    log::info!("Setup téléchargement : {}% ({:.1}/{:.1} MB)", pct,
                        written as f64 / 1_048_576.0, info.installer_size as f64 / 1_048_576.0);
                }
            }
        }
    }

    // Vérifier taille du fichier téléchargé
    let downloaded_size = std::fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);
    if info.installer_size > 0 && downloaded_size < info.installer_size / 2 {
        std::fs::remove_file(&dest).ok();
        return Err(anyhow!("Fichier téléchargé tronqué ({} / {} bytes)", downloaded_size, info.installer_size));
    }

    log::info!("Lancement installateur silencieux ({} MB)", downloaded_size / 1_048_576);

    std::process::Command::new(&dest)
        .args(["/SILENT", "/CLOSEAPPLICATIONS"])
        .spawn()
        .map_err(|e| anyhow!("Impossible de lancer l'installateur : {e}"))?;

    // Quitter pour laisser l'installateur remplacer l'exe
    std::process::exit(0);
}

/// Compare deux versions semver simplement (ex: "0.2.0" > "0.1.1").
fn is_newer(candidate: &str, current: &str) -> bool {
    let parse = |v: &str| -> (u32, u32, u32) {
        let parts: Vec<u32> = v.split('.').filter_map(|p| p.parse().ok()).collect();
        (
            parts.first().copied().unwrap_or(0),
            parts.get(1).copied().unwrap_or(0),
            parts.get(2).copied().unwrap_or(0),
        )
    };
    parse(candidate) > parse(current)
}
