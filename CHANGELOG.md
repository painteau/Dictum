# CHANGELOG — Dictum

## 2026-06-07 — v0.5.4 — Profils et résumés

### Ajouté
- `config.rs` : `profile_name()` — "Français standard", "Minimal", "Personnalisé"
- `config.rs` : `whisper_config_display()` — résumé config whisper complet
- `config.rs` : `log_summary()` affiche le profil
- `tray.rs` : À propos affiche le profil
- `main.rs` : utilise `whisper_config_display()`

---

## 2026-06-07 — v0.5.3 — History API avancée

### Ajouté
- `history.rs` : `search()` — recherche case-insensitive
- `history.rs` : `filter_by_min_length()` — filtre par longueur minimale
- `config.rs` : `validate()` vérifie extension modèle et temperature élevée
- `config.rs` : `is_ready_message()` — message lisible état Dictum
- `main.rs` : log `is_ready_message()` au démarrage

---

## 2026-06-07 — v0.5.2 — Validation et diagnostics

### Ajouté
- `config.rs` : `validate()` — liste des problèmes de config
- `config.rs` : `full_status()` — rapport complet 2 lignes
- `tray.rs` : À propos affiche `validate()` + `full_status()`
- `main.rs` : `validate()` au démarrage avec log warn
- `config.rs` : `threads_display()`, `record_duration_label()`
- `history.rs` : `last_timestamp()`, `total_chars()`, `average_length()`

---

## 2026-06-07 — v0.5.1 — Helpers finaux

### Ajouté
- `config.rs` : `threads_display()` — "auto (max 8)" ou "4"
- `config.rs` : `record_duration_label()` — "300ms–30s"
- `config.rs` : `log_summary()` enrichi avec durée enregistrement et threads
- `history.rs` : `last_timestamp()`, `total_chars()`, `average_length()`
- `tray.rs` : À propos affiche stats historique (nb + total chars)
- `tray.rs` : utilise `threads_display()`
- `main.rs` : log historique avec stats complets
- `main.rs` : log enregistrement utilise `record_duration_label()`

---

## 2026-06-07 — v0.5.0 — Config API complète

### Milestone
API Config atteint maturité. 25+ champs, 15+ helpers.

### Helpers Config disponibles
- `model_name()` — nom court du modèle
- `hotkey_string()` — combo "Ctrl+F9"
- `language_display()` — "Français", "Auto-détection"
- `silence_level_label()` — "normal", "élevé"
- `beep_description()` — "800Hz/600Hz 80ms"
- `whisper_speed_label()` — "normal", "lent"
- `inject_mode_label()` — "majuscule+typo_fr"
- `is_model_ready()`, `is_whisper_cli_ready()`, `is_fully_ready()`
- `has_substitutions()`, `substitution_count()`
- `models_dir()`, `data_dir()`, `log_path()`, `history_export_path()`
- `log_summary()` — résumé compact tout-en-un
- `reset_to_default()` — réinit avec log
- `app_version()` — version statique

---

## 2026-06-07 — v0.4.9 — Helpers expressifs

### Ajouté
- `config.rs` : `silence_level_label()` — "normal", "élevé", "désactivé"...
- `config.rs` : `beep_description()` — "800Hz/600Hz 80ms" ou "désactivé"
- `config.rs` : `language_display()` — "Français", "English", "Auto-détection"...
- `config.rs` : `log_summary()` enrichi avec silence level et beep
- `transcribe.rs` : log RMS avec silence_level_label()
- `main.rs` : log beep_description() au démarrage

---

## 2026-06-07 — v0.4.8 — API finale

### Ajouté
- `config.rs` : `language_display()` — nom lisible de la langue (fr→Français, auto→Auto-détection...)
- `config.rs` : `is_fully_ready()` — vérifie modèle + whisper-cli
- `config.rs` : `sanitize()` logge si silence_threshold hors limites
- `tray.rs` : À propos affiche `language_display()` au lieu du code ISO
- `config.rs` : `log_summary()` utilise `language_display()`
- `main.rs` : utilise `is_fully_ready()` au lieu de `transcribe::is_ready()`
- `transcribe.rs` : log nom du modèle par transcription

---

## 2026-06-07 — v0.4.7 — Code propre

### Ajouté
- `config.rs` : `model_name()` helper pour nom court du modèle
- `transcribe.rs` : log modèle actif dans chaque transcription
- `tray.rs` : À propos utilise `model_name()`
- `main.rs` : utilise `model_name()` et `is_model_ready()`

### Zéro warning `cargo build`

---

## 2026-06-07 — v0.4.6 — Refactoring API

### Ajouté
- `config.rs` : `model_name()`, `hotkey_string()`, `models_dir()`, `substitution_count()`, `has_substitutions()`
- `setup.rs` : utilise `Config::models_dir()`
- `hotkey.rs` : utilise `config.hotkey_string()`
- `tray.rs` : utilise `hotkey_string()` dans À propos et tooltip
- `config.rs` : `log_summary()` utilise `model_name()` et `hotkey_string()`
- `main.rs` : utilise `has_substitutions()` et `substitution_count()`

---

## 2026-06-07 — v0.4.5 — Finitions API

### Ajouté
- `config.rs` : `log_summary()` affiche les flags actifs (beep, typo_fr, auto_enter...)
- `config.rs` : sanitize `log_level` invalide avec warn
- `tray.rs` : Ouvrir le log respecte `EDITOR` env var
- `tray.rs` : À propos affiche état modèle et whisper-cli (✓/✗)

---

## 2026-06-07 — v0.4.4 — API Config enrichie

### Ajouté
- `config.rs` : `is_model_ready()`, `is_whisper_cli_ready()`, `reset_to_default()`
- `transcribe.rs` : `is_ready()` utilise les nouveaux helpers avec logs détaillés
- `setup.rs` : `needs_setup()` utilise les helpers, log ce qui manque
- `tray.rs` : utilise `Config::reset_to_default()`
- `main.rs` : log arrêt enrichi (transcriptions + historique)
- `audio.rs` : messages `expect` en français
- `tray.rs` : message `expect` icône en français

---

## 2026-06-07 — v0.4.3 — Logs 100% français

### Corrigé
- Tous les messages log traduits en français
- `audio.rs` : "Audio stream error" → "Erreur stream audio"
- `hotkey.rs` : "Hotkey listener error" → "Erreur listener hotkey"
- `main.rs` : "Recording started/too short/Failed" → français
- `main.rs` : "Busy transcribing" → "Transcription en cours"

### Ajouté
- `transcribe.rs` : log debug chemin WAV temp
- `inject.rs` : log debug Auto-Enter déclenché
- `setup.rs` : log info/error fin téléchargement wizard
- `updater.rs` : log fermeture avant mise à jour
- `downloader.rs` : log SHA256 vérifié

---

## 2026-06-07 — v0.4.2 — Diagnostics complets

### Ajouté
- `transcribe.rs` : log RMS calculé et seuil silence
- `transcribe.rs` : log debug stderr whisper-cli si non vide
- `transcribe.rs` : warn si transcription vide après filtrage
- `updater.rs` : log progression Setup tous les 25%
- `downloader.rs` : log fin téléchargement avec débit moyen
- `main.rs` : log nb samples avant transcription
- `main.rs` : log info si résultat vide
- `audio.rs` : log debug config stream audio
- `tray.rs` : log réinitialisation config
- `history.rs` : log debug ajout entrée historique
- `config.rs` : `log_summary()` utilisé au démarrage et reload
- `substitution.rs` : early return si pas de règles

---

## 2026-06-07 — v0.4.1 — Logs exhaustifs

### Ajouté
- `hotkey.rs` : log info à chaque press et release
- `inject.rs` : log info après injection réussie
- `audio.rs` : log micro actif en info
- `config.rs` : `log_summary()` résumé compact
- `main.rs` : `log_summary()` au démarrage et après reload
- `substitution.rs` : early return si aucune règle
- `downloader.rs` : log Skip avec taille MB
- `transcribe.rs` : log preview 50 chars et facteur temps réel

---

## 2026-06-07 — v0.4.0 — Milestone matin

### Milestone
Config complète avec 20+ champs. Whisper optimisé. Logs exhaustifs.

### Depuis v0.3.9
- `config.rs` : `log_level` configurable (error/warn/info/debug/trace)
- `config.rs` : sanitize valide `log_level`
- `main.rs` : log niveau de log actif au démarrage
- `main.rs` : log_level lu depuis config avant init_logger
- `tray.rs` : log debug event id menu tray
- `audio.rs` : log taille samples KB
- `transcribe.rs` : timeout adaptatif vraiment utilisé dans subprocess
- `transcribe.rs` : `transcribe_start` correctement nommé
- `substitution.rs` : limite 100 règles max
- `downloader.rs` : `user_agent()` helper, User-Agent cohérent

### Config v0.4 — 22 champs configurables
language, hotkey, model_path, auto_enter, french_typography, auto_capitalize,
substitutions, microphone, max_record_secs, min_record_ms, max_history,
beep_enabled, beep_start_freq, beep_end_freq, beep_duration_ms, silence_threshold,
pause_media, prefix_space, whisper_threads, whisper_temperature, inject_delay_ms,
whisper_no_speech, log_level

---

## 2026-06-07 — v0.3.9 — Finitions 1h du matin

### Ajouté
- `history.rs` : export format Markdown avec en-tête
- `substitution.rs` : limite 100 règles max
- `inject.rs` : log debug longueur et flags avant injection
- `downloader.rs` : `user_agent()` helper, User-Agent propre dans `has_internet()`
- `config.rs` : `Config::app_version()` helper statique
- `transcribe.rs` : fallback si whisper-cli retourne erreur mais stdout non vide
- `updater.rs` : log version actuelle vs dernière lors du check
- `transcribe.rs` : log info threads/temp/lang par transcription

---

## 2026-06-07 — v0.3.8 — Nettoyage output Whisper

### Ajouté
- `transcribe.rs` : `strip_ansi()` supprime codes ESC résiduels
- `transcribe.rs` : normalisation CRLF→LF stdout Windows
- `transcribe.rs` : `--print-special false`, `--word-thold 0.01`
- `transcribe.rs` : log info whisper threads/temp/lang par transcription
- `updater.rs` : log version actuelle vs dernière
- `config.rs` : `Config::app_version()` helper statique

---

## 2026-06-07 — v0.3.7 — Config audio complète

### Ajouté
- `config.rs` : `beep_start_freq` (800Hz), `beep_end_freq` (600Hz), `beep_duration_ms` (80ms)
- `config.rs` : sanitize valide fréquences (100-20000Hz) et durée (max 2000ms)
- `transcribe.rs` : `--print-colors false` pour éviter codes ANSI dans stdout
- `inject.rs` : `inject_raw()` API publique sans typographie
- README : tous les paramètres beep dans le tableau config

---

## 2026-06-07 — v0.3.6 — Robustesse avancée

### Ajouté
- `downloader.rs` : vérification Content-Length vs taille attendue
- `history.rs` : `get_by_index()`, `all_texts()`, drop explicite lock avant dialog
- `main.rs` : AppEvent::ReloadConfig dans le pipeline
- `tray.rs` : Recharger config notifie le pipeline
- `setup.rs` : URL manifest affichée en cas d'erreur réseau
- `hotkey.rs` : Escape/ESC supporté
- `config.rs` : sanitize whisper_temperature clamp(0, 1)
- `transcribe.rs` : log debug args whisper-cli complets
- `substitution.rs` : log case_insensitive dans debug

---

## 2026-06-07 — v0.3.5 — Whisper avancé

### Ajouté
- `config.rs` : `whisper_temperature` (0.0=déterministe, défaut), `whisper_no_speech`
- `transcribe.rs` : `--temperature`, `--no-speech-thold`, détection hallucination
- `transcribe.rs` : `--best-of 1 --beam-size 1` pour réduire hallucinations
- `audio.rs` : beep double si timeout enregistrement
- `main.rs` : log config complète au démarrage (médias, beep, threads, délai)
- `history.rs` : `last_text()` helper
- `updater.rs` : log taille MB du setup

---

## 2026-06-07 — v0.3.4 — Config avancée

### Ajouté
- `config.rs` : `inject_delay_ms` (défaut 80ms), `whisper_threads` (défaut 0=auto)
- `tray.rs` : À propos affiche nb threads CPU whisper
- `hotkey.rs` : log debug si hotkey pressée sans modificateurs requis
- `main.rs` : log nb substitutions configurées au démarrage
- README : `inject_delay_ms` et `whisper_threads` dans le tableau config

---

## 2026-06-07 — v0.3.3 — Performances

### Ajouté
- `transcribe.rs` : `--threads` auto (nb cœurs CPU, max 8) — accélère whisper-cli
- `setup.rs` : wizard affiche toutes les touches NumPad dans le sélecteur
- `hotkey.rs` : NumLock, KpMinus, KpPlus, KpMultiply, KpDivide, KpReturn
- `main.rs` : log arrêt propre avec bilan session
- `main.rs` : log état modèle (présent/MANQUANT)
- `audio.rs` : log max_secs au démarrage enregistrement
- `downloader.rs` : log débit MB/s
- `config.rs` : `open_data_dir()`, `Config::log_path()`, `Config::history_export_path()`
- `tray.rs` : item "Ouvrir le dossier Dictum"
- `history.rs` : affiche `J-N` pour entrées > 24h

---

## 2026-06-07 — v0.3.2 — Dernières finitions nuit

### Ajouté
- `tray.rs` : item "Ouvrir le dossier Dictum" dans l'explorateur
- `tray.rs` : tooltip affiche la hotkey active et les instructions
- `config.rs` : `open_data_dir()` ouvre le dossier dans l'explorateur
- `config.rs` : `sanitize()` valide l'extension `.bin` du model_path
- `history.rs` : affiche `J-N` pour les entrées de plus de 24h
- `main.rs` : log état modèle (présent/MANQUANT) au démarrage
- `downloader.rs` : log débit en MB/s tous les 10%
- `hotkey.rs` : Backspace, Delete, Enter supportés comme hotkey
- `substitution.rs` : règles triées par longueur (priorité aux longues)
- `audio.rs` : `list_devices()` marque le micro défaut avec `(défaut)`
- `transcribe.rs` : vérif taille WAV avant lancement whisper

### Corrigé
- `updater.rs` : nettoyage anciens Setup-*.exe dans TEMP
- `transcribe.rs` : normalisation espaces multiples

---

## 2026-06-07 — v0.3.1 — Polish final

### Ajouté
- `audio.rs` : `list_devices()` marque le micro défaut avec `(défaut)`
- `substitution.rs` : règles triées par longueur décroissante (priorité aux règles longues)
- `tray.rs` : menu Historique affiche le nombre d'entrées en temps réel
- `inject.rs` : typographie FR étendue (ordre `...`→`…` d'abord, tiret demi-cadratin)
- `config.rs` : champ `config_version` pour migrations futures
- `main.rs` : check config_version, réinit si obsolète
- `history.rs` : `len()` et `is_empty()` publics
- `downloader.rs` : log nom fichier au début de chaque téléchargement

### Corrigé
- Log transcription avec preview 100 chars

---

## 2026-06-07 — v0.3.0 — Stable

### Milestone
Première version considérée stable. Core complet, robuste, CLI riche.

### Ajouté depuis v0.2.9
- `hotkey.rs` : touches NumPad 0-9 supportées
- `tray.rs` : Exporter historique ouvre Notepad automatiquement
- `transcribe.rs` : normalisation espaces multiples dans output Whisper
- `inject.rs` : délai injection adaptatif 60-80ms
- `audio.rs` : log durée enregistrement à l'arrêt
- `updater.rs` : vérif taille installateur avant lancement

### État du projet v0.3.0
**Core :** enregistrement, transcription, injection — stable  
**CLI :** `--version`, `--help`, `--language`, `--model`, `--output`, `--quiet`, `--no-save`, `--stdout`, `--debug`, `--config`, `--list-devices`, `--list-languages`  
**Tray :** 12 items de menu, 3 états icône, tooltip dynamique  
**Config :** 14 champs configurables avec sanitize()  
**Infra :** CI/CD GitHub Actions, auto-update, CDN manifest, Inno Setup  

---

## 2026-06-07 — v0.2.9 — Robustesse réseau

### Ajouté
- `downloader::has_internet()` — vérif connectivité HEAD sur le manifest (timeout 5s)
- `setup.rs` : vérif internet avant téléchargement, message d'erreur explicite
- `main.rs` : check update différé 10s après démarrage, ignoré si hors ligne
- `config.rs` : `sanitize()` logge les corrections avec `log::warn!`
- `tray.rs` : tooltip affiche modèle actif `[ggml-medium]`

---

## 2026-06-07 — v0.2.8 — Zéro warning

### Corrigé
- `transcribe.rs` : variable `start` préfixée via `cargo fix` — zéro warning `cargo build`
- `config.rs` : double `}` parasite supprimé

### Ajouté
- `config.rs` : `log_path()`, `history_export_path()` helpers centralisés
- `tray.rs` : À propos affiche chemin `config.json`
- `tray.rs` : confirmation Oui/Non avant installation mise à jour
- `tray.rs` : Exporter historique en item dédié (sans effacement)
- `config.rs` : `Config::save_to(path)` pour export config custom

---

## 2026-06-07 — v0.2.7 — Finitions finales

### Ajouté
- `history.rs` : `export_to_file()`, export auto avant effacement depuis le tray
- `transcribe.rs` : log durée transcription vs durée audio (facteur temps réel)
- `tray.rs` : item update affiche version + taille MB
- `substitution.rs` : log debug quand règle appliquée
- `downloader.rs` : skip si fichier déjà complet (taille exacte)
- `setup.rs` : titre wizard différent binaires vs modèle
- `config.rs` : `prefix_space` — espace avant injection
- CLI `--stdout` : alias de `--quiet --no-save` pour scripts pipe

### Corrigé
- Aucun warning `cargo build`

---

## 2026-06-07 — v0.2.6 — CLI robuste

### Ajouté
- CLI `--config/-c` : fichier config personnalisé
- CLI `--debug` : active les logs détaillés (debug level)
- CLI `--no-save` : transcription sans créer de fichier `.txt`
- `Config::load_from(path)` pour support config externe
- `audio.rs` : log du device micro actif (niveau debug)
- `tray.rs` : menu réorganisé avec séparateurs logiques
- `history.rs` : texte tronqué à 120 chars dans le dialog
- `transcribe.rs` : filtre numéros de séquence SRT dans output
- `downloader.rs` : log progression tous les 10%
- `config.rs` : `open_in_editor` respecte `EDITOR` env var

### Corrigé
- `init_logger` : fallback `env_logger` si `flexi_logger` échoue
- `hotkey.rs` : message fallback liste les touches valides
- `inject.rs` : trim whitespace, early return si vide

---

## 2026-06-07 — v0.2.5 — Finitions matin

### Ajouté
- `config.rs` : `open_in_editor` respecte la var d'env `EDITOR`, fallback notepad
- `downloader.rs` : log progression tous les 10% (MB / MB)
- `hotkey.rs` : message fallback liste les touches valides
- `inject.rs` : trim whitespace avant injection, early return si vide
- `.cargo/config.toml` : retry réseau x3, jobs parallèles
- `Cargo.toml` + `installer.iss` : version 0.2.4 alignée

---

## 2026-06-07 — v0.2.4 — CLI complet

### Ajouté
- CLI `--model/-m` : forcer un modèle Whisper pour la transcription fichier
- CLI `--output/-o` : choisir le fichier de sortie (défaut : `.txt` à côté du fichier)
- CLI `--list-devices` : liste les microphones disponibles
- CLI `--list-languages` : liste les 57 langues Whisper supportées
- Compteur de transcriptions de session dans le tooltip tray
- `session_count` dans `AppState` (incrémenté à chaque transcription réussie)
- `config.sanitize()` : valeurs invalides corrigées silencieusement au chargement
- Log hotkey active au démarrage du listener
- `audio.rs` : message d'erreur micro avec liste des appareils disponibles

### Corrigé
- Silence CLI : affiche `[Silence détecté]` au lieu de sauvegarder un fichier vide

---

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
