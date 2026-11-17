# Les maquettes

Les écrans d'Opn-Display en HTML et CSS, faits avec le matériel
« Utilitaire 98 » d'OpenContact et des données inventées. Rien n'y marche :
on les regarde avant de coder. Le brief est dans
[../docs/brief-conception.md](../docs/brief-conception.md).

Un écran a un état : « validé » quand le mainteneur l'a dit, « retenu » quand
il a laissé Claude décider, « proposé » tant que personne n'a tranché.

Un écran s'ouvre dans un navigateur, depuis le disque. Il n'y a aucun script.

## Les fichiers

- `kit/` : le matériel d'OpenContact, copié le 16 novembre 2026 depuis son
  dépôt, à la version `a1d317b`. Ses jetons sont dans `kit/styles/tokens/`,
  ses polices et ses icônes dans `kit/assets/`. `kit/composants.css` réunit
  le style des composants de son kit (`design/components/`).
- `icones/` : les icônes qui manquent dans OpenContact, prises dans le même
  jeu, pixelarticons 2.4.2, sous licence MIT.
- `icones.css` : les icônes utilisées, écrites dans la feuille. Un masque
  pris dans un fichier ne se charge pas quand la page est ouverte depuis le
  disque.
- `opn-display.css` : les pièces d'Opn-Display, faites avec les jetons.
- `ecrans/` : un fichier par écran.
- `captures/` : chaque écran à 1366 × 768 et à 1920 × 1080, en clair et en
  sombre.

## Le menu

Validé le 16 novembre 2026. La Progression s'y ajoute le 17, avec les huit
idées.

- Quatre entrées en haut à gauche : Labs, Cours, Pannes, Progression.
  Réglages à droite.
- Labs ouvre la liste des labs. Un lab ouvert prend tout l'écran, et « Mes
  labs » revient à la liste.
- Un cours ou une panne se choisit dans sa liste, puis s'ouvre dans un lab.
- Progression montre ce que tu sais faire, ton parcours et ce qu'il faut
  revoir.

## Les écrans

| Écran | Le moment | État |
|---|---|---|
| `lab` | travailler dans le lab, et donner son accord à l'IA | validé |
| `labs` | choisir un lab, voir ce que le PC sait faire | validé |
| `nouveau-lab` | décrire un lab, voir le lab proposé et sa RAM, le lancer | validé |
| `panne` | recevoir une panne, demander un indice | validé |
| `repare` | dire « c'est réparé », voir les tests et l'explication | validé |
| `cours` | suivre un cours dans le lab | validé |
| `ouverture` | la première ouverture : ce que le PC sait faire, ce que l'app installe | retenu |
| `installation` | suivre l'installation | retenu |
| `ia-brancher` | brancher l'IA | retenu |
| `ia-regler` | régler l'IA | retenu |
| `ia-memoire` | voir, corriger, couper ou effacer sa mémoire | retenu |
| `ia-journal` | voir ce que l'IA a fait | retenu |
| `cours-liste` | choisir un cours, en demander un autre à l'IA | retenu |
| `pannes-liste` | choisir une panne du catalogue, ou en demander une inventée | retenu |
| `image` | fournir l'image de Windows Server | retenu |
| `schema-direct` | le schéma en direct : ce qui diffère du lab prévu passe en rouge | validé |
| `paquets` | les paquets en animation, pas à pas | validé |
| `tableau` | le tableau blanc où l'IA dessine | validé |
| `replay` | revoir une session | validé |
| `compte-rendu` | le compte rendu à la fin d'une panne | validé |
| `progression` | ce qu'on sait faire, son parcours, ce qu'il faut revoir | retenu |
| `parcours` | choisir pourquoi on est là, et la suite proposée | retenu |
| `examen` | le mode examen | validé |
| `examen-resultat` | le résultat de l'examen | retenu |

## L'écran du lab

- À gauche en haut, le schéma. Une colonne « Ajouter » donne les machines à
  poser : serveur, poste, routeur, switch, pare-feu, stockage.
- À gauche en bas, les terminaux, un onglet par machine. Sur un grand écran,
  deux terminaux tiennent côte à côte.
- À droite, l'IA : son niveau, la discussion, la demande d'accord, le champ
  pour lui écrire.
- En bas, la barre d'état : la RAM que prend le lab, le réglage de l'IA,
  l'heure du dernier enregistrement.
- La fenêtre où l'on tape a sa barre de titre en bleu marine. Les autres ont
  une barre en trame.
- Une machine en marche a un carré vert. Une machine éteinte est en trame. La
  machine dont le terminal est ouvert a son nom en bleu marine.

## Ce que l'IA montre à droite

Le panneau de droite est toujours l'IA. Ce qu'elle y montre change.

- La discussion, et une demande d'accord quand elle veut agir sur le PC.
- Une panne : la carte reste en haut du panneau, avec « Un indice » et
  « C'est réparé ». Les tests sont écrits mais cachés jusqu'à la
  vérification. Le schéma ne montre rien en rouge.
- Une vérification : chaque test, sa commande et son résultat, puis
  l'explication de la panne. Une ligne rappelle que l'IA peut se tromper.
- Un cours : l'étape, une commande à taper dans une machine d'un geste,
  « Précédent » et « Suivant ».

La pastille à côté du nom du lab dit ce qui se passe : le nombre de machines
en marche, une panne en cours, une panne réparée, un cours.

## La première ouverture

- Le menu n'est pas encore là : il arrive quand l'app est installée.
- À gauche, ce que le PC sait faire. Ce qui manque est dit avec sa raison.
- À droite, ce que l'app installe. Le réglage de la mémoire de WSL2 est une
  case à cocher, vide au départ, avec la ligne que l'app écrira.
- Pendant l'installation, chaque étape a un carré : plein quand elle est
  faite, en trame quand elle est en cours, vide quand elle attend.

## Les réglages

- Une liste à gauche : Brancher l'IA, Régler l'IA, Sa mémoire, Son journal,
  Images, Ton PC, Thème.
- Brancher l'IA : une clé API, un modèle local, ou l'abonnement ChatGPT. Le
  choix retenu s'ouvre, les autres disent leur état en une ligne.
- Régler l'IA, en deux cadres. « Sur ton PC » : avec accord, libre, ou
  seulement les outils de lab. « Avec toi » : la façon de régler (par niveau,
  un par un, ou elle s'adapte), puis ce que ça donne dans le lab, pour la
  solution et pour son initiative.
- Sa liberté sur le PC et sa mémoire restent des réglages à part, quelle que
  soit la façon de régler.
- Sa mémoire : un interrupteur, les lignes qu'elle garde, qu'on corrige ou
  qu'on retire une à une, et « Effacer sa mémoire » à part, en rouge.
- Son journal : l'heure, l'endroit, ce qu'elle a fait, et l'accord donné ou
  refusé.

## Les cours, les pannes, une image

- Cours : le cours commencé en haut, avec « Reprendre ». Dessous, les cours
  prêts, filtrés par famille. À droite, un champ pour demander un autre
  sujet à l'IA.
- Pannes : le catalogue dit ce qui ne marche plus, jamais la cause. À
  droite, « Une panne inventée » : l'IA casse quelque chose dans le lab
  ouvert, au niveau choisi.
- Une image que l'app ne peut pas contenir se fournit en trois pas
  numérotés, dans une fenêtre posée sur le lab. Chaque pas porte son bouton.
  On peut lancer le lab sans la machine.

## Les huit idées

Elles viennent de la première fiche. Le mainteneur les a choisies le
17 novembre 2026, après les avoir vues ici, une par écran. Claude les a
ensuite reliées aux autres écrans.

- Le schéma en direct : en haut à droite du schéma, « Comparer au lab
  prévu ». Ce qui diffère passe en rouge, avec le nombre d'écarts. Il n'est
  pas proposé pendant une panne ni pendant un examen.
- Les paquets : « Voir les paquets », à côté. La fenêtre du bas passe des
  terminaux aux paquets. La liste dit qui parle à qui et ce que le message
  veut dire. Un carré bleu marine montre le paquet sur le schéma.
- Le tableau blanc : un onglet « Tableau » à côté de « Schéma », dans la
  même fenêtre. Il n'apparaît que quand l'IA a dessiné quelque chose.
- Le replay : « Revoir la session », depuis le compte rendu ou le résultat
  d'un examen. Le panneau de droite devient le déroulé, le schéma et les
  terminaux montrent le moment choisi.
- Le compte rendu : après la vérification, l'IA propose « Lire le compte
  rendu ». Une fenêtre posée sur le lab, avec la durée, les indices, les
  tests, la cause, ce qui a été fait et ce qu'il faut revoir.
- La progression : une entrée du menu. Un niveau sur quatre pavés par
  savoir-faire, le parcours avec ses trois prochaines étapes, et ce qu'il
  faut revoir.
- Les parcours : « Changer », dans la Progression. Quatre raisons d'être
  là, et la suite proposée, qui change avec le choix.
- Le mode examen : depuis Pannes, « Un examen », avec le nombre de pannes et
  le temps. Pas d'indice, l'IA se tait. Le résultat arrive à la fin, panne
  par panne, avec la cause de chacune.

## Les icônes des machines

| Machine | Icône de pixelarticons |
|---|---|
| Serveur | `server` |
| Poste | `monitor` |
| Routeur | `move` |
| Switch | `arrows-horizontal` |
| Pare-feu | `wall` |
| Stockage | `database` |
| Internet | `cloud` |
