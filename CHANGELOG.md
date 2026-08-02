# Journal des modifications

Le format suit [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/) et le
projet applique le [versionnage sémantique](https://semver.org/lang/fr/).

## [Non publié]

## [0.1.0] — 2026-08-02

Première version publique.

### Ajouté

- **Serveurs** : création, édition, suppression, groupes, tags, couleurs,
  favoris, recherche instantanée, tri, vues grille et liste.
- **~/.ssh/config** : détection des hôtes via `ssh -G`, édition et suppression
  des entrées avec sauvegarde du fichier.
- **Terminal** : xterm.js sur PTY, onglets, splits horizontaux et verticaux,
  sessions préservées pendant la navigation, raccourcis clavier.
- **Clés SSH** : scan de `~/.ssh`, génération (ED25519, RSA 4096, ECDSA), import,
  édition du contenu privé et public, gestion de la passphrase, empreintes,
  détection et correction des permissions trop ouvertes.
- **Agent SSH** : mémorisation de la passphrase dans le coffre natif du système.
- **Tunnels** : redirections locale, distante et dynamique (SOCKS), avec suivi
  d'état et détection des échecs.
- **Base locale** chiffrée avec SQLCipher, clé conservée dans le coffre natif.
- **Apparence** : thèmes clair et sombre, deux accents, réglages du terminal.
- **Journalisation** avec rotation des fichiers.
- Interfaces de synchronisation (phase 2), sans implémentation réseau.

[Non publié]: https://github.com/meissaniang/galvus/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/meissaniang/galvus/releases/tag/v0.1.0
