# Opn-Display

Ce fichier est lu au début de chaque session. Il dit comment on travaille et
ce qui est déjà décidé. Le détail est dans `docs/`.

Opn-Display est un nom de travail. Rien n'est codé.

## Comment on travaille

- En français, en tutoyant, avec des réponses courtes et simples.
- Une étape à la fois. On attend l'avis du mainteneur avant de passer à la
  suivante.
- On discute avant d'agir. Rien ne se code, ne se lance ou ne se publie sans
  le feu vert du mainteneur.
- Avant de proposer quelque chose, relire `docs/projet.md`,
  `docs/architecture.md`, `docs/mesures.md`, `docs/questions-ouvertes.md` et
  `docs/carnet.md`.
- Rien n'est décidé tant que le mainteneur ne l'a pas dit. Une fiche, une
  recherche ou une proposition de Claude ne vaut pas décision.
- On ne décide pas à la place du mainteneur. Une question qu'il n'a pas
  tranchée va dans `docs/questions-ouvertes.md`, avec une recommandation.
- Les questions se posent avec l'outil à choix, jusqu'à ce qu'il ne reste
  aucune ambiguïté.
- Les questions portent sur le produit. Pas sur l'organisation du mainteneur
  (son temps, ses autres projets).
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
- Ni image propriétaire (Windows, ESXi, Cisco) ni clé API dans le dépôt.

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

Tout date du 16 novembre 2026. Le détail est dans `docs/projet.md`,
l'historique dans `docs/carnet.md`.

Le produit :

- Une vraie application de bureau. Le lab, les terminaux et l'IA sont
  dedans. Un serveur MCP viendra peut-être plus tard.
- L'app fait tourner elle-même les machines du lab, comme GNS3 et EVE-NG.
- Ce qu'elle ne peut pas émuler, elle le fait faire à un logiciel installé
  sur le PC et elle le commande : Packet Tracer, VMware Workstation, Cisco
  Modeling Labs, les bacs à sable du cloud.
- L'émulation se construit d'abord, le pilotage ensuite.
- Les trois références : GNS3 et EVE-NG pour l'émulation, Hermes Agent pour
  l'IA.
- Gratuit et libre. La licence reste à choisir.
- Un design propre à Opn-Display, qui ne reprend pas celui d'OpenContact.
- Le nom est un nom de travail. Le dépôt est privé, le mainteneur décidera
  plus tard quand l'ouvrir.

Juillet 2027 :

- Une version publique, pour tout le monde, en français.
- Sur Windows 11, Linux et macOS.
- Tout ce qui peut s'émuler : Linux et ses services, le réseau, Windows
  Server et Active Directory, les pare-feu et la sécurité, ESXi, un SAN et
  un NAS, le WiFi simulé.
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
- Elle se branche par abonnement, par clé API ou par modèle local.
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

- L'architecture et la pile. `docs/architecture.md` est une proposition.
- L'ordre de construction et la licence. Claude doit les proposer.
- L'abonnement Claude dans l'app. Les règles d'Anthropic ne laissent que le
  serveur MCP.
- Les idées de la première fiche qui n'ont pas été rediscutées : les
  espaces, les modes, le schéma en direct, la progression.
- Rien n'est mesuré sur le PC du mainteneur.

La liste complète est dans `docs/questions-ouvertes.md`.

## Les autres dépôts

- `mahi010110/Opn...`, branche `claude/sweet-hypatia-p4qlsl` : les recherches
  et la première fiche du projet. La fiche sert à comprendre l'idée, elle ne
  dit pas ce qui est décidé.
- OpenContact (`mahi010110/Open-Contact`) : son dossier `natif/` montre
  comment une app Tauri lance Codex pour l'abonnement ChatGPT et parle à
  Ollama. Son design ne s'applique pas ici.
- Opn-Link (`mahi010110/opn-link`) : la même façon de travailler et
  d'écrire.
