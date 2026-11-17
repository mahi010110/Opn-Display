# L'app

Le code d'Opn-Display. La pile est dans
[../docs/architecture.md](../docs/architecture.md), partie 5.

## Les dossiers

- `moteur/` : le moteur, en Rust, sans rien de Tauri. Il vérifie le PC et
  installe le Linux de l'app. Ce qui touche à Windows est dans `windows.rs`,
  ce qui touche à Linux dans `linux.rs`. Le reste se teste partout.
- `src-tauri/` : l'app de bureau. Elle expose le moteur aux écrans par trois
  commandes : `verifier_pc`, `installer`, `installer_wsl`. L'installation
  envoie son avancée par l'évènement `installation`.
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
