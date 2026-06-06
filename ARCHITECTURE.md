# Architecture — Dictum

## Vue d'ensemble

Dictum est une app de dictée vocale Windows 100% locale. Elle capture le micro, transcrit via Whisper, et injecte le texte là où est le curseur.

```
┌─────────────────────────────────────────────────────────────────┐
│                        dictum.exe                               │
│                                                                 │
│  Thread principal     Thread hotkey      Thread pipeline        │
│  ┌────────────┐       ┌────────────┐    ┌──────────────────┐   │
│  │ Tray icon  │       │   rdev     │    │ RecordHandle     │   │
│  │ Msg pump   │──────▶│  listener  │───▶│ (thread propre)  │   │
│  │ Menu events│       │ (bloquant) │    │ cpal 16kHz mono  │   │
│  └────────────┘       └────────────┘    └──────┬───────────┘   │
│                                                │               │
│                                         samples Vec<f32>       │
│                                                │               │
│                                         ┌──────▼───────────┐   │
│                                         │  transcribe.rs   │   │
│                                         │  whisper-cli.exe │   │
│                                         │  (subprocess)    │   │
│                                         └──────┬───────────┘   │
│                                                │               │
│                                         ┌──────▼───────────┐   │
│                                         │   inject.rs      │   │
│                                         │   enigo          │   │
│                                         │   (SendInput)    │   │
│                                         └──────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## Fichiers sources

| Fichier | Rôle |
|---------|------|
| `main.rs` | Point d'entrée, `AppState` partagé, spawn des threads, logger fichier |
| `config.rs` | `Config` sérialisée JSON dans `%LOCALAPPDATA%\Dictum\config.json` |
| `audio.rs` | Capture micro CPAL 16 kHz mono f32, stream isolé, beep Beep API, timeout |
| `transcribe.rs` | Détection silence RMS, écrit WAV temp, appelle `whisper-cli.exe` subprocess |
| `inject.rs` | Injection texte via `enigo` (SendInput Win32), typographie française |
| `hotkey.rs` | Écoute globale clavier via `rdev` (thread bloquant), hold-to-record |
| `history.rs` | 10 dernières transcriptions avec horodatage, persistées JSON |
| `substitution.rs` | Remplacement abréviations/corrections après transcription |
| `tray.rs` | Icône dynamique (bleu/rouge), menu : historique, clipboard, config, log |
| `media.rs` | Toggle VK_MEDIA_PLAY_PAUSE (SendInput Win32) pour pause/reprise médias |
| `setup.rs` | Wizard egui 5 étapes : GPU auto-détect, modèle, langue, hotkey, download |
| `downloader.rs` | Fetch manifest JSON CDN, téléchargement binaires+modèles, SHA256 |
| `updater.rs` | Check GitHub releases API, télécharge Setup.exe, lance `/SILENT` |
| `build.rs` | Génère .ico 32x32, embed version info Windows via `winresource` |

## Modèle de threads

```
main()
├── Thread hotkey     → rdev::listen() [bloquant, envoie AppEvent via channel]
├── Thread pipeline   → reçoit AppEvent, pilote record/transcribe/inject
│   └── Thread transcrip. → whisper-cli subprocess + injection (spawné par pipeline)
│       └── Thread audio  → cpal Stream [propre thread, non-Send]
└── Thread principal  → tray + message pump Windows [boucle infinie]
```

### Canaux de communication

```
hotkey thread ──[AppEvent]──▶ pipeline thread
pipeline thread ──[stop signal]──▶ audio thread
audio thread ──[Vec<f32> samples]──▶ pipeline thread
```

## Flux de données

```
Micro (WASAPI)
    │ f32 samples @ 16kHz mono
    ▼
audio::RecordHandle
    │ Vec<f32> (à la release du hotkey)
    ▼
transcribe::transcribe()
    │ écrit dictum_record.wav dans %TEMP%
    │ appelle whisper-cli.exe --model ... --file ...
    │ parse stdout (filtre timestamps)
    ▼
substitution::apply()
    │ remplace abréviations
    ▼
inject::inject_text()
    │ capitalisation, typographie FR
    │ enigo::text() → SendInput Win32
    ▼
Curseur actif (n'importe quelle app)
```

## Infrastructure CDN

```
cdn.breizhzion.com/dictum/
├── manifest.json          ← URL modifiable sans recompiler
├── whisper-cli.exe        ← whisper.cpp v1.8.6 Win32
├── ggml.dll
├── ggml-base.dll
├── ggml-cpu.dll
└── whisper.dll

Modèles (HuggingFace) :
├── ggml-medium.bin        ← 1.5 GB
└── ggml-large-v3.bin      ← 3 GB
```

Le manifest pointe vers HuggingFace pour les modèles (fichiers trop lourds pour le CDN). Les binaires whisper-cli sont hébergés directement sur le CDN Breizhzion pour garantir la version et la disponibilité.

## Données utilisateur

```
%LOCALAPPDATA%\Dictum\
├── config.json            ← paramètres (hotkey, langue, substitutions...)
├── history.json           ← 10 dernières transcriptions
└── models\
    └── ggml-medium.bin    ← (ou large-v3)

%LOCALAPPDATA%\Dictum\
├── whisper-cli.exe        ← téléchargé au setup
├── ggml.dll
├── ggml-base.dll
├── ggml-cpu.dll
└── whisper.dll
```

## Dépendances Rust

| Crate | Rôle |
|-------|------|
| `cpal` | Capture audio multi-plateforme (WASAPI sur Windows) |
| `hound` | Écriture fichier WAV pour whisper-cli |
| `rdev` | Écoute globale clavier (hotkeys système) |
| `enigo` | Injection texte (SendInput Win32) |
| `tray-icon` | Icône barre système Windows |
| `eframe` + `egui` | Wizard premier lancement |
| `reqwest` | Téléchargement manifest + binaires (blocking) |
| `sha2` + `hex` | Vérification intégrité SHA256 |
| `serde` + `serde_json` | Config/historique JSON |
| `crossbeam-channel` | Communication inter-threads |
| `winapi` | Message pump Windows, MessageBox natif |
| `dirs` | Chemin `%LOCALAPPDATA%` portable |
| `log` + `env_logger` | Logging (console en debug, silencieux en release) |

## Décisions d'architecture

### subprocess whisper-cli vs binding Rust
`whisper-rs` (binding C++) ne compile pas sur Windows avec MSVC récent (bug struct size dans bindgen). On appelle `whisper-cli.exe` en subprocess : même performance, zéro dépendance native dans notre binaire.

### Stream audio dans son propre thread
`cpal::Stream` n'implémente pas `Send` sur Windows (WASAPI). Le stream vit dans le thread qui l'a créé, et on communique via channels crossbeam.

### Message pump natif vs winit
Le tray utilise une boucle `PeekMessage/DispatchMessage` Win32 native plutôt que winit, pour éviter une dépendance lourde sur la fenêtre principale (l'app n'a pas de fenêtre visible).

### Manifest JSON distant
L'URL des modèles et binaires est centralisée dans un JSON hébergé. Permet de changer les URLs (migration CDN, nouvelle version whisper) sans recompiler ni redistribuer le logiciel.
