# Contribuer à Dictum

## Prérequis

```powershell
winget install Rustup.Rustup
winget install LLVM.LLVM
winget install Kitware.CMake
winget install Microsoft.VisualStudio.2022.BuildTools
```

## Compilation locale

```powershell
git clone https://github.com/painteau/Dictum
cd Dictum
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
cargo build
```

## Structure du projet

```
src/
  main.rs         — orchestration, threads, CLI
  config.rs       — Config JSON
  audio.rs        — capture CPAL, beep
  transcribe.rs   — whisper-cli subprocess, détection silence
  inject.rs       — injection texte, typographie FR
  hotkey.rs       — hotkeys globaux (rdev)
  history.rs      — historique 10 entrées
  substitution.rs — remplacement abréviations
  tray.rs         — icône système, menus
  media.rs        — pause/reprise médias Win32
  setup.rs        — wizard egui premier lancement
  downloader.rs   — manifest CDN, téléchargement SHA256
  updater.rs      — auto-update GitHub releases
build.rs          — icône .ico, version info Windows
```

## Créer une release

```powershell
git tag v0.x.x
git push origin v0.x.x
# GitHub Actions compile et publie Dictum-Setup-x.x.x-x64.exe
```

## Modifier le manifest des modèles

Le fichier `D:/git/cdn/dictum/manifest.json` sur le repo CDN contrôle les URLs des modèles et binaires. Modifier et pusher pour que tous les clients reçoivent les nouvelles URLs sans recompiler Dictum.

## Conventions

- Un commit par feature
- Messages de commit en français ou anglais, préfixés (`feat:`, `fix:`, `docs:`, `ci:`)
- Zéro warning Rust (`cargo build` doit être propre)
- Mettre à jour `CHANGELOG.md` et `TODO.md` à chaque PR
