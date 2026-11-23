# Opn-Display

Ce fichier est lu au début de chaque session. Il dit comment on travaille et
ce qui est déjà décidé. Le détail est dans `docs/`.

Opn-Display est un nom de travail. Le code est dans `app/`. L'étape 5 de
l'ordre de construction (le réseau) est en cours.

## Comment on travaille

- En français, en tutoyant, avec des réponses courtes et simples.
- Une étape à la fois. On attend l'avis du mainteneur avant de passer à la
  suivante.
- On discute avant d'agir. Rien ne se code, ne se lance ou ne se publie sans
  le feu vert du mainteneur.
- Avant de proposer quelque chose, relire `docs/projet.md`,
  `docs/architecture.md`, `docs/mesures.md`, `docs/questions-ouvertes.md` et
  `docs/carnet.md`.
- Pour dessiner un écran, lire d'abord `docs/brief-conception.md`.
- Rien n'est décidé tant que le mainteneur ne l'a pas dit. Une fiche, une
  recherche ou une proposition de Claude ne vaut pas décision.
- On ne décide pas à la place du mainteneur. Une question qu'il n'a pas
  tranchée va dans `docs/questions-ouvertes.md`, avec une recommandation.
- Les questions se posent avec l'outil à choix, jusqu'à ce qu'il ne reste
  aucune ambiguïté.
- Les questions portent sur le produit. Pas sur l'organisation du mainteneur
  (son temps, ses autres projets).
- Pas de calendrier ni de planning : on avance étape par étape, sans placer
  les choses dans le temps (17 novembre 2026).
- Une décision prise est notée dans `docs/carnet.md` avec sa date, puis
  reportée dans `projet.md` ou `architecture.md`.
- Pas de script, ni dans le dépôt ni à faire lancer au mainteneur. Ce qui
  doit se faire ou se mesurer sur un PC, c'est l'app qui le fait.
- Ce qui peut se mesurer se mesure avant d'être décidé. Une mesure va dans
  `docs/mesures.md`, avec sa date et sa source.
- Un fait pris dans une recherche se vérifie à la source avant d'être écrit
  dans la doc. Les recherches se sont déjà contredites.
- Un changement visible (un écran, un bouton) est montré au mainteneur en
  captures avant d'être gardé.
- Fidèle à la réalité : ce que l'app émule se fait comme en vrai. Quand un
  choix s'en éloigne, même un choix du mainteneur, Claude le dit et explique
  ce qui se fait en vrai (23 novembre 2026).
- Pour les maquettes, le mainteneur a laissé la main à Claude (« C'est toi
  qui as la main », 16 novembre 2026). Claude décide, montre les captures et
  note ce qu'il a retenu. Le mainteneur peut revenir dessus. Ça ne vaut pas
  pour le code, la licence ni ce qui se publie.
- Ni image propriétaire (Windows, ESXi, Cisco) ni clé API dans le dépôt.

## Le code

- Il est dans `app/` : le moteur en Rust (`moteur/`), l'app Tauri
  (`src-tauri/`), les écrans en React et TypeScript (`src/`).
  `app/README.md` dit comment il est rangé.
- Avant d'enregistrer : `cargo test -p moteur`, `cargo clippy` pour Linux
  et pour Windows (`--target x86_64-pc-windows-gnu`), `npm run build`.
- Ce qui est propre à Windows ne tourne pas dans l'environnement cloud. Il
  se compile ici, et GitHub fabrique l'installeur Windows à chaque envoi
  (`.github/workflows/app-windows.yml`). Le mainteneur l'essaie sur son PC.
- Un écran se vérifie en captures : sous Linux pour de vrai, et les écrans
  propres à Windows avec des réponses simulées du moteur.

## Comment on écrit

Le mainteneur ne veut ni texte ni code qui sentent l'IA.

Dans la doc :

- des phrases courtes et normales, comme quand on explique à un camarade ;
- pas de tiret cadratin, une virgule ou un point suffit ;
- le gras rarement, pour un mot qu'on doit retrouver d'un coup d'œil ;
- pas de phrase qui annonce la liste suivante (« Trois raisons : »), on
  donne la liste ;
- pas de formules toutes faites (« ce n'est pas X, c'est Y », « le vrai
  problème, c'est… »), ni de conclusion qui répète ce qu'on vient de dire ;
- pas de mots creux : crucial, essentiel, robuste, fluide, garantir, en
  somme, notamment ;
- pas d'images ni de métaphores, on dit la chose ;
- un tableau seulement pour comparer plusieurs choses sur plusieurs points.

Dans le code, quand il y en aura :

- un commentaire seulement quand le code ne peut pas dire pourquoi : une
  contrainte, un piège, une source. Jamais pour répéter ce que fait la ligne ;
- un nom clair plutôt qu'un commentaire ;
- pas de bandeaux, pas de code commenté laissé en place, pas d'emoji.

## Ce qui est décidé

Tout date du 16 et du 17 novembre 2026. Le détail est dans `docs/projet.md`,
l'historique dans `docs/carnet.md`.

Le produit :

- Une vraie application de bureau. Le lab, les terminaux et l'IA sont
  dedans. Un serveur MCP viendra peut-être plus tard.
- L'app fait tourner elle-même les machines du lab, comme GNS3 et EVE-NG.
- Ce qu'elle ne peut pas émuler, elle le fait faire à un logiciel installé
  sur le PC et elle le commande : Packet Tracer, VMware Workstation, Cisco
  Modeling Labs, les bacs à sable du cloud.
- L'émulation se construit d'abord, le pilotage ensuite.
- Le réseau se fait en libre : cœur, distribution, accès, EtherChannel en
  LACP, RSTP, MSTP, VRRP. L'IA dit l'équivalent Cisco. Le vrai Cisco
  (HSRP, PVST+, PAgP, les commandes Cisco des switchs) passe par Packet
  Tracer ou CML, au pilotage : Cisco interdit de faire tourner ses images
  ailleurs.
- Les trois références : GNS3 et EVE-NG pour l'émulation, Hermes Agent pour
  l'IA.
- Gratuit et libre. La licence est à choisir : le mainteneur décidera plus
  tard. Claude recommande MIT ou Apache-2.0.
- L'ordre de construction : les maquettes, puis l'app qui vérifie le PC, puis
  un petit lab Linux, puis l'IA, puis le réseau avec la vue des baies, puis
  les services et la haute disponibilité, puis les autres familles de
  machines, puis macOS et la version publique, puis le pilotage. Le détail
  est dans `docs/questions-ouvertes.md`.
- Une vraie infra, complète, comme les concurrents ou mieux, dans les
  détails aussi : l'équilibrage de charge, la redondance, la sécurité, les
  ports, les baies (17 novembre 2026). L'inventaire est dans
  `docs/infra-complete.md`.
- Les baies sont une vue physique à côté du schéma. Les images d'autres
  constructeurs tournent quand l'utilisateur les fournit et que la licence
  le permet. Les interfaces web du lab s'ouvrent dans l'app.
- Le matériel d'interface et d'UX est celui d'OpenContact, « Utilitaire 98 » :
  ses jetons, ses polices, ses icônes, ses composants et les règles de son
  kit `design/`. Les écrans d'Opn-Display se dessinent avec ce matériel.
- Un dessin simple et classique, proche de GNS3 et de Packet Tracer, avec le
  panneau de l'IA sur le côté.
- Le menu : Labs, Cours, Pannes, Progression, et Réglages à droite.
  L'écran du lab : le schéma, les terminaux, l'IA sur le côté. Le panneau
  de droite est toujours l'IA : elle y montre une discussion, une panne,
  une vérification ou un cours.
- Les écrans validés en maquette sont listés dans `maquettes/README.md`. Un
  écran nouveau part de `maquettes/` et de `maquettes/opn-display.css`.
- La technique : Tauri 2 et Rust pour l'app, le moteur et l'IA ; React et
  TypeScript pour les écrans ; Debian pour le Linux de l'app ; le matériel
  d'OpenContact copié dans le code, avec sa version notée. Les machines du
  lab tournent avec Incus, Docker viendra plus tard pour les images qui
  n'existent qu'en Docker. L'agent de l'IA est une boucle écrite à la main
  en Rust. Le reste de la pile se choisit à chaque étape, après mesure
  (`docs/architecture.md`, partie 5).
- Le nom est un nom de travail. Le dépôt est privé, le mainteneur décidera
  plus tard quand l'ouvrir.

Juillet 2027 :

- Une version publique, pour tout le monde, en français.
- Sur Windows 11, Linux et macOS.
- Tout ce qui peut s'émuler : Linux et ses services, le réseau, Windows
  Server et Active Directory, les pare-feu et la sécurité, ESXi, un SAN et
  un NAS, le WiFi simulé.
- Les huit idées de la première fiche : le schéma en direct, les paquets en
  animation, le tableau blanc, le replay, le compte rendu, la progression,
  les parcours, le mode examen.
- Le lab tourne sur le PC de l'utilisateur.
- Après l'examen : le multijoueur, la voix, le lab dans le navigateur, le lab
  sur un serveur distant.

L'IA :

- Elle construit le lab, glisse des pannes, donne des cours, explique, et dit
  si une réparation est bonne.
- C'est elle qui vérifie, avec les tests qu'elle lance.
- Les pannes viennent d'un catalogue, et elle en invente en direct.
- Elle agit sur le PC : installer des outils, piloter les logiciels, lancer
  des commandes, lire et écrire des fichiers. Elle agit aussi dans le lab.
- Se règlent : sa liberté sur le PC, sa liberté dans le lab, ce qu'elle fait
  quand on demande la solution, son initiative, sa mémoire.
- Trois façons de régler, au choix de l'utilisateur : les niveaux Prof,
  Coach et Silencieuse, des réglages séparés, ou une IA qui s'adapte seule.
  Elles portent sur le lab, la solution et l'initiative. Sa liberté sur le
  PC et sa mémoire se règlent à part.
- Elle se branche par une clé API, par un modèle local, ou par l'abonnement
  ChatGPT avec Codex. Pour l'abonnement Claude, la question est posée à
  Anthropic. Sans accord, il attend le serveur MCP.
- Les cours : des cours prêts, et des cours écrits en direct.
- Les images : elle les récupère quand elle peut, sinon l'utilisateur les
  fournit. L'app ne contient ni Windows, ni ESXi, ni image Cisco.

L'examen :

- Le projet fil rouge du mainteneur pour le titre AIS, en juillet 2027.
- Le formateur a validé le sujet. L'examen se passe sur le référentiel
  actuel.
- Le PC du développement et de la démo : Windows 11 Famille, 32 Go.

## Les garde-fous

Proposés par Claude. Le mainteneur les a retenus en disant « je te fais
confiance », il peut revenir dessus.

1. L'IA écrit ses tests au moment où elle pose la panne. L'app montre les
   tests lancés et leurs résultats.
2. L'app sauvegarde le lab avant chaque panne inventée.
3. Sur le PC, le réglage par défaut est « avec accord ». Supprimer des
   fichiers ou toucher aux protections de Windows demande toujours un oui.
4. Kali et l'IA n'attaquent que les machines du lab.

## Ce qui n'est pas décidé

- Le reste de la pile : les VM. À leur étape, après mesure.
- La licence.
- La question à Anthropic sur l'abonnement Claude.
- Les écrans du pilotage, quand il viendra.
- Le lien avec OpenContact et Opn-Link, après la première version.
- Rien n'est mesuré sur le PC du mainteneur.

La liste complète est dans `docs/questions-ouvertes.md`.

## Les autres dépôts

- `mahi010110/Opn...`, branche `claude/sweet-hypatia-p4qlsl` : les recherches
  et la première fiche du projet. La fiche sert à comprendre l'idée, elle ne
  dit pas ce qui est décidé.
- OpenContact (`mahi010110/Open-Contact`) : son kit `design/` et ses jetons
  (`styles/tokens/`) donnent le matériel « Utilitaire 98 », et son
  `CLAUDE.md` dit comment il s'emploie (parties 4 à 7). Son dossier `natif/`
  montre comment une app Tauri lance Codex pour l'abonnement ChatGPT et
  parle à Ollama.
- Opn-Link (`mahi010110/opn-link`) : la même façon de travailler et
  d'écrire.
