<div align="center">

<img src="design/galvus-icon.png" width="120" alt="Galvus" />

# Galvus

**Client SSH desktop moderne, open source et 100 % local.**

[![CI](https://github.com/meissaniang/Galvus/actions/workflows/ci.yml/badge.svg)](https://github.com/meissaniang/Galvus/actions/workflows/ci.yml)
[![Release](https://github.com/meissaniang/Galvus/actions/workflows/release.yml/badge.svg)](https://github.com/meissaniang/Galvus/actions/workflows/release.yml)
[![Licence MIT](https://img.shields.io/badge/licence-MIT-23C48A)](LICENSE)
[![Tauri v2](https://img.shields.io/badge/Tauri-v2-24C8DB)](https://tauri.app)

</div>

---

Galvus est une alternative libre à Termius. Vos serveurs, vos clés et vos tunnels
restent sur votre machine : **aucun compte, aucun cloud, aucune télémétrie**.

Les connexions s'appuient sur l'**OpenSSH de votre système** — Galvus ne
réimplémente pas le protocole SSH. Les métadonnées sont stockées dans une base
**SQLite chiffrée** (SQLCipher) dont la clé vit dans le **coffre natif** de l'OS
(Trousseau macOS, Credential Manager, Secret Service).

## Fonctionnalités

- **Serveurs** — création, groupes, tags, couleurs, favoris, recherche instantanée,
  vues grille et liste. Les hôtes de `~/.ssh/config` sont détectés et éditables.
- **Terminal** — xterm.js, onglets multiples, splits horizontaux et verticaux,
  sessions préservées pendant la navigation.
- **Clés SSH** — génération (ED25519, RSA 4096, ECDSA), import, édition du contenu,
  gestion de la passphrase, empreintes, contrôle des permissions.
- **Agent SSH** — la passphrase n'est saisie qu'une fois puis mémorisée dans le
  coffre natif.
- **Tunnels** — redirections locale, distante et dynamique (SOCKS), avec état en
  temps réel.
- **Apparence** — thèmes clair et sombre, deux accents, police et taille du
  terminal réglables.

## Installation

Téléchargez la dernière version depuis la page
[Releases](https://github.com/meissaniang/Galvus/releases) :

| Plateforme | Fichier                       |
| ---------- | ----------------------------- |
| macOS      | `.dmg`                        |
| Windows    | `.msi` ou l'installeur `.exe` |
| Linux      | `.AppImage`, `.deb` ou `.rpm` |

> Les binaires ne sont pas encore signés. macOS demande un clic droit → **Ouvrir**
> au premier lancement, et Windows affiche un avertissement SmartScreen.

**Prérequis** : un client OpenSSH sur la machine — présent par défaut sur macOS et
Linux, ainsi que sur Windows 10 (1809+) et Windows 11.

## Développement

Il vous faut [Node.js](https://nodejs.org) 20+, [pnpm](https://pnpm.io) et
[Rust](https://rustup.rs), plus les
[dépendances système de Tauri](https://tauri.app/start/prerequisites/).

```bash
git clone https://github.com/meissaniang/Galvus.git
cd galvus
pnpm install
pnpm tauri dev
```

Commandes utiles :

```bash
pnpm tauri build      # installeurs dans src-tauri/target/release/bundle/
pnpm test             # tests unitaires du frontend (Vitest)
pnpm lint             # ESLint
pnpm format           # Prettier
cd src-tauri && cargo test && cargo clippy --all-targets
```

## Architecture

```
src/                      Frontend Vue 3 + TypeScript
  components/  pages/  layouts/      Interface
  stores/                            État Pinia
  services/                          Logique métier
  repositories/                      Seul point d'appel des commandes Tauri
  types/  utils/  router/

src-tauri/src/            Backend Rust
  commands/                          Commandes exposées au frontend
  services/                          Terminal (PTY), tunnels, sync (phase 2)
  ssh/                               Config, clés, agent
  database/                          SQLite chiffrée + repositories
  security/                          Coffre natif, validation des chemins
  models/  errors/  config/
```

Règles structurantes : aucun SQL dans les composants, aucun appel Tauri hors des
`repositories`, et toute opération sensible côté Rust.

## Sécurité

Les clés privées ne quittent jamais `~/.ssh` et leurs permissions sont vérifiées.
Aucune passphrase n'est écrite en base : elles vivent dans le coffre natif via
l'agent SSH. Pour signaler une vulnérabilité, consultez [SECURITY.md](SECURITY.md).

## Contribuer

Les contributions sont bienvenues — voir [CONTRIBUTING.md](CONTRIBUTING.md) et le
[code de conduite](CODE_OF_CONDUCT.md).

## Licence

[MIT](LICENSE) — Meissa Niang.
