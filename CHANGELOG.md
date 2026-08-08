# Journal des modifications

Le format suit [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/) et le
projet applique le [versionnage sémantique](https://semver.org/lang/fr/).

## [Non publié]

### Modifié

- **Terminal refondu dans l'esprit de Termius, sur la palette Material Dark.**
  Les seize couleurs ANSI sont désormais définies : xterm retombait jusqu'ici
  sur ses valeurs héritées du VGA, qui rendaient la sortie de `ls`, `git` ou
  `htop` criarde. L'en-tête qui
  coiffait chaque pane a disparu — la sortie occupe toute la surface — et le
  pane actif se signale par un liseré interne qui ne décale pas la grille de
  caractères. Onglets plats, barre d'état allégée.
- Le rendu passe sur le GPU (WebGL), avec repli automatique sur le rendu DOM
  quand l'accélération n'est pas disponible.
- **Fiche serveur allégée** : les tags et les pastilles de couleur quittent le
  formulaire, qu'ils encombraient à la création pour un usage rare. Les valeurs
  déjà enregistrées sont conservées, continuent de s'afficher et suivent les
  sauvegardes ; seule leur saisie disparaît. À défaut de couleur choisie, la
  pastille reprend une teinte dérivée du nom.

### Ajouté

- **Recherche dans le terminal** (⌘F) avec compteur d'occurrences, navigation
  par ⏎ / ⇧⏎ et surlignage des correspondances.
- **Séparateurs de split redimensionnables** à la souris.
- **URL cliquables** dans la sortie, ouvertes dans le navigateur du système.
- **⌘1…9** pour aller directement à un onglet.
- **Les `Include` du `~/.ssh/config` sont suivis.** Les hôtes déclarés dans un
  fichier inclus — `Include config.d/*` — n'apparaissaient pas dans Galvus alors
  qu'ils fonctionnaient avec `ssh`. Ils sont désormais listés, rangés par
  fichier de déclaration puis par groupe : un hôte du fichier de configuration
  affiche `~/.ssh/config.d/work › Prod` au lieu d'un « Sans groupe » qui les
  mélangeait tous. Les modifications visent le fichier qui déclare l'hôte, et
  non plus le fichier principal — ce qui y aurait écrit un doublon masquant
  l'original.
- **Logo du système à la place des initiales.** Le système est reconnu dans la
  bannière affichée par OpenSSH à la connexion — « Welcome to Ubuntu 22.04.5
  LTS » — et la pastille du serveur porte alors le logo de la distribution, sur
  les cartes, dans la liste et dans les onglets du terminal. Dix-neuf systèmes
  reconnus. Rien n'est exécuté sur la machine distante pour cela ; un serveur
  sans bannière garde ses initiales, et le système reste renseignable à la main
  dans la fiche. Il est conservé avec le reste : en base pour les serveurs
  locaux, dans le commentaire `# galvus:` pour les hôtes du fichier de
  configuration.

## [0.2.0] — 2026-08-02

Les hôtes du `~/.ssh/config` deviennent des serveurs à part entière, et la
configuration peut être sauvegardée ou transportée.

### Ajouté

- **Hôtes du fichier de configuration au même rang que les autres** : même
  carte, mêmes groupes, mêmes actions. Ils acceptent désormais un groupe, une
  couleur, des tags et le statut favori, conservés dans un commentaire
  `# galvus:` au-dessus du bloc — invisible pour OpenSSH, et qui suit le fichier
  lors d'une sauvegarde.
- **Création directe dans `~/.ssh/config`** : un sélecteur d'emplacement permet
  de choisir entre la base chiffrée et le fichier de configuration.
- **Déplacement entre les deux emplacements** depuis le dialog d'édition, sans
  ressaisie.
- **Sauvegarde et restauration** de la configuration dans un fichier unique
  (Paramètres › Avancé) : serveurs, hôtes du fichier de configuration, tunnels
  et préférences. Aucun secret n'y figure — ni clé privée, ni passphrase.
  L'import est additif et n'écrase jamais une entrée existante.
- **Installation par Homebrew** sur macOS, qui lève la mise en quarantaine et
  évite l'avertissement au premier lancement.
- **Notification des nouvelles versions** : un bandeau discret signale une
  version plus récente et renvoie à sa page de publication. C'est la seule
  requête réseau de l'application — une interrogation quotidienne de l'API
  publique de GitHub, sans aucune donnée transmise, désactivable dans
  Paramètres › Avancé.

### Modifié

- Le tri s'applique aussi aux hôtes du fichier de configuration.
- Les installeurs portent leur système dans leur nom
  (`Galvus_0.2.0_x64-setup-windows.exe`).

### Corrigé

- Les contrôles des barres d'outils ne répondaient plus : la zone de
  déplacement de fenêtre captait les clics destinés aux menus et boutons.
- Le déplacement de la fenêtre, inopérant avec la barre de titre intégrée.
- Compilation sur Windows et Linux, impossible car le coffre natif était figé
  sur l'implémentation macOS.
- Message trompeur « Galvus est endommagé » sur macOS : la marche à suivre est
  désormais documentée.

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

[Non publié]: https://github.com/meissaniang/Galvus/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/meissaniang/Galvus/releases/tag/v0.2.0
[0.1.0]: https://github.com/meissaniang/Galvus/releases/tag/v0.1.0
