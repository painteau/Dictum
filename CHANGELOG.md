# CHANGELOG — Dictum

## 2026-06-07 — v0.2.3 — Robustesse

### Ajouté
- `transcribe.rs` : timeout 5 min subprocess whisper-cli, kill si dépassé
- `updater.rs` : retry x3 avec backoff exponentiel (1s, 2s) si réseau indisponible
- `updater.rs` : ignore pre-releases et drafts GitHub
- `inject.rs` : injection multi-lignes (segments Whisper avec `\n`)
- CLI : flag `--language fr` pour transcription fichier
- `.github/SECURITY.md` : politique de sécurité
- README : section Mode CLI complète

### Corrigé
- `inject.rs` : message d'erreur en français

---

## 2026-06-07 — v0.2.2 — Finitions nuit

### Ajouté
- `inject.rs` : apostrophe typographique `'` → `'` (U+2019), points de suspension `...` → `…` (U+2026)
- `downloader.rs` : reprise téléchargement interrompu via HTTP Range, timeout 10min, client configuré
- Template Pull Request GitHub
- CI : cache LLVM entre les builds, timeout 30min, restore-keys Cargo
- `max_history` configurable dans config.json (défaut 10)

### Corrigé
- `transcribe.rs` : suppression `--output-txt`, nettoyage fichiers `.txt` sidecar whisper-cli
- `transcribe.rs` : `current_dir(data_dir)` pour que Windows trouve `ggml.dll` et DLLs
- `history.rs` : `push_with_limit()` respecte `config.max_history`

---

## 2026-06-07 — v0.2.1 — Polish nuit

### Ajouté
- Touches hotkey supplémentaires : Insert, Home, End, PageUp, PageDown, ScrollLock, Pause, CapsLock, BackQuote
- Wizard : affiche toutes les nouvelles touches
- Tray : Réinitialiser config aux valeurs par défaut
- Tray : icône **3 états** — bleu (repos), orange (transcription), rouge (enregistrement)
- `CONTRIBUTING.md` — guide de contribution, structure projet, conventions
- Templates GitHub Issues (bug report, feature request)
- `LICENSE` MIT
- Badges CI + release dans README
- Mode CLI `--version`, `--help`
- Mode CLI transcription fichier audio (`dictum.exe fichier.wav`)
- Substitutions `case_insensitive` optionnel

### Corrigé
- Transcription vide (silence) ignorée : pas injectée, pas dans l'historique
- Indentation pipeline transcription dans `main.rs`
- Messages logs en français
- Version `Cargo.toml` = `0.2.0` alignée sur les releases

---

## 2026-06-07 — v0.2.0 — Milestone nuit

### Ajouté
- Mode CLI : `dictum.exe fichier.wav` → transcription + sauvegarde `.txt`
- Menu tray : Copier dernière dictée dans le presse-papiers (arboard)
- `src/media.rs` : toggle VK_MEDIA_PLAY_PAUSE, pause/reprise médias automatique
- Horodatage HH:MM sur chaque entrée d'historique
- Menu tray : Effacer historique, Recharger config, Ouvrir log, À propos (version + modèle)
- `config.rs` : `beep_enabled`, `silence_threshold`, `pause_media`
- Log fichier `%LOCALAPPDATA%\Dictum\dictum.log` via `flexi_logger`
- Détection silence RMS avant transcription (évite appels Whisper inutiles)
- `src/updater.rs` : auto-update silencieux au démarrage
- `build.rs` : icône .ico générée, version info Windows embarquée
- Icône tray dynamique (bleu/rouge selon état enregistrement)

---

## 2026-06-07 — v0.1.9 — Nuit de polish

### Ajouté
- `src/media.rs` — pause/reprise médias automatique via VK_MEDIA_PLAY_PAUSE (configurable `pause_media`)
- `src/history.rs` — horodatage HH:MM sur chaque entrée de l'historique
- `tray.rs` — menus : Effacer historique, Recharger config, Ouvrir log, À propos (version + modèle)
- `config.rs` — nouveaux champs : `silence_threshold` (0.005), `beep_enabled`, `pause_media`
- `transcribe.rs` — détection silence RMS configurable, log debug si silence ignoré
- `audio.rs` — beep Windows (Beep API kernel32), timeout `max_record_secs` via `recv_timeout`
- `build.rs` — génère icône `.ico` 32x32 programmatiquement, embed version info via `winresource`
- Log fichier `dictum.log` dans `%LOCALAPPDATA%\Dictum\` via `flexi_logger`
- `updater.rs` — auto-update silencieux : check GitHub releases, télécharge `Dictum-Setup.exe`, lance `/SILENT`

### Corrigé
- `setup.rs` — config sauvegardée sur disque avant fermeture du wizard
- `setup.rs` — `needs_setup` vérifie maintenant `whisper-cli.exe` ET le modèle
- `tray.rs` — icône tray change dynamiquement (bleu → rouge pendant enregistrement)
- `Cargo.toml` — suppression `const_format` inutilisé, version alignée `0.1.6`

---

## 2026-06-07 — v0.1.7 — Icône exe + polish

### Ajouté
- `build.rs` — génère un `.ico` 32x32 (cercle bleu acier) en mémoire au build, l'embed dans l'exe via `winresource` (version info + icône visible dans l'Explorateur Windows)
- `[build-dependencies] winresource` dans `Cargo.toml`

### Corrigé
- `setup.rs` — le wizard sauvegarde maintenant la config sur disque avant de fermer (fix: config non persistée)
- `Cargo.toml` — version alignée sur la release GitHub (`0.1.6`)
- `README.md` — refonte complète : architecture subprocess whisper-cli, instructions install/build, roadmap

---

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
