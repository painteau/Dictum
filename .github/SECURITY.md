# Politique de sécurité

## Versions supportées

| Version | Support |
|---------|---------|
| 0.2.x   | ✅ Oui  |
| < 0.2.0 | ❌ Non  |

## Signaler une vulnérabilité

Créer une issue privée via **GitHub Security Advisories** (onglet Security du repo).

Ne pas ouvrir d'issue publique pour les vulnérabilités.

## Périmètre

Dictum traite du texte audio localement. Pas de serveur, pas de cloud.
Les seuls vecteurs externes sont :
- Le manifest JSON téléchargé depuis `cdn.breizhzion.com`
- Les binaires whisper-cli téléchargés au setup
- Les releases GitHub vérifiées via API publique
