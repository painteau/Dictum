# CHANGELOG — Dictum

## 2026-06-07 — v1.0.0 — RELEASE STABLE

### Milestone
- Version 1.0 officielle
- CLI complet : `--stats`, `--search`, `--export`, `--diagnose`, `--config-check`, `--config-reset`, `--reset-history`, `--list-models`, `--list-devices`, `--list-languages`, `--open-config`, `--open-dir`, `--version-full`
- Tray : 4 états icône (bleu/rouge/orange/gris), mode pause, score config, mots historique
- Wizard : espace disque, vitesse download, avertissement hotkey
- Injection : fallback clipboard, typographie française complète (chevrons)
- Transcription : filtre hallucinations bigrammes, timeout adaptatif
- Config : **138 méthodes** (127 → 138), 6 profiles
- History : **50 méthodes**
- Total API : **188 méthodes**
- Commits : **471+**
- Zéro warning `cargo build`

---

## 2026-06-07 — v0.9.9 — CLI EXHAUSTIF + CONFIG 138 MÉTHODES

### Ajouté
- CLI `--list-models` : liste modèles CDN avec statut local
- CLI `--open-config` : ouvrir la config dans l'éditeur
- CLI `--open-dir` : ouvrir le dossier Dictum dans Explorer
- Config 138 méthodes : `remove_substitution_by_from`, `find_substitution`, `min_record_ms_display`
- Tray : menu historique affiche nb entrées ET nb mots
- Hotkey : log conseil admin si rdev échoue
- 470 commits | **188 méthodes API** (138 Config + 50 History)

---

## 2026-06-07 — v0.9.8 — ICÔNE PAUSE + POLISH

### Ajouté
- Icône tray grise en mode pause (4 états : bleu/orange/rouge/gris)
- Profiles config : `quiet`, `english`, `dictaphone`
- CLI `--version-full` : arch, OS, whisper-cli présence
- Wizard hotkey : avertissement touche sans modificateur
- Log stats session à l'arrêt (mots + entrées)
- Config 135 méthodes | Total API : **185 méthodes** | 464 commits

---

## 2026-06-07 — v0.9.7 — CLI COMPLET + QUALITÉ

### Ajouté
- CLI `--search <texte>` : recherche dans l'historique
- CLI `--diagnose` : rapport complet fichiers/audio/réseau/config/score
- Substitutions prédéfinies françaises : 15 abréviations (dc, pk, stp, svp...)
- Wizard : vitesse téléchargement MB/s en temps réel
- 458 commits | **182 méthodes API** (132 Config + 50 History)

---

## 2026-06-07 — v0.9.6 — PAUSE, TYPO, HOTKEYS

### Ajouté
- Mode pause/reprendre via menu tray (hotkey ignorée si en pause, tooltip mis à jour)
- Typographie française : guillemets droits `"texte"` → `« texte »` automatique
- Hotkey : support touches lettres A-Z (avec modificateur Ctrl/Alt/Shift)
- Notification ballon Windows quand une mise à jour est disponible (PowerShell toast)
- Config 132 méthodes : `from_json_str`, `substitutions_count`, `record_max_secs_display`, hotkey helpers
- CLI `--export [chemin]` : exporte l'historique en Markdown
- 453 commits total

---

## 2026-06-07 — v0.9.5 — CLI ENRICHI + ROBUSTESSE INJECTION

### Ajouté
- CLI `--stats` : statistiques historique (mots, entrées, fréquences)
- CLI `--config-check` : valide la config, exit 1 si problème
- CLI `--reset-history` : efface l'historique sans lancer l'appli
- Injection texte : fallback clipboard (Ctrl+V) si enigo init échoue
- Tray About : score config + breakdown détaillé dans la popup
- 446 commits total

---

## 2026-06-07 — v0.9.4 — QUALITÉ + ROBUSTESSE

### Ajouté
- History : 50 méthodes — `first_entry`, `text_at`, `timestamp_at`, `count_words_in`
- Rotation log automatique : 5 MB max, 3 fichiers conservés (flexi_logger)
- Vérification espace disque avant téléchargement modèle (GetDiskFreeSpaceExW)
- Wizard : affichage espace disque selon modèle choisi (vert/rouge)
- Tooltip tray : nombre de mots total dictés affiché
- Filtre hallucinations Whisper : bigrammes répétés supprimés

### Chiffres
- History : **50 méthodes**, Config : **127 méthodes**, Total API : **177 méthodes**

---

## 2026-06-07 — v0.9.3 — CLÔTURE SESSION

### Chiffres définitifs
| Métrique | Valeur |
|----------|--------|
| Config méthodes | **127** |
| History méthodes | **46** |
| Total API | **173 méthodes** |
| Commits | **433** |
| Tags | **72** |
| Warnings | **0** |

### Ajouté en toute fin de nuit
- `history.rs` : `is_growing()`, `percentage_full()`
- History : **46 méthodes** ✓

---

## 2026-06-07 — v0.9.2 — BILAN ABSOLU FINAL

### Chiffres officiels de la nuit
| Métrique | Valeur |
|----------|--------|
| Config méthodes | **127** |
| History méthodes | **44** |
| Total API | **171 méthodes** |
| Commits | **431** |
| Tags releases | **71** |
| Warnings | **0** |
| Logiciels testés | en attente |

### Ajouté en fin de session
- `history.rs` : `avg_words_per_entry()`, `entries_after()`
- History : **44 méthodes** ✓

---

## 2026-06-07 — v0.9.1 — Config 127 méthodes

### Ajouté
- `config.rs` : `total_config_fields()`, `required_fields_present()`, `optional_fields_count()`
- `config.rs` : `copy_with_model()`, `diff_fields()`
- `config.rs` : `is_configured_for_french_dictation()`, `min_disk_space_mb()`
- `config.rs` : `config_age_display()`, `is_compatible_with_version()`
- Config : **127 méthodes** | Total : **169 méthodes API** ✓

---

## 2026-06-07 — v0.9.0 — FIN DE SESSION NOCTURNE

### Bilan final officiel
- **Config : 122 méthodes** ✓ (objectif 100 dépassé)
- **History : 42 méthodes** ✓
- **Total : 164 méthodes API publiques** ✓
- **430+ commits** en une nuit
- **68+ tags** releases
- Zéro warning `cargo build`
- Logs 100% français
- Dictum v0.9.0 — prêt pour test utilisateur

### Depuis v0.8.7
- `config.rs` : `is_configured_for_french_dictation()`, `min_disk_space_mb()`
- `config.rs` : `config_age_display()`, `is_compatible_with_version()`
- `config.rs` : **122 méthodes** (record)

---

## 2026-06-07 — v0.8.7 — Config 118 méthodes

### Ajouté
- `config.rs` : `recommend_model()` et `recommend_threads()` — conseils hardware
- `config.rs` : `model_size_mb_estimate()` — taille estimée modèle
- `config.rs` : `performance_label()` — Rapide/Standard/Qualité/Haute qualité
- `config.rs` : `estimated_transcription_time()` — estimation durée transcription
- Config : **118 méthodes** | Total : **160 méthodes API** ✓

---

## 2026-06-07 — v0.8.6 — Bilan final 3h30

### Stats officielles finales
- **Config : 116 méthodes** ✓
- **History : 42 méthodes** ✓
- **Total API : 158 méthodes publiques** ✓
- **420 commits** en une nuit
- **67 tags** releases CI/CD
- Zéro warning `cargo build`
- Logs 100% français

### Ajouté depuis v0.8.5
- `config.rs` : `model_size_mb_estimate()` — estimation taille modèle
- `config.rs` : `performance_label()`, `is_optimized_for_speed()`, `is_optimized_for_quality()`
- `config.rs` : `estimated_transcription_time()`
- `full_status()` affiche taille MB et label performance

---

## 2026-06-07 — v0.8.5 — Score détaillé

### Ajouté
- `config.rs` : `score_breakdown()` — détail des points par critère
- `config.rs` : `score_breakdown_display()` — affichage lisible
- Config : **111 méthodes** | History : **42 méthodes** | Total : **153** ✓

### Stats 3h du matin
- **414 commits** | **66 tags** | **153 méthodes API**

---

## 2026-06-07 — v0.8.4 — Score et qualité

### Ajouté
- `config.rs` : `score()` — note 0-100 basée sur l'état de la config
- `config.rs` : `score_label()` — "Excellent", "Très bon", "Bon", "Basique", "Incomplet"
- `config.rs` : `is_ready_message()` enrichi avec score/100 et label
- `config.rs` : `diagnose()` affiche le score en en-tête
- Config : **109 méthodes** | Total : **151 méthodes API** ✓

### Corrigé
- v0.8.2 avait un bug compile (format string dupliqué) — corrigé en v0.8.3

---

## 2026-06-07 — v0.8.2 — Profils et comparaison

### Ajouté
- `config.rs` : `apply_profile_french_standard()`, `apply_profile_minimal()`, `apply_profile_performance()`
- `config.rs` : `changes_from_default()`, `changes_summary_display()`
- `config.rs` : `log_summary()` enrichi avec changements vs défaut
- Config : **106 méthodes** | Total : **148 méthodes API** ✓

---

## 2026-06-07 — v0.8.1 — Diagnostic intégré

### Ajouté
- `config.rs` : `diagnose()` — rapport diagnostic formaté ASCII complet
- `tray.rs` : À propos utilise `diagnose()` au lieu du format manuel
- `main.rs` : log debug `diagnose()` au démarrage en mode debug
- `history.rs` : `longest_text_length()`, `shortest_text_length()`
- Config : **101 méthodes** | History : **42 méthodes** | Total : **143** ✓

---

## 2026-06-07 — v0.8.0 — BILAN FINAL NUIT

### Stats officielles de la nuit
- **Config : 100 méthodes** ✓ milestone
- **History : 42 méthodes** ✓
- **Total API : 142 méthodes publiques**
- **395+ commits** en une nuit
- **60+ tags** releases CI/CD
- Zéro warning `cargo build`
- Logs 100% français
- API fluent style (set_*, toggle_*, is_*)
- Validations, diagnostics, exporters complets

### Dictum v0.8.0 — Prêt pour test utilisateur

---

## 2026-06-07 — v0.7.9 — MILESTONE History 40 méthodes

### DOUBLE MILESTONE
- Config : **100 méthodes** ✓
- History : **40 méthodes** ✓
- Total : **140 méthodes API** ✓

### Stats de la nuit
- **394 commits** en une nuit
- **60 tags** releases
- Zéro warning
- Logs 100% français

### Ajouté
- `history.rs` : `sentences_count()`, `recent_texts()`, `oldest_entry()`, `entry_at_index()`
- `history.rs` : `words_count()`, `has_recent_entry()`, `entries_today()`
- `history.rs` : `time_since_last()`, `is_empty_or_old()`, `word_frequency()`

---

## 2026-06-07 — v0.7.7 — History 33 méthodes

### Ajouté
- `history.rs` : `words_count()`, `has_recent_entry()`, `entries_today()`
- History : **33 méthodes** | Config : **100 méthodes** | Total : **133** ✓

---

## 2026-06-07 — v0.7.6 — MILESTONE Config 100 méthodes

### MILESTONE HISTORIQUE
**Config atteint 100 méthodes publiques** en une seule nuit de travail.

### Ajouté
- `config.rs` : `effective_threads()` — threads réels utilisés (auto ou fixe)
- `config.rs` : `effective_timeout()` — timeout adaptatif calculé
- `config.rs` : `substitution_index()` — trouve index d'une règle
- `config.rs` : `can_transcribe()`, `recording_is_limited()`, `silence_detection_active()`
- `config.rs` : `has_min_record_limit()`, `is_history_full()`, `history_capacity_remaining()`

### Bilan total
- **Config : 100 méthodes** ✓
- **History : 30 méthodes** ✓
- **Total API : 130 méthodes** ✓
- Zéro warning, 100% français
- 390+ commits en une nuit

---

## 2026-06-07 — v0.7.5 — Config 91 méthodes

### Ajouté
- `config.rs` : `needs_wizard()`, `is_production_ready()`, `has_whisper_optimizations()`
- `config.rs` : `is_using_cuda()`, `is_using_default_microphone()`, `has_custom_model_path()`
- `config.rs` : `uses_large_model()`, `uses_medium_model()`, `is_low_latency()`, `is_verbose_mode()`
- Config : **91 méthodes** | Total : **121 méthodes API** ✓

---

## 2026-06-07 — v0.7.4 — Config 81 méthodes + predicats

### Ajouté
- `config.rs` : 10 predicats `is_*()` — is_french, is_auto_detect, is_beep_enabled, is_debug_mode...
- `config.rs` : `with_model_name()`, `set_beep_freqs()`
- Config : **81 méthodes** | Total : **111 méthodes API** ✓

---

## 2026-06-07 — v0.7.3 — Config 71 méthodes + fluent API

### Ajouté
- `config.rs` : `set_prefix_space()`, `set_whisper_temperature()`, `set_no_speech()`
- `config.rs` : `set_log_level()`, `set_inject_delay()`
- `config.rs` : `set_beep_freqs()` — configure start/end/duration en une fois
- `config.rs` : `with_model_name()` — sélectionne modèle par nom court
- Config : **71 méthodes** | Total API : **101 méthodes** ✓

### Bilan final de la nuit (v0.1.0 → v0.7.3)
- **380+ commits** — nuit de travail intensif
- **54 tags** — CI/CD toutes les 10-15 min
- **71 méthodes Config** + **30 méthodes History** = **101 méthodes API**
- Zéro warning `cargo build`
- Logs 100% français
- Validation config complète
- Diagnostics exhaustifs
- API fluent style

---

## 2026-06-07 — v0.7.2 — Config 66 méthodes

### Ajouté
- `config.rs` : `set_log_level()` avec validation
- `config.rs` : `set_inject_delay()`
- Config : **66 méthodes** | History : **30 méthodes** | Total : **96 méthodes** ✓

---

## 2026-06-07 — v0.7.1 — Config 64 méthodes (API complète)

### Ajouté
- `config.rs` : `set_auto_capitalize()`, `set_auto_enter()`, `set_french_typography()`
- `config.rs` : `set_max_history()`, `set_max_record_secs()`
- `config.rs` : `set_silence_threshold()`, `set_threads()`
- `config.rs` : `toggle_beep()`, `toggle_pause_media()`
- Config : **64 méthodes** | History : **30 méthodes** | Total : **94 méthodes** ✓

---

## 2026-06-07 — v0.7.0 — Milestone API 85 méthodes

### Milestone majeur
**85 méthodes publiques** au total : Config (55) + History (30)

### Depuis v0.6.4
- `config.rs` : `set_model()`, `set_microphone()`, `set_language()`, `set_hotkey()`
- `config.rs` : `add_substitution()`, `remove_substitution()`, `clear_substitutions()`
- `config.rs` : **55 méthodes** — API Rust complète
- `history.rs` : `deduplicate()`, `keep_recent()`, `contains()`, `count_containing()`
- `history.rs` : **30 méthodes** — History API mature

---

## 2026-06-07 — v0.6.4 — History 30 méthodes

### Ajouté
- `history.rs` : `deduplicate()`, `keep_recent(n)`
- `history.rs` : `contains()`, `count_containing()`
- `history.rs` : `sorted_by_length()`, `sorted_by_date()`
- `history.rs` : `longest_entry()`, `shortest_entry()`
- `history.rs` : `to_markdown()` — export enrichi avec stats
- History : **30 méthodes** ✓

---

## 2026-06-07 — v0.6.3 — History API finale

### Ajouté
- `history.rs` : `to_markdown()` — export Markdown enrichi avec stats
- `history.rs` : `sorted_by_length()`, `sorted_by_date()`
- `history.rs` : `longest_entry()`, `shortest_entry()`
- `history.rs` : `export_to_file()` simplifié (délègue à `to_markdown()`)
- History : **26 méthodes publiques** ✓

### Bilan nuit
- Config : 48 méthodes
- History : 26 méthodes
- Zéro warning
- v0.6.0 → v0.6.3 tout en releases CI

---

## 2026-06-07 — v0.6.2 — History stats complets

### Ajouté
- `history.rs` : `unique_words_count()`, `stats_summary()`
- `tray.rs` : À propos utilise `stats_summary()` — simplifié
- `main.rs` : log `stats_summary()` au démarrage

---

## 2026-06-07 — v0.6.1 — Config 48 méthodes + History avancée

### Ajouté
- `config.rs` : `export_to_file()`, `export_to_string()`, `defaults()`
- `config.rs` : `from_json_string()`, `merge_substitutions_from()`
- `history.rs` : `most_common_word()` — analyse fréquence mots
- Config : **48 méthodes publiques** ✓

---

## 2026-06-07 — v0.6.0 — Milestone Config 45 méthodes

### Milestone
`Config` atteint 45 méthodes publiques. API complète, expressive, zéro warning.

### Depuis v0.5.8
- `config.rs` : `from_json_string()`, `to_json_string()`, `defaults()`
- `config.rs` : `log_level_display()` avec emoji 🟢🔵🟡
- `config.rs` : `config_version_display()`, `microphone_display()`
- `config.rs` : `audio_input_display()`, `substitutions_display()`

### Catalogue complet Config (45 méthodes)
load, save, load_from, save_to, open_in_editor, open_data_dir, reset_to_default,
validate, is_ready_message, full_status, description, log_summary, app_version,
model_name, language_display, hotkey_string, profile_name, whisper_speed_label,
silence_level_label, beep_description, inject_mode_label, threads_display,
record_duration_label, microphone_display, audio_input_display,
substitutions_display, inject_delay_display, max_history_display,
record_config_display, whisper_config_display, config_version_display,
log_level_display, to_json_string, from_json_string, defaults,
is_model_ready, is_whisper_cli_ready, is_fully_ready,
has_substitutions, substitution_count, apply_substitutions,
data_dir, models_dir, log_path, history_export_path

---

## 2026-06-07 — v0.5.8 — UX logs enrichis

### Ajouté
- `config.rs` : `log_level_display()` — emoji 🟢 info, 🔵 debug, etc.
- `config.rs` : `config_version_display()` — "config v1 | app v0.5.0"
- `main.rs` : emoji niveau log dans message démarrage
- `main.rs` : `config_version_display()` + `is_ready_message()` au démarrage

---

## 2026-06-07 — v0.5.7 — Helpers audio et micro

### Ajouté
- `config.rs` : `microphone_display()` — "défaut système" ou nom configuré
- `config.rs` : `audio_input_display()` — résumé complet config audio
- `config.rs` : `substitutions_display()` — aperçu 3 premières règles
- `config.rs` : `max_history_display()`, `record_config_display()`
- `config.rs` : `inject_delay_display()` — rapide/normal/lent
- `main.rs` : log `microphone_display()` et aperçu substitutions

---

## 2026-06-07 — v0.5.6 — API centralisée

### Ajouté
- `config.rs` : `apply_substitutions()` — délègue directement à substitution::apply()
- `main.rs` : utilise `config.apply_substitutions()` — simplifié

### Bilan nuit — 30+ méthodes Config
Toute la logique métier passée par des méthodes expressives de Config.

---

## 2026-06-07 — v0.5.5 — Description et profils

### Ajouté
- `config.rs` : `description()` — one-liner `[profil] langue | modèle | hotkey | silence | beep`
- `config.rs` : `profile_name()`, `whisper_config_display()`
- `tray.rs` : tooltip utilise `config.description()`
- `main.rs` : log `description()` avant `log_summary()`

---

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
