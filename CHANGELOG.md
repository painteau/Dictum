# CHANGELOG — Dictum

## 2026-06-06 — v0.1.3 — Auto-update

### Ajouté
- `src/updater.rs` — check silencieux au démarrage via GitHub API, comparaison semver, téléchargement `Dictum-Setup.exe` dans `%TEMP%`, lancement `/SILENT /CLOSEAPPLICATIONS`
- `tray.rs` — item "Mise à jour disponible" apparaît dynamiquement dans le menu tray quand une nouvelle version est détectée
- `main.rs` — thread arrière-plan check update au démarrage, stockage via `Mutex<Option<UpdateInfo>>`
- `installer.iss` — script Inno Setup : installe dans `Program Files`, raccourci Startup coché par défaut, désinstallateur propre
- `release.yml` — étape Inno Setup ajoutée, publie `Dictum-Setup-*.exe` en plus de `dictum.exe`

### Comportement mise à jour
1. Dictum démarre, thread background check GitHub releases API
2. Si version plus récente : item tray s'active
3. Clic → télécharge `Dictum-Setup-x.x.x.exe` → lance `/SILENT` → Dictum se ferme → installateur remplace l'exe → redémarre

---

## 2026-06-06 — v0.1.2 — CI/CD + release GitHub

### Ajouté
- `.github/workflows/release.yml` — GitHub Actions : compile automatiquement sur tag `v*` et publie `dictum.exe` en asset de release
- Première release publique `v0.1.1` sur `github.com/painteau/Dictum/releases`
- `ARCHITECTURE.md` — doc complète threads, flux de données, CDN, décisions d'architecture

### Workflow release
```powershell
git tag v0.x.x
git push origin v0.x.x
```
GitHub compile sur `windows-latest`, build release Rust, crée la release avec `dictum.exe` automatiquement.

---

## 2026-06-06 — v0.1.1 — Setup wizard + téléchargement modèles

### Ajouté
- `src/downloader.rs` — manifest JSON distant (URL modifiable sans recompiler), téléchargement modèle avec progression, vérification SHA256, détection GPU NVIDIA via `nvidia-smi`
- `src/setup.rs` — wizard egui premier lancement : détection GPU automatique, choix qualité (medium/large), langue, hotkey, barre de progression téléchargement
- `.gitignore` — exclusion `*.bin`, `*.gguf`, `models/`, `target/`
- Dépendances ajoutées : `eframe 0.27`, `egui 0.27`, `reqwest 0.11` (blocking), `sha2 0.10`, `hex 0.4`
- `main.rs` — détection premier lancement, appel wizard si modèle absent

### Logique wizard
- GPU détecté automatiquement (nvidia-smi) → modèle large-v3 pré-sélectionné si VRAM >= 4 GB
- 2 questions utilisateur seulement : qualité + langue
- Hotkey configurable dans le wizard (touches F1..F12, modificateurs)
- Typographie française auto-activée si langue = `fr`

---

## 2026-06-06 — v0.1.0 — Fondations

### Ajouté
- Structure projet Rust (`cargo new`)
- `Cargo.toml` avec dépendances : `whisper-rs`, `cpal`, `rdev`, `enigo`, `tray-icon`, `crossbeam-channel`, `serde_json`, `winapi`
- `src/main.rs` — orchestration multi-thread, `AppState` partagé via `Arc<Mutex>`
- `src/config.rs` — `Config` sérialisée en JSON dans `%LOCALAPPDATA%\Dictum\config.json`, auto-générée au premier lancement
- `src/audio.rs` — capture microphone 16 kHz mono f32 via CPAL, stream isolé dans son propre thread (contournement `Stream: !Send`)
- `src/transcribe.rs` — inférence Whisper locale via `whisper-rs`, support 99 langues, détection automatique
- `src/inject.rs` — injection texte au curseur via `enigo`, typographie française (espaces insécables), capitalisation automatique, option Auto-Enter
- `src/hotkey.rs` — écoute globale clavier via `rdev` (thread dédié), support modificateurs Ctrl/Alt/Shift, hold-to-record
- `src/history.rs` — historique des 10 dernières transcriptions, persisté en JSON
- `src/substitution.rs` — remplacement d'abréviations et corrections personnalisées
- `src/tray.rs` — icône barre système, menu contextuel (Paramètres, Historique, Microphones, Quitter), message pump Windows natif, icône bleue/rouge selon état
- `README.md` — doc complète : prérequis, compilation, configuration, architecture, roadmap
- `TODO.md` — liste des tâches v1/v2/v3
- Suppression console en mode release (`#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`)

### Architecture
- Thread 1 (principal) : tray + message pump Windows
- Thread 2 : `rdev` hotkey listener (bloquant)
- Thread 3 : pipeline record/transcribe/inject
- Thread 4 : cpal audio stream (propre à chaque enregistrement)
