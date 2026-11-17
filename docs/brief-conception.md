# Le brief de conception

Pour celui qui dessine les écrans d'Opn-Display. Écrit le 16 novembre 2026.
Il rassemble ce qui est décidé, ce qui est à proposer, et ce que le
mainteneur veut voir avant de choisir. Rien n'est codé.

## 1. Ce qu'on attend

- Une proposition de menu : par où on entre, où l'on va, comment on revient.
- Les écrans des moments de la partie 5, en maquettes, avec le matériel
  « Utilitaire 98 » d'OpenContact.
- Un dessin simple et classique. Le mainteneur l'a demandé : proche de ce
  qu'on connaît déjà dans GNS3 et Packet Tracer, avec le panneau de l'IA sur
  le côté. Pas d'idée qui demande un mode d'emploi.
- Pour chaque choix, sa raison en une phrase.

Le menu se montre d'abord, en mots et sur l'écran du lab. Le reste des écrans
vient après l'accord du mainteneur.

Les maquettes se montrent en captures : à 1366 × 768 et à 1920 × 1080, en
clair et en sombre.

## 2. Opn-Display en quelques lignes

Une app de bureau qui fait tourner une infrastructure informatique entière
sur le PC de l'utilisateur : serveurs Linux, réseau, Windows Server, pare-feu,
stockage. Une IA construit le lab, y glisse des pannes, donne des cours et
vérifie les réparations.

Elle est gratuite, en français, pour tout le monde. Elle sort en juillet 2027
sur Windows 11, Linux et macOS. Le détail est dans [projet.md](projet.md).

## 3. Ce qui est fixé

### Le style

Simple et classique (16 novembre 2026). Trois zones que l'utilisateur d'un
outil de lab connaît déjà : le schéma du lab, les terminaux, l'IA sur le
côté.

### Le matériel

Celui d'OpenContact, « Utilitaire 98 » : ses jetons (`styles/tokens/`), ses
polices (`assets/fonts/`), ses icônes (`assets/icons/`) et les règles de son
kit (`design/readme.md`).

- coins droits (0 ou 2 px), traits de 1 px à l'encre, ombres dures sans
  flou, aucun dégradé ;
- le fond de la page est un bureau, les contenus sont des fenêtres ;
- le teal pour ce qui se clique, le bleu marine seulement pour la sélection
  et les barres de titre, jamais les deux sur le même élément ;
- trois états de relief : relevé, enfoncé, en creux ;
- la trame en damier avec parcimonie, jamais derrière un texte ;
- Silkscreen pour les titres, seulement à 8, 16 ou 24 px. IBM Plex Mono pour
  les données et les commandes. Public Sans pour le texte ;
- les icônes pixelarticons, jamais d'emoji ;
- le focus en pointillé ;
- le thème sombre, obligatoire, est un anthracite aux traits clairs ;
- les objets réagissent tout de suite, seuls les passages d'un état à un
  autre glissent, et vite ;
- le ton : on tutoie, on dit les choses simplement. Les messages du système
  sont des faits, en mono.

La façon d'employer ce matériel est dans le `CLAUDE.md` d'OpenContact,
parties 4 à 7. Ce qui parle du téléphone (la zone du pouce, la barre du bas,
le clavier) ne s'applique pas : Opn-Display est une app de bureau, à la
souris et au clavier. S'appliquent ici :

- un seul bouton principal par vue ;
- un geste qu'on peut défaire se fait tout de suite, avec « Annuler » après.
  Une confirmation seulement pour ce qui ne se défait pas ;
- un message qui passe (toast) seulement si l'écran ne le montre pas déjà ;
- ce que l'écran peut montrer ne s'écrit pas en phrase ;
- des mots de tous les jours, pas d'images ;
- un état vide dit quoi faire, pas « aucune donnée ».

### Ce que le produit impose au dessin

- L'IA agit dans le lab et sur le PC. Sa liberté se règle : libre, avec
  accord, ou limitée aux outils de lab. Par défaut, sur le PC, elle demande.
- Supprimer des fichiers ou toucher aux protections de Windows demande
  toujours un oui, même en mode libre.
- Trois façons de régler l'IA, au choix : les niveaux Prof, Coach et
  Silencieuse, des réglages séparés, ou une IA qui s'adapte seule.
- L'IA écrit ses tests en posant la panne. L'app montre les tests lancés et
  leurs résultats.
- Pendant qu'elle pose une panne, l'app ne montre pas ce que fait l'IA
  (proposé dans [architecture.md](architecture.md), pas encore décidé).
- L'app sauvegarde le lab avant une panne inventée. L'utilisateur peut y
  revenir.
- L'IA peut se tromper, sur un cours comme sur une vérification. L'écran ne
  doit pas faire croire le contraire.
- L'app ne contient ni Windows Server ni ESXi. L'IA va chercher l'image quand
  elle peut, sinon elle guide l'utilisateur pour la fournir.
- Tout ne tourne pas en même temps sur un PC. L'app dit ce qui tient avant de
  lancer (proposé, pas encore décidé).
- L'app tient un journal de ce que l'IA a fait, sur le PC et dans le lab
  (proposé, pas encore décidé).
- Le mainteneur ne veut aucun script à lancer : tout se fait dans l'app.

### Ce qui n'est pas dans la première version

Le multijoueur, la voix, le lab dans le navigateur, le lab sur un serveur
distant, le serveur MCP. Le pilotage de Packet Tracer et de VMware
Workstation se construit après l'émulation : ses écrans se dessineront à ce
moment-là.

## 4. Les gens

Des personnes inventées. L'app vise tout le monde, ces quatre-là servent à
vérifier les écrans.

- Yanis, 24 ans, prépare le titre AIS en alternance. Un portable Windows 11
  de 16 Go. Il veut s'entraîner au dépannage le soir, sans formateur.
- Inès, 19 ans, débute. Elle n'a jamais ouvert un terminal. Elle veut
  comprendre sans se sentir perdue.
- Marc, 38 ans, administrateur, prépare une certification réseau. Un PC fixe
  sous Linux, 64 Go. Il veut monter un gros lab vite et que l'IA se taise.
- Sofia, 27 ans, curieuse, sur un Mac à puce Apple. Une partie des labs ne
  tourne pas chez elle, et elle doit le comprendre sans chercher.

## 5. Les moments

| Moment | Ce qu'on doit pouvoir faire |
|---|---|
| La première ouverture | L'app vérifie le PC et dit ce qu'il sait faire. Elle installe son environnement Linux, et demande avant de toucher à un réglage de Windows |
| Brancher l'IA | Choisir entre une clé API, un modèle local et l'abonnement ChatGPT, et savoir que c'est branché |
| Créer un lab | Écrire une phrase (« un AD, un DNS et un pare-feu »), voir le lab proposé, le changer, le lancer |
| Avant de lancer | Voir si le lab tient dans la RAM du PC, et quoi retirer sinon |
| Fournir une image | Comprendre pourquoi Windows Server n'est pas dans l'app, et le fournir en étant guidé |
| Travailler dans le lab | Voir le schéma, ouvrir le terminal ou l'écran d'une machine, parler à l'IA |
| Donner son accord | L'IA veut faire quelque chose sur le PC : lire ce qu'elle va faire, dire oui ou non |
| Recevoir une panne | Lire ce qui ne marche plus, chercher, demander un indice |
| Dire « c'est réparé » | Voir les tests de l'IA un par un, avec leur résultat, puis son explication |
| Revenir en arrière | Remettre une machine ou tout le lab comme avant la panne |
| Suivre un cours | Lire, et essayer dans le lab ce que le cours montre |
| Régler l'IA | Choisir une des trois façons de régler, voir et corriger sa mémoire |
| Voir ce que l'IA a fait | Ouvrir le journal de ses actions |

## 6. L'écran

- Une app de bureau, à la souris et au clavier. Pas de téléphone.
- Deux tailles pour les maquettes : 1366 × 768, un petit portable, et
  1920 × 1080. Ces tailles sont une proposition.
- L'écran du lab est dense : le schéma, un ou plusieurs terminaux, et l'IA
  doivent tenir ensemble à 1366 × 768.
- Le jour de l'examen, l'app est montrée en direct au jury. Les textes
  doivent rester lisibles quand on agrandit.
- Les commandes et les terminaux sont en IBM Plex Mono.

## 7. Les huit idées de la première fiche

Le mainteneur les a d'abord cochées en ajoutant « honnêtement je sais pas ».
Après les avoir vues en maquettes, il les a toutes choisies pour la première
version (17 novembre 2026).

Dans le lab :

- le schéma en direct : les machines et les câbles, avec ce qui diffère du
  lab prévu en rouge ;
- les paquets en animation, pas à pas ;
- le tableau blanc où l'IA dessine pour expliquer ;
- le replay d'une session.

Autour du lab :

- le compte rendu à la fin d'un lab ou d'une panne ;
- la progression : les compétences validées, ce qu'il faut réviser ;
- les parcours : je débute, je prépare une certification, je prépare un
  titre, je suis curieux ;
- le mode examen : l'IA se tait, l'utilisateur répare seul, il a le résultat
  à la fin.

Chacune se rattache aux écrans du lab sans les alourdir. Leur place est dans
[../maquettes/README.md](../maquettes/README.md).

## 8. Les points difficiles

- Trois choses sur un petit écran. Le schéma, les terminaux et l'IA à
  1366 × 768, sans que l'une des trois devienne illisible.
- Le schéma ne doit pas donner la réponse. Pendant une panne, mettre en
  rouge ce qui est cassé montre où chercher. À régler avec le mode en cours.
- L'accord sans fatigue. L'IA demande souvent. La demande doit se lire en
  deux secondes, et dire ce qui va changer sur le PC.
- Montrer les tests. L'utilisateur doit voir ce que l'IA a vérifié, et
  pouvoir douter de sa conclusion.
- Les trois façons de régler l'IA. Elles ne doivent pas faire trois écrans de
  réglages à comprendre.
- Ce que le PC ne sait pas faire. Pas de KVM, pas assez de RAM, un Mac à puce
  Apple : l'app le dit à l'endroit où ça bloque, avec ce qui reste possible.
- La première ouverture. L'installation de l'environnement Linux prend du
  temps. L'utilisateur doit savoir où elle en est et ce qu'elle change.

## 9. Ce qui reste ouvert

- Les mots de l'écran. Une proposition à faire valider : lab, machine,
  câble, panne, test, indice, cours, image.
- Les icônes des machines. Le jeu copié dans OpenContact n'a ni routeur, ni
  switch, ni serveur, ni pare-feu. Claude en a pris dans le reste de
  pixelarticons ([../maquettes/README.md](../maquettes/README.md)).
- Le logo. Opn-Display est un nom de travail.

## 10. Comment travailler

- Lire aussi [projet.md](projet.md), [architecture.md](architecture.md),
  [carnet.md](carnet.md) et [questions-ouvertes.md](questions-ouvertes.md).
- Les maquettes sont des pages HTML et CSS, sans script, dans `maquettes/`.
- Des données inventées : le lab « Agence Exemple », les machines `srv-web`,
  `srv-ad`, `pc1`, `fw1`.
- Aucune capture d'un logiciel propriétaire dans le dépôt.
- Écrire comme le dit le `CLAUDE.md`, partie « Comment on écrit ».

## 11. Ce qu'il faut éviter

- Reprendre les écrans d'OpenContact : sa barre du bas, ses feuilles de
  téléphone, ses onglets.
- Une idée de navigation qu'il faut apprendre.
- Un mur de réglages.
- Une visite guidée pour expliquer l'écran.
- Du flou, des dégradés, des coins arrondis, des emoji.
- Un écran qui fait croire que l'IA a toujours raison.
