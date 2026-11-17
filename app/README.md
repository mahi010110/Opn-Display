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
  ChatGPT (`codex.rs`) avec son relais MCP (`mcp.rs`), et les outils du lab
  avec l'accord de l'utilisateur (`outils.rs`).
- `src-tauri/` : l'app de bureau. Elle expose le moteur aux écrans par ses
  commandes : `verifier_pc`, `installer`, `arreter_installation`,
  `retirer_linux`, `installer_wsl`, puis pour le lab `etat_lab`,
  `lancer_lab`, `arreter_lab`, `demarrer_machine`, `arreter_machine`,
  `remettre_a_zero`, `veiller`, pour les terminaux `ouvrir_terminal`,
  `ecrire_terminal`, `redimensionner_terminal`, `fermer_terminal`, et pour
  l'IA `etat_ia`, `brancher_ia`, `essayer_ia`, `modeles_ia`, `regler_ia`,
  `parler_ia`, `repondre_accord`, `arreter_ia`, `oublier_conversation`. Les
  évènements : `installation`, `lab-avance`, `lab`, `ia` et `ia-accord`. Ce
  qu'affiche un terminal passe par un canal, en octets. Les réglages de l'IA
  sont dans `ia.json`, dans le dossier de configuration de l'app, et sa clé
  dans le trousseau du système. Lancée avec la variable `OPN_OUTILS`, l'app
  ne fait que relayer les outils pour Codex.
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

## Incus et le premier lab

L'installation pose Incus après Debian. Sous Windows : `apt-get install
incus` dans Debian, systemd allumé dans `/etc/wsl.conf`, puis Debian
redémarré. Sous Linux, l'app prend l'Incus du système. Elle y crée la place
de ses machines (le pool `opn-display`, un simple dossier) et y range l'image
Debian 13 d'Incus sous le nom `opn-display-debian-13`.

Le premier lab a son projet Incus, `opn-display-premier-lab`. Chaque câble
est une paire veth posée par l'app (`od1-c1a` et `od1-c1b`), dont chaque
bout entre dans une machine comme carte réseau. Les adresses sont écrites
dans la machine pour systemd-networkd, avant son premier démarrage. Sous
Linux, poser un câble demande les droits de root.

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
