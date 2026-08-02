# Contribuer à Galvus

Merci de votre intérêt ! Ce document décrit comment proposer une modification.

## Avant de commencer

- **Bug ou petite correction** : ouvrez directement une pull request.
- **Nouvelle fonctionnalité** : ouvrez d'abord une
  [issue](https://github.com/meissaniang/galvus/issues) pour en discuter. Cela
  évite de développer quelque chose qui ne rentrerait pas dans le périmètre.
- **Vulnérabilité de sécurité** : ne passez pas par une issue publique, suivez
  [SECURITY.md](SECURITY.md).

## Environnement

Node.js 20+, [pnpm](https://pnpm.io), [Rust stable](https://rustup.rs) et les
[dépendances système de Tauri](https://tauri.app/start/prerequisites/).

```bash
pnpm install
pnpm tauri dev
```

## Avant de pousser

La CI exécute ces vérifications ; lancez-les en local pour éviter les allers-retours.

```bash
pnpm lint             # ESLint
pnpm format:check     # Prettier
pnpm build            # typage strict (vue-tsc) + build
pnpm test             # Vitest

cd src-tauri
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Conventions

**Commits** — format [Conventional Commits](https://www.conventionalcommits.org),
en français, à l'impératif :

```
feat(keys): génération de clés ED25519
fix(terminal): resize du PTY après un split
docs(readme): prérequis Windows
```

Préfixes utilisés : `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `ci`.

**Code** — le style est imposé par les outils : Prettier et ESLint côté frontend,
`rustfmt` et Clippy côté Rust. Écrivez du code qui ressemble à celui qui l'entoure,
et commentez le _pourquoi_ plutôt que le _quoi_.

**Architecture** — trois règles à respecter :

1. Aucun SQL dans les composants ; tout passe par un repository.
2. Aucun appel `invoke` hors du dossier `src/repositories`.
3. Toute opération sensible (fichiers, secrets, processus) vit côté Rust.

**SSH** — les connexions interactives utilisent le binaire OpenSSH du système. Ne
réimplémentez pas le protocole, et privilégiez `ssh -G` à un parseur maison pour
lire la configuration.

## Pull requests

Une PR par sujet, avec une description de ce qui change et pourquoi. Ajoutez des
tests pour toute logique non triviale, et mettez à jour le
[CHANGELOG](CHANGELOG.md) sous la section « Non publié ».
