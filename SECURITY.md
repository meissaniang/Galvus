# Politique de sécurité

Galvus manipule des clés SSH et des accès à des serveurs : la sécurité est une
priorité du projet.

## Signaler une vulnérabilité

**N'ouvrez pas d'issue publique.**

Utilisez l'onglet [Security → Report a vulnerability](https://github.com/meissaniang/Galvus/security/advisories/new)
du dépôt, qui crée un signalement privé.

Merci d'inclure une description du problème, les étapes pour le reproduire, la
version de Galvus et votre système d'exploitation, ainsi que l'impact que vous
estimez.

Vous recevrez un accusé de réception sous **72 heures**. Une fois le correctif
publié, votre signalement sera crédité dans les notes de version, sauf si vous
préférez rester anonyme.

## Versions suivies

Le projet étant en développement actif avant la 1.0, seule la dernière version
publiée reçoit des correctifs de sécurité.

## Modèle de sécurité

Ce que Galvus garantit :

- Les **clés privées** restent dans `~/.ssh`. L'application ne les copie nulle
  part et ne les transmet à aucun service.
- Les **passphrases** ne sont jamais écrites en base ni sur disque : elles vivent
  dans le coffre natif du système via l'agent SSH, et sont transmises à
  `ssh-keygen` / `ssh-add` par un pseudo-terminal — jamais par la ligne de
  commande ni par une variable d'environnement.
- La **base locale** (serveurs, tunnels) est chiffrée au repos avec SQLCipher ;
  sa clé est générée aléatoirement et stockée dans le coffre natif.
- Le **protocole SSH** n'est pas réimplémenté : Galvus délègue à l'OpenSSH du
  système, qui reçoit les correctifs de sécurité de votre distribution.
- **Aucune donnée ne sort de la machine** : pas de compte, pas de synchronisation,
  pas de télémétrie.

Ce qui reste hors de son contrôle : la sécurité du système hôte, celle des
serveurs distants, et la robustesse des passphrases que vous choisissez.

## Vulnérabilités connues sans correctif disponible

**RUSTSEC-2024-0429** — unsoundness dans `glib::VariantStrIter` (sévérité
modérée), corrigée en amont dans `glib` 0.20.

Elle n'est pas corrigeable ici : `glib` arrive de manière transitive par
`gtk` 0.18, les liaisons Rust de GTK3 dont Tauri dépend pour sa fenêtre et ses
menus sous Linux. Ces liaisons plafonnent `glib` à `^0.18`, et il n'existe pas de
version 0.20 pour GTK3. La correction viendra du passage de Tauri à GTK4.

Portée réelle : **Linux uniquement** — macOS et Windows n'embarquent pas cette
dépendance. Galvus n'utilise par ailleurs jamais `glib` directement, et donc
jamais l'API concernée.

## Bonnes pratiques

Gardez les permissions de vos clés privées restrictives — Galvus le signale et
propose de corriger. Protégez vos clés par une passphrase, et préférez ED25519 à
RSA 2048, que l'application marque comme déprécié.
