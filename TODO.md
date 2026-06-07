# TODO — Dictum

## v0.3.0 — État actuel (stable)

### Infra
- [x] Manifest JSON `cdn.breizhzion.com/dictum/manifest.json`
- [x] SHA256 modèles + binaires
- [x] `whisper-cli.exe` + DLLs sur CDN
- [x] CI/CD GitHub Actions — release auto sur tag `v*`
- [x] Inno Setup — `Dictum-Setup-x.x.x-x64.exe`
- [x] Auto-update silencieux (GitHub releases API)

### Core
- [x] Hotkey global hold-to-record (F1-F12, NumPad, Insert, Home, End...)
- [x] Enregistrement CPAL 16kHz mono f32
- [x] Transcription whisper-cli subprocess (timeout 5min)
- [x] Injection texte via enigo (SendInput Win32)
- [x] Détection silence RMS configurable
- [x] Durée minimale enregistrement configurable
- [x] Beep audio configurable (beep Windows)
- [x] Pause médias automatique (VK_MEDIA_PLAY_PAUSE)

### Qualité texte
- [x] Majuscule auto première lettre
- [x] Typographie française (espaces insécables, apostrophe U+2019, `…`)
- [x] Substitutions case-insensitive
- [x] Préfixe espace optionnel (`prefix_space`)
- [x] Normalisation espaces multiples
- [x] Filtre tags Whisper ([BLANK_AUDIO], (Music), timestamps SRT)

### Tray
- [x] Icône 3 états (bleu/orange/rouge)
- [x] 14 items de menu
- [x] Tooltip dynamique (modèle + compteur session)
- [x] Confirmation OUI/NON avant mise à jour
- [x] Export historique avec ouverture Notepad
- [x] Reload config sans redémarrage
- [x] Reset config aux valeurs par défaut

### CLI
- [x] `--version`, `--help`, `--debug`
- [x] `fichier.wav` — transcription fichier
- [x] `--language`, `--model`, `--output`
- [x] `--quiet`, `--no-save`, `--stdout`
- [x] `--config`, `--list-devices`, `--list-languages`
- [x] Support WAV stéréo → mono mix-down
- [x] Warning si sample rate non 16kHz

### Robustesse
- [x] Config sanitize() avec logs corrections
- [x] Vérif connectivité internet avant download
- [x] Reprise téléchargement interrompu (HTTP Range)
- [x] Skip si fichier déjà complet
- [x] Retry x3 check update (backoff exponentiel)
- [x] Ignore pre-releases/drafts GitHub
- [x] Vérif taille installateur avant lancement
- [x] Timeout subprocess whisper-cli (5 min)
- [x] Fallback env_logger si flexi_logger échoue

## v0.5 — État actuel (stable et expressif)

### Config API — 25+ champs, 15+ helpers
- [x] `model_name()`, `hotkey_string()`, `language_display()`
- [x] `silence_level_label()`, `beep_description()`, `whisper_speed_label()`
- [x] `inject_mode_label()`, `is_fully_ready()`, `reset_to_default()`
- [x] `log_summary()`, `models_dir()`, `app_version()`

### History API
- [x] `len()`, `is_empty()`, `last_text()`, `last_timestamp()`
- [x] `total_chars()`, `average_length()`, `all_texts()`
- [x] `get_by_index()`, `push_with_limit()`, `export_to_file()`

### Qualité code
- [x] Zéro warning `cargo build`
- [x] Tous les logs en français
- [x] Code entièrement refactorisé avec helpers

## v1.0 — RELEASE STABLE (atteint 2026-06-07)

- [x] CLI complet (13 commandes)
- [x] Tray 4 états icône + mode pause
- [x] 188 méthodes API (Config 138 + History 50)
- [x] Rotation log automatique
- [x] Filtre hallucinations bigrammes
- [x] Notification toast mise à jour
- [x] Fallback clipboard injection
- [x] Typographie française chevrons

## v1.1 — Atteint 2026-06-07

- [x] CLI complet (25+ commandes)
- [x] Mode batch (--batch)
- [x] Test micro (--test-mic)
- [x] Gestion substitutions CLI (--add-sub, --rm-sub, --list-subs)
- [x] Profiles CLI (--profile)
- [x] Tray stats session

## v1.2 — Prochaines étapes

- [ ] Tester l'application sur une machine fraîche (wizard + dictée)
- [x] Fenêtre paramètres graphique egui (remplace Notepad)
- [ ] Traduction automatique locale
- [ ] Reformulation IA 7 styles
- [ ] Plugin VS Code

## v0.6 — Roadmap (intégré dans v0.8)

- [ ] Fenêtre paramètres graphique (egui) — remplace Notepad
- [x] Traduction automatique locale
- [ ] Reformulation IA 7 styles
- [x] Transcription drag & drop fichier audio/vidéo
- [ ] Mode sélection (reformule texte sélectionné)
- [ ] Moteur Parakeet (600 MB, plus rapide)
- [x] Support CUDA via feature flag
- [x] Rotation automatique fichier log
- [x] Notification Windows toast (au lieu de dialog bloquant)
- [ ] Configuration hotkey live (sans redémarrage)

## v1.0 — Idées futures

- [ ] Mode live (streaming temps réel)
- [ ] Plugin VS Code
- [ ] API locale HTTP pour intégrations tierces
- [ ] Identification locuteurs (diarisation)
- [ ] Interface multi-langue
