# Les maquettes

Les écrans d'Opn-Display en HTML et CSS, faits avec le matériel
« Utilitaire 98 » d'OpenContact et des données inventées. Rien n'y marche :
on les regarde avant de coder. Un écran est une proposition tant que le
mainteneur ne l'a pas validé. Le brief est dans [../docs/brief-conception.md](../docs/brief-conception.md).

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

Validé le 16 novembre 2026.

- Trois entrées en haut à gauche : Labs, Cours, Pannes. Réglages à droite.
- Labs ouvre la liste des labs. Un lab ouvert prend tout l'écran, et « Mes
  labs » revient à la liste.
- Un cours ou une panne se choisit dans sa liste, puis s'ouvre dans un lab.
- La progression n'a pas d'entrée tant qu'elle n'est pas décidée.

## Les écrans

| Écran | Le moment | État |
|---|---|---|
| `lab` | travailler dans le lab, et donner son accord à l'IA | validé |
| `labs` | choisir un lab, voir ce que le PC sait faire | validé |
| `nouveau-lab` | décrire un lab, voir le lab proposé et sa RAM, le lancer | validé |
| `panne` | recevoir une panne, demander un indice | validé |
| `repare` | dire « c'est réparé », voir les tests et l'explication | validé |
| `cours` | suivre un cours dans le lab | validé |
| `ouverture` | la première ouverture : ce que le PC sait faire, ce que l'app installe | proposé |
| `installation` | suivre l'installation | proposé |
| `ia-brancher` | brancher l'IA | proposé |
| `ia-regler` | régler l'IA | proposé |
| `ia-memoire` | voir, corriger, couper ou effacer sa mémoire | proposé |
| `ia-journal` | voir ce que l'IA a fait | proposé |

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
  soit la façon de régler. C'est une proposition.
- Sa mémoire : un interrupteur, les lignes qu'elle garde, qu'on corrige ou
  qu'on retire une à une, et « Effacer sa mémoire » à part, en rouge.
- Son journal : l'heure, l'endroit, ce qu'elle a fait, et l'accord donné ou
  refusé.

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
