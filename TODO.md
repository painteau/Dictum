# TODO — Dictum

## v1 — En cours

- [ ] Héberger `manifest.json` (Cloudflare R2 ou Workers KV) + mettre la vraie URL dans `downloader.rs`
- [ ] Renseigner SHA256 des modèles dans le manifest
- [ ] Compiler et tester (`cargo build --release`)
- [ ] Tester wizard premier lancement (détection GPU, choix modèle, téléchargement)
- [ ] Tester injection texte dans Notepad, VS Code, navigateur
- [ ] Tester hotkey F9 hold-to-record
- [ ] Vérifier typographie française (espaces insécables)
- [ ] Tester substitutions automatiques
- [ ] Vérifier historique 10 entrées
- [ ] Icône tray : couleur rouge pendant enregistrement
- [ ] Ajouter icône `.ico` embarquée dans l'exe (ressource Windows)
- [ ] Démarrage automatique Windows (raccourci Startup)
- [ ] Sauvegarder config depuis le wizard avant fermeture fenêtre

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
