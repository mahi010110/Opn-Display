# Carnet

Les décisions dans l'ordre où elles ont été prises. Si ce carnet ne dit pas
la même chose que [projet.md](projet.md) ou
[architecture.md](architecture.md), ce sont eux qui ont raison.

## Du 12 au 15 novembre 2026 : l'idée

Repris du carnet d'une première discussion (dépôt `Opn...`, branche
`claude/sweet-hypatia-p4qlsl`, fichier
`docs/projets/carnet-de-discussion.md`), pour la partie qui concerne le lab.
Le projet s'appelait alors « projet 2, le lab IA ».

- Le mainteneur veut une app qui réunit les supports de sa formation (Packet
  Tracer, VMware Workstation, bacs à sable du cloud) et fait tout tourner,
  « même VM, ESXi, SAN, NAS, WiFi ». Une IA y construit le lab avec lui ou à
  sa place, glisse des pannes, donne des cours et explique avec un visuel en
  direct.
- Deux idées ont été comparées : l'app fait tout tourner elle-même (celle du
  mainteneur), ou l'app pilote les outils existants (celle de Claude). Le
  mainteneur garde la sienne.
- Il décrit le projet comme un mélange de GNS3, d'EVE-NG et de Hermes Agent.
- L'app doit servir à tout le monde, pas seulement à sa formation.
- Il en fait son projet fil rouge pour le titre AIS, examen en juillet 2027.
  C'est lui qui choisit le sujet.
- Le multijoueur lui plaît, pour plus tard.
- Deux recherches ont été faites le 13 novembre. Ce qu'elles disent :
  l'app n'existe nulle part en entier, GNS3 3.1 en est le plus proche pour le
  réseau, et plusieurs limites tiennent à la RAM, aux licences et aux
  processeurs.
- Une fiche a été écrite le 15 novembre, avec une première version en deux
  temps.

## 16 novembre 2026 : le projet démarre dans son dépôt

Le dépôt s'appelle `Opn-Display`. Il est privé.

### La fiche ne vaut pas décision

Claude a lu la doc et présenté comme décidé ce que la fiche contenait. Le
mainteneur a corrigé : il a transmis la fiche pour faire comprendre l'idée,
et il n'est pas d'accord avec plusieurs de ses choix. Rien n'est décidé tant
qu'il ne l'a pas dit.

### Les réponses du mainteneur

Claude a posé ses questions par séries, avec des réponses à choisir.

| Sujet | La fiche du 15 novembre | Décidé le 16 novembre |
|---|---|---|
| Qui dit que c'est réparé | Des scripts, jamais l'IA | L'IA, avec les tests qu'elle lance |
| Les pannes | Toutes essayées sur une copie du lab avant | Un catalogue, et des pannes inventées en direct |
| L'IA sur le PC | Jamais | Oui : installer, piloter, lancer des commandes, lire et écrire des fichiers. Réglable |
| Le moteur | L'app fait tout tourner, et se branche ailleurs quand c'est inévitable | Pareil. L'émulation se construit d'abord, le pilotage ensuite |
| Pour juillet 2027 | Conteneurs et réseau, puis AD, pare-feu et Wazuh | Tout ce qui peut s'émuler, ESXi, SAN, NAS et WiFi compris |
| Les systèmes | Windows 11 et Linux, macOS si possible | Windows 11, Linux et macOS |
| Qui l'utilise en juillet | La classe, en test | Tout le monde, version publique |
| Les réglages de l'IA | Trois niveaux : Prof, Coach, Silencieuse | Trois façons au choix : ces niveaux, des réglages séparés, ou une IA qui s'adapte seule |
| Le pilotage | Après la première version | Après l'émulation |

Les autres réponses :

- l'action de l'IA dans le lab, la solution qu'elle donne ou non, son
  initiative et sa mémoire sont réglables ;
- l'IA se branche par abonnement, par clé API ou par modèle local ;
- les cours : des cours prêts, et des cours écrits en direct ;
- les images : l'IA les récupère quand elle peut, sinon l'utilisateur les
  fournit ;
- le lab tourne sur le PC seulement. Le navigateur et le serveur distant
  viennent après l'examen ;
- le multijoueur et la voix viennent après l'examen ;
- l'app sort en français ;
- le design est propre à Opn-Display ;
- Opn-Display est un nom de travail ;
- le dépôt reste privé, le mainteneur décidera plus tard ;
- le formateur a vu et validé le sujet ;
- le centre a confirmé que l'examen se passe sur le référentiel actuel ;
- la formation se fait en centre, avec un stage ;
- le PC du mainteneur : Windows 11 Famille, 32 Go ;
- Claude doit proposer l'ordre de construction et la licence.

Le mainteneur ne veut pas de questions sur son organisation (son temps, ses
autres projets). Claude s'en tient au produit.

### L'abonnement, comme dans OpenContact

Claude avait écrit qu'avec un abonnement, l'IA ne pourrait pas parler dans le
panneau de l'app. Le mainteneur a rappelé que c'est déjà fait dans
OpenContact. Vérifié dans `natif/src-tauri/src/ia.rs` : l'app y lance Codex,
l'outil officiel, pour l'abonnement ChatGPT. La même voie est à vérifier pour
Claude.

### Les garde-fous

Claude a donné quatre conseils. Le mainteneur a répondu : « je sais pas, je
te fais confiance ». Ils sont retenus, et il peut revenir dessus.

- L'IA écrit ses tests au moment où elle pose la panne. L'app montre les
  tests lancés et leurs résultats.
- L'app sauvegarde le lab avant chaque panne inventée.
- Sur le PC, le réglage par défaut est « avec accord ». Supprimer des
  fichiers ou toucher aux protections de Windows demande toujours un oui.
- Kali et l'IA n'attaquent que les machines du lab.

### Les références

Le mainteneur donne trois outils à comparer : GNS3 et EVE-NG, qui font déjà
l'émulation, et Hermes Agent pour l'IA. La comparaison est dans
[projet.md](projet.md), partie 2, et ce qu'on leur reprend dans
[architecture.md](architecture.md).

### La doc

Écrite ce jour : [projet.md](projet.md), [architecture.md](architecture.md),
[questions-ouvertes.md](questions-ouvertes.md) et ce carnet. Le mainteneur
l'a validée (« C'est bon »).

### Les mesures

Faites le même jour depuis l'environnement cloud, à la demande du mainteneur.
Le détail est dans [mesures.md](mesures.md).

- WSL2 marche sur Windows 11 Famille, avec KVM, le réseau virtuel et l'iSCSI
  dans son noyau. Le WiFi simulé demande le noyau 6.18.
- Par défaut, WSL2 ne prend que la moitié de la RAM du PC.
- Anthropic interdit à une app tierce de proposer la connexion claude.ai ou
  de faire passer des requêtes par l'abonnement. La voie sûre pour
  l'abonnement Claude est le serveur MCP.
- GNS3 a 858 pannes de réseau, sous forme de descriptions. Aucune ne dit
  comment vérifier la réparation.
- Sous Linux : 300 machines reliées en 4 secondes, 18 Mo par routeur FRR, un
  disque de VM remis à zéro en moins d'une seconde.
- Rien n'est mesuré sur le PC du mainteneur. Ce qui reste est dans
  [mesures.md](mesures.md), partie 8.

### Une vraie application, pas de script

Après les mesures, Claude recommandait un script pour lire trois valeurs sur
le PC du mainteneur, et le serveur MCP pour l'abonnement Claude. Le
mainteneur a répondu : « Non, je veux une véritable application. Peut-être
plus tard un MCP. Je veux pas de script, rien du tout. »

- Opn-Display est une vraie application. Le lab, les terminaux et l'IA sont
  dedans.
- Le serveur MCP viendra peut-être plus tard. Il n'est pas dans la première
  version.
- Pas de script. Le dossier `mesures/` et ses trois scripts sont retirés du
  dépôt. Claude ne prépare aucun script pour le PC du mainteneur.
- Ce qui reste à mesurer sur son PC, l'app le mesurera elle-même.
- La seule voie sûre pour l'abonnement Claude passait par le serveur MCP. La
  question est dans [questions-ouvertes.md](questions-ouvertes.md).

### Le CLAUDE.md

Écrit le même jour, à la racine du dépôt : comment on travaille, comment on
écrit, ce qui est décidé.

Le mainteneur l'a lu et validé : « le reste me va ». Sont donc retenus :

- la règle « pas de script », telle qu'elle y est écrite ;
- deux règles reprises d'Opn-Link : la façon d'écrire, et les captures
  montrées avant de garder un changement visible ;
- pour brancher l'IA : une clé API, un modèle local, ou l'abonnement ChatGPT
  avec Codex. Pour l'abonnement Claude, la question sera posée à Anthropic.
  Sans accord, il attend le serveur MCP.

### Le matériel d'OpenContact

Le mainteneur avait répondu « un design à lui ». Il corrige : « J'ai oublié,
on utilisera le même matériel UI/UX qu'OpenContact. »

- Le matériel d'interface et d'UX est celui d'OpenContact, « Utilitaire 98 » :
  ses jetons, ses polices, ses icônes, ses composants et les règles de son
  kit `design/`.
- Claude comprend que les écrans et la navigation restent à dessiner pour
  Opn-Display, avec ce matériel : OpenContact n'a ni lab, ni terminal, ni
  schéma. Le mainteneur peut corriger.

### L'ordre de construction, et la licence

Claude a proposé l'ordre et la licence. Le mainteneur a dit qu'il n'avait pas
compris la demande, Claude l'a reformulée plus simplement, puis il a répondu.

- L'ordre est décidé. D'abord les maquettes, puis l'app qui vérifie le PC,
  puis un petit lab de deux ou trois machines Linux, puis l'IA, puis les
  autres familles de machines (réseau, Windows Server, pare-feu et sécurité,
  stockage et WiFi, ESXi), puis macOS et la version publique, puis le
  pilotage. Le détail est dans [questions-ouvertes.md](questions-ouvertes.md).
- La licence n'est pas choisie : « Je décide plus tard ». Le projet reste
  gratuit et libre.

### Le brief de conception

Le mainteneur a donné son accord pour commencer par le brief (« Let's go »).
Claude lui a posé quatre questions sur les idées d'écrans de la première
fiche.

- Les huit idées : le schéma en direct, les paquets en animation, le tableau
  blanc, le replay, le compte rendu, la progression, les parcours, le mode
  examen. Il les a toutes cochées, en ajoutant « honnêtement je sais pas ».
  Elles ne sont donc pas décidées. Elles se dessinent en maquettes, et il
  tranchera en voyant les captures.
- Le menu : « Propose-moi le menu ». Les quatre espaces de la fiche ne sont
  pas retenus d'office.
- Le dessin : « Simple et classique », proche de GNS3 et de Packet Tracer,
  avec le panneau de l'IA sur le côté.
- Le brief est dans [brief-conception.md](brief-conception.md).

### La première maquette

Le mainteneur a donné son accord pour copier le matériel d'OpenContact et
commencer (« Go »).

- Le matériel est copié dans `maquettes/kit/`, depuis le dépôt d'OpenContact
  à la version `a1d317b`.
- Les icônes de machines manquaient. Elles viennent du même jeu,
  pixelarticons 2.4.2 : `server`, `move` pour le routeur,
  `arrows-horizontal` pour le switch, `wall` pour le pare-feu, `database`
  pour le stockage.
- Claude propose un menu à trois entrées, Labs, Cours et Pannes, avec
  Réglages à droite, et l'écran du lab : le schéma, les terminaux, l'IA sur
  le côté.
- Les captures sont dans `maquettes/captures/`.
- Le mainteneur a validé le menu et l'écran du lab : « C'est bien », et il a
  demandé la suite.
- Cinq écrans de plus : la liste des labs, un nouveau lab, une panne, la
  vérification, un cours. Le mainteneur les a validés : « Super ! Je
  confirme ».
- À droite de l'écran du lab, le panneau est toujours l'IA. Ce qu'elle y
  montre change : une discussion, une panne, une vérification, un cours.
- Pendant une panne, le schéma ne montre rien en rouge, et les tests restent
  cachés jusqu'à la vérification.
- Six écrans de plus sont proposés : la première ouverture, l'installation,
  et quatre réglages de l'IA (la brancher, la régler, sa mémoire, son
  journal). Ils attendent son avis.
- Dans « Régler l'IA », Claude propose que la liberté de l'IA sur le PC et sa
  mémoire restent à part. Les trois façons de régler (par niveau, un par un,
  elle s'adapte) portent sur le lab, la solution et l'initiative.
- Le mainteneur a répondu : « À toi de décider. C'est toi qui as la main. »
  Claude retient donc les six écrans et ce choix sur « Régler l'IA ». Pour
  la suite des maquettes, Claude décide, montre les captures et note ce
  qu'il retient. Le mainteneur peut revenir dessus. Le code, la licence et
  ce qui se publie restent à son accord.
- Trois écrans de plus, retenus par Claude : la liste des cours, la liste
  des pannes, et la fenêtre pour fournir l'image de Windows Server. Le
  catalogue des pannes dit ce qui ne marche plus, jamais la cause.
- Les huit idées de la première fiche sont dessinées, une par écran. Claude
  recommande d'en garder trois pour la première version : le schéma en
  direct, le compte rendu et le mode examen. Les cinq autres viendraient
  plus tard. Le mainteneur n'a pas encore tranché.

## 17 novembre 2026

### Les huit idées, dans la première version

Claude a montré les huit idées en maquettes, une par écran, et recommandait
d'en garder trois : le schéma en direct, le compte rendu et le mode examen.
Le mainteneur a choisi « Les huit ».

- Les huit idées sont dans la version de juillet 2027 : le schéma en direct,
  les paquets en animation, le tableau blanc, le replay, le compte rendu, la
  progression, les parcours, le mode examen.
- C'est plus de travail avant juillet. Claude propose une place pour chacune
  dans l'ordre de construction, avec ce dont elle dépend
  ([questions-ouvertes.md](questions-ouvertes.md)).

Claude les a ensuite reliées aux écrans déjà retenus, comme le mainteneur
lui a laissé la main sur les maquettes :

- la Progression devient une entrée du menu : Labs, Cours, Pannes,
  Progression, et Réglages à droite ;
- le schéma a deux outils en haut à droite : « Comparer au lab prévu » et
  « Voir les paquets ». Pendant une panne ou un examen, seul « Voir les
  paquets » reste, pour ne pas donner la réponse ;
- après la vérification, l'IA propose « Lire le compte rendu » ;
- la page Pannes propose un examen, à côté de la panne inventée ;
- la page Progression porte le parcours et la suite proposée. « Changer »
  ouvre le choix du parcours ;
- un écran de plus : le résultat de l'examen.
