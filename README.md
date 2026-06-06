# Dictum

Dictée vocale Windows 100% locale, propulsée par Whisper AI. Zéro cloud, zéro abonnement.

## Installation

Télécharger **`Dictum-Setup-x.x.x-x64.exe`** depuis la page [Releases](https://github.com/painteau/Dictum/releases) et l'exécuter.

L'installateur :
- Copie `dictum.exe` dans `Program Files\Dictum`
- Propose d'ajouter Dictum au démarrage Windows
- Installe un désinstallateur propre

Au premier lancement, un **wizard** guide la configuration : choix du modèle, langue, hotkey — puis télécharge automatiquement Whisper.

## Fonctionnement

1. Maintenir **F9** (configurable)
2. Parler
3. Relâcher
4. Le texte apparaît là où est le curseur

Compatible avec n'importe quelle application : navigateur, éditeur de code, messagerie, terminal...

## Fonctionnalités

- Hotkey global configurable (hold-to-record), modificateurs Ctrl/Alt/Shift
- Transcription 100% locale via [Whisper](https://github.com/ggerganov/whisper.cpp) (99 langues)
- Détection automatique de langue
- Injection directe dans n'importe quelle app Windows
- Majuscule automatique en début de phrase
- Typographie française (espaces insécables avant `? ! : ;`)
- Substitutions automatiques (abréviations, corrections personnalisées)
- Historique des 10 dernières transcriptions
- Icône barre des tâches système (bleue au repos, grise en transcription)
- Auto-Enter optionnel
- Mise à jour automatique (vérification silencieuse au démarrage)

## Configuration

Le fichier de config est généré automatiquement au premier lancement :
```
%LOCALAPPDATA%\Dictum\config.json
```

Cliquer **"Paramètres"** dans le menu tray pour l'ouvrir dans Notepad.

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
    { "from": "virgule", "to": "," }
  ]
}
```

Touches supportées : `F1`...`F12`, `Space`, `Tab`

## Modèles disponibles

| Modèle | Taille | Vitesse | Qualité |
|--------|--------|---------|---------|
| `medium` | 1.5 GB | Rapide | Excellent |
| `large-v3` | 3 GB | Standard | Maximum |

Les modèles sont téléchargés automatiquement par le wizard depuis [HuggingFace](https://huggingface.co/ggerganov/whisper.cpp).

## Prérequis système

- Windows 10/11 x64
- Connexion internet uniquement pour le téléchargement initial (~1.5 GB pour medium)
- Microphone fonctionnel

## Compilation depuis les sources

```powershell
# Prérequis
winget install Rustup.Rustup
winget install LLVM.LLVM
winget install Kitware.CMake
winget install Microsoft.VisualStudio.2022.BuildTools

# Build
cd D:\git\Dictum
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
cargo build --release
# Binaire : target\release\dictum.exe
```

## Créer une release

```powershell
git tag v0.x.x
git push origin v0.x.x
# GitHub Actions compile et publie Dictum-Setup-x.x.x-x64.exe automatiquement
```

## Architecture

```
src/
  main.rs         orchestration, AppState, canaux inter-threads
  config.rs       Config JSON (%LOCALAPPDATA%\Dictum\config.json)
  audio.rs        capture CPAL 16kHz mono f32 (thread dédié)
  transcribe.rs   écrit WAV temp, appelle whisper-cli.exe en subprocess
  inject.rs       injection texte via enigo (SendInput Win32)
  hotkey.rs       écoute globale rdev (thread dédié, bloquant)
  history.rs      10 dernières transcriptions (persistées JSON)
  substitution.rs remplacement abréviations
  tray.rs         icône système + message pump Windows natif
  setup.rs        wizard egui premier lancement
  downloader.rs   manifest JSON CDN, téléchargement SHA256
  updater.rs      check GitHub releases, auto-update silencieux
build.rs          génère icône .ico + embed version info Windows
```

**Voir [ARCHITECTURE.md](ARCHITECTURE.md) pour le détail complet.**

## Roadmap v2

- Fenêtre paramètres graphique (egui)
- Traduction automatique locale (parle FR, obtient EN)
- Reformulation IA 7 styles
- Transcription fichiers audio/vidéo par drag & drop
- Identification de locuteurs
- Mode sélection (reformule le texte sélectionné)
- Notification sonore début/fin enregistrement
- Détection silence automatique
