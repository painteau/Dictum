# TODO — Dictum

## v1 — En cours

### Infra
- [x] Manifest JSON hébergé sur `cdn.breizhzion.com/dictum/manifest.json`
- [x] SHA256 des modèles dans le manifest
- [x] `whisper-cli.exe` + DLLs hébergés sur le CDN
- [x] Build release (`cargo build --release`) — 7.7 MB
- [x] Repo GitHub public `painteau/Dictum`
- [x] Release `v0.1.1` avec `dictum.exe`
- [x] GitHub Actions CI/CD — release auto sur tag `v*`

### À tester
- [ ] Wizard premier lancement (détection GPU, choix modèle, téléchargement)
- [ ] Injection texte dans Notepad, VS Code, navigateur
- [ ] Hotkey F9 hold-to-record
- [ ] Typographie française (espaces insécables)
- [ ] Substitutions automatiques
- [ ] Historique 10 entrées
- [ ] Icône tray rouge pendant enregistrement

### À finir
- [ ] Sauvegarder config depuis le wizard avant fermeture fenêtre
- [ ] Icône `.ico` embarquée dans l'exe (ressource Windows)
- [ ] Démarrage automatique Windows (raccourci Startup)

## v2 — Roadmap

- [ ] Fenêtre paramètres graphique (egui) — remplace l'ouverture Notepad
- [ ] Traduction automatique locale (parle FR, obtient EN)
- [ ] Reformulation IA 7 styles (oral, pro, casual, concis, simplifié, structuré, custom)
- [ ] Transcription fichiers audio/vidéo par drag & drop
- [ ] Identification de locuteurs (diarisation)
- [ ] Moteur Parakeet (600 MB, ultra-rapide)
- [ ] Moteur Qwen3-ASR (4 GB, haute précision)
- [ ] 5 hotkeys configurables avec actions distinctes
- [ ] Mode sélection : sélectionne texte existant → traduit/reformule
- [ ] Pause média automatique pendant enregistrement
- [ ] Support CUDA (GPU NVIDIA) via feature flag
- [ ] Notification sonore debut/fin enregistrement
- [ ] Détection silence automatique (stop sans relâcher la touche)

## v3 — Idées futures

- [ ] Mode live (transcription en temps réel, streaming)
- [ ] Plugin VS Code
- [ ] Accès API locale (HTTP) pour intégrations tierces
