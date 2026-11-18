# L'app

Le code d'Opn-Display. La pile est dans
[../docs/architecture.md](../docs/architecture.md), partie 5.

## Les dossiers

- `moteur/` : le moteur, en Rust, sans rien de Tauri. Il vérifie le PC,
  installe le Linux de l'app et Incus, et fait tourner le premier lab. Ce qui
  touche à Windows est dans `windows.rs`, ce qui touche à Linux dans
  `linux.rs`. Les commandes qui tournent là où vivent les machines passent
  par `systeme.rs` : sous Windows, `wsl.exe --exec` dans Debian. Le lab est
  dans `lab.rs`, Incus dans `incus.rs`, les terminaux dans `terminal.rs`, la
  veille sur Incus dans `veille.rs`. L'IA est dans `ia/` : la boucle de
  l'agent (`mod.rs`), le format d'OpenAI qui sert aussi pour Ollama
  (`openai.rs`), celui d'Anthropic (`anthropic.rs`), Codex pour l'abonnement
  ChatGPT (`codex.rs`) avec son relais MCP (`mcp.rs`), les outils du lab
  avec l'accord de l'utilisateur (`outils.rs`), les trois façons de la
  régler et l'IA qui s'adapte (`reglage.rs`), sa mémoire (`memoire.rs`) et
  son journal (`journal.rs`). Les pannes sont dans
  `pannes.rs` : la pose, l'instantané du lab, les tests et le retour en
  arrière. Le catalogue du premier lab est `pannes/premier-lab.json`. Un
  lab proposé par l'IA est vérifié et placé sur le schéma dans
  `proposition.rs`, avec l'aide de `reseau.rs` pour les adresses.
- `src-tauri/` : l'app de bureau. Elle expose le moteur aux écrans par ses
  commandes : `verifier_pc`, `installer`, `arreter_installation`,
  `retirer_linux`, `installer_wsl`, puis pour les labs `etat_labs`,
  `ouvrir_lab`, `etat_lab`, `lancer_lab`, `arreter_lab`, `demarrer_machine`,
  `arreter_machine`, `remettre_a_zero`, `retirer_du_lab`, `creer_lab`,
  `supprimer_lab`, `veiller`, pour les terminaux `ouvrir_terminal`,
  `ecrire_terminal`, `redimensionner_terminal`, `fermer_terminal`, et pour
  l'IA `etat_ia`, `brancher_ia`, `essayer_ia`, `modeles_ia`, `regler_ia`,
  `memoire_ia`, `activer_memoire`, `corriger_memoire`,
  `retirer_de_la_memoire`, `effacer_memoire`, `journal_ia`, `parler_ia`,
  `aider_ia`, `repondre_accord`, `arreter_ia`, `oublier_conversation`,
  `inventer_panne`, `proposer_lab`, `oublier_proposition`, et pour les
  pannes `catalogue_pannes`, `poser_panne`,
  `panne_en_cours`, `indice_panne`, `verifier_panne`, `abandonner_panne`,
  `fermer_panne`. Les évènements : `installation`, `lab-avance`, `lab`,
  `ia`, `ia-accord` et `panne`. Ce qu'affiche un terminal passe par un
  canal, en octets. Les réglages de l'IA sont dans `ia.json`, sa mémoire
  dans `memoire.json` et la panne posée dans `panne.json`, dans le dossier
  de configuration de l'app ; sa clé est dans le trousseau du système. Le
  journal de l'IA (`journal.json`), les labs construits (`labs/`) et la
  date d'ouverture de chaque lab (`ouverts.json`) sont dans son dossier de
  données.
  Lancée avec la variable `OPN_OUTILS`, l'app ne fait que relayer les outils
  pour Codex.
- `src/` : les écrans, en React et TypeScript. Ils reprennent les maquettes
  validées (`../maquettes/`).
- `src/kit/` : le matériel d'OpenContact, copié depuis son dépôt à la
  version `a1d317b` : les jetons (`styles/tokens/`), les polices
  (`assets/fonts/`, licence OFL) et le style de ses composants
  (`composants.css`). Quand OpenContact change, on le recopie et on note la
  nouvelle version ici.
- `src/styles/` : les pièces d'Opn-Display (`opn-display.css`) et les
  icônes pixelarticons (`icones.css`, licence MIT), reprises des maquettes.

## Le Linux de l'app sous Windows

L'app lit la liste des distributions de WSL tenue par Microsoft
(`DistributionInfo.json`), y prend l'image de Debian fabriquée par le projet
Debian, la télécharge, vérifie son empreinte SHA-256, puis l'importe dans
WSL2 sous le nom `Opn-Display`. Le disque est rangé dans le dossier de l'app,
sous `AppData\Local`.

## Incus et les labs

L'installation pose Incus après Debian. Sous Windows : `apt-get install
incus` dans Debian, systemd allumé dans `/etc/wsl.conf`, puis Debian
redémarré. Sous Linux, l'app prend l'Incus du système. Elle y crée la place
de ses machines (le pool `opn-display`, un simple dossier) et y range l'image
Debian 13 d'Incus sous le nom `opn-display-debian-13`.

Le premier lab est écrit dans le moteur. Ceux que l'IA construit sont
rangés chacun dans un fichier JSON, sous `labs/` dans le dossier des
données de l'app. Chaque lab a son numéro, qui entre dans le nom de ses
câbles, et son projet Incus : `opn-display-premier-lab`,
`opn-display-lab-2`…

Chaque câble est une paire veth posée par l'app (`od1-c1a` et `od1-c1b`
pour le premier câble du lab 1), dont chaque bout entre dans une machine
comme carte réseau. Les adresses sont écrites dans la machine pour
systemd-networkd, avant son premier démarrage. Un switch est un conteneur
Debian dont les ports entrent dans un pont Linux, `br0`. Sous Linux, poser
un câble demande les droits de root.

## Les pannes

Une panne du catalogue dit ce que l'utilisateur constate, les commandes qui
la posent, ses tests, ses indices et son explication. Un test est une
commande qui réussit (code 0) quand tout marche.

Avant de poser une panne, l'app prend un instantané Incus de chaque machine
(`avant-panne`). Elle lance ensuite les commandes de pose, puis les tests :
si aucun n'échoue, la panne n'a rien cassé et le lab revient à l'instantané.
« Abandonner » y revient aussi. Une panne réparée efface l'instantané.

Une panne inventée passe par l'outil `poser_une_panne`, que l'IA n'a que
quand l'utilisateur en demande une. Elle prend le même chemin.

La panne posée est gardée dans `panne.json`, dans le dossier de
configuration de l'app, et reprise à l'ouverture. Le fichier part quand la
panne est réparée, abandonnée, ou que le lab est remis à zéro.

## Le fabriquer

Il faut Rust, Node 22, et sous Linux les bibliothèques de Tauri
(`libwebkit2gtk-4.1-dev` et les autres de la doc de Tauri).

```
npm install
cargo test -p moteur
npm run tauri dev
```

`npm run tauri dev` ouvre l'app avec les écrans rechargés à chaud.
L'installeur Windows est fabriqué par GitHub (`.github/workflows/app-windows.yml`)
à chaque envoi qui touche `app/`.
