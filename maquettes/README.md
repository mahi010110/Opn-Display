# Les maquettes

Les écrans d'Opn-Display en HTML et CSS, faits avec le matériel
« Utilitaire 98 » d'OpenContact et des données inventées. Rien n'y marche :
on les regarde avant de coder. Ce sont des propositions, rien n'est décidé.
Le brief est dans [../docs/brief-conception.md](../docs/brief-conception.md).

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

## Le menu proposé

- Trois entrées en haut à gauche : Labs, Cours, Pannes. Réglages à droite.
- Labs ouvre la liste des labs. Un lab ouvert prend tout l'écran, et « Mes
  labs » revient à la liste.
- Un cours ou une panne se choisit dans sa liste, puis s'ouvre dans un lab.
- La progression n'a pas d'entrée tant qu'elle n'est pas décidée.

## Les écrans

| Écran | Le moment |
|---|---|
| `lab` | travailler dans le lab, et donner son accord à l'IA |

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
