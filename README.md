# Dictum

Dictée vocale Windows propulsée par Whisper AI local. 100% hors ligne, zéro cloud.

## Fonctionnement

1. Maintenir la touche hotkey (défaut : **F9**)
2. Parler
3. Relâcher
4. Le texte apparaît là où est le curseur

## Fonctionnalités

- Hotkey global configurable (hold-to-record)
- Transcription 100% locale via Whisper (99 langues)
- Détection automatique de langue
- Injection directe dans n'importe quelle app
- Majuscule automatique en début de phrase
- Typographie française (espaces insécables avant `? ! : ;`)
- Substitutions automatiques (abréviations, corrections)
- Historique des 10 dernières transcriptions
- Icône barre des tâches système
- Auto-Enter optionnel
- Liste des microphones disponibles

## Prérequis

### Pour compiler

- [Rust](https://rustup.rs) stable (1.75+)
- CMake 3.20+
- Visual Studio Build Tools (MSVC, composant C++)

```powershell
winget install cmake
winget install Microsoft.VisualStudio.2022.BuildTools
```

### Modèle Whisper

Télécharger un modèle au format ggml depuis Hugging Face :

| Modèle | Taille | Vitesse | Qualité |
|--------|--------|---------|---------|
| `ggml-medium.bin` | 1.5 GB | Rapide | Excellent |
| `ggml-large-v3.bin` | 3 GB | Standard | Maximum |

Lien : `https://huggingface.co/ggerganov/whisper.cpp/tree/main`

Placer le fichier dans :
```
%LOCALAPPDATA%\Dictum\models\ggml-medium.bin
```

## Compilation

```powershell
cd D:\git\Dictum
cargo build --release
```

Le binaire final : `target\release\dictum.exe`

Aucune dépendance runtime (Whisper est compilé statiquement).

## Configuration

Config auto-générée au premier lancement :
```
%LOCALAPPDATA%\Dictum\config.json
```

Exemple :
```json
{
  "model_path": "C:\\Users\\...\\AppData\\Local\\Dictum\\models\\ggml-medium.bin",
  "language": "auto",
  "hotkey": {
    "ctrl": false,
    "alt": false,
    "shift": false,
    "key": "F9"
  },
  "auto_enter": false,
  "french_typography": true,
  "auto_capitalize": true,
  "max_record_secs": 30,
  "microphone": null,
  "substitutions": [
    { "from": "euh", "to": "" },
    { "from": "Thierry point", "to": "Thierry." }
  ]
}
```

Touches supportées : `F1`...`F12`, `Space`, `Tab`

Cliquer **"Paramètres"** dans le menu tray pour ouvrir le fichier dans Notepad.

## Démarrage automatique

Ajouter un raccourci vers `dictum.exe` dans :
```
%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup
```

## GPU (optionnel)

Pour activer CUDA dans `Cargo.toml` :
```toml
whisper-rs = { version = "0.13", features = ["cuda"] }
```

Requiert CUDA Toolkit 11.8+ et une carte NVIDIA.

## Roadmap v2

- [ ] Fenêtre paramètres graphique (egui)
- [ ] Traduction automatique (parle FR, obtient EN)
- [ ] Reformulation IA (oral → professionnel, concis, etc.)
- [ ] Transcription fichiers audio/vidéo (drag & drop)
- [ ] Identification de locuteurs
- [ ] Moteur Parakeet en option (plus rapide, 600 MB)
- [ ] 5 hotkeys configurables avec actions distinctes
- [ ] Mode sélection (reformule le texte sélectionné)

## Architecture

```
src/
  main.rs         orchestration, AppState, canaux inter-threads
  config.rs       Config (serde_json vers %LOCALAPPDATA%\Dictum\)
  audio.rs        capture CPAL 16kHz mono f32 (thread dédié)
  transcribe.rs   inférence Whisper via whisper-rs
  inject.rs       injection texte via enigo + typographie
  hotkey.rs       écoute globale rdev (thread dédié)
  history.rs      10 dernières transcriptions (persistées)
  substitution.rs remplacement abréviations
  tray.rs         icône système + message pump Windows
```
