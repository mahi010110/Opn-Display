# Questions ouvertes

Le 16 novembre 2026, le mainteneur a répondu à neuf séries de questions. Ses
décisions sont dans [carnet.md](carnet.md) et reportées dans
[projet.md](projet.md). Ce qui pouvait se mesurer à distance l'a été le même
jour ([mesures.md](mesures.md)). Ce fichier garde ce qui reste, avec une
recommandation pour chaque point.

## Ce que Claude doit proposer

### L'ordre de construction

Le mainteneur veut tout pour juillet 2027, et a demandé une proposition
d'ordre après les mesures. Il a déjà fixé un point : l'émulation avant le
pilotage.

Recommandation, à confirmer après les mesures : commencer par un petit lab
complet sur le PC du mainteneur. Deux ou trois machines Linux, l'IA pose une
panne, l'utilisateur répare, l'IA vérifie. Ensuite on ajoute les machines une
famille à la fois : réseau, Windows Server, pare-feu et sécurité, stockage,
WiFi, ESXi. À chaque étape, l'app marche et peut se montrer.

### La licence

Recommandation provisoire : MIT ou Apache-2.0, comme Opn-Link. L'app lance
QEMU et les autres outils sous GPL comme des programmes séparés, ce qui est
permis. Si on veut un jour intégrer du code sous GPL dans l'app, par exemple
les pannes de GNS3, il faudra passer en GPL-3.0. À confirmer quand la pile
est choisie.

### Le design

Le mainteneur veut un design propre à Opn-Display. Reste à dire qui le
conçoit, et à partir de quoi.

Recommandation : écrire un brief de conception, comme pour Opn-Link, puis
montrer plusieurs propositions en captures avant de choisir.

## Les souhaits qui butent sur une limite

### Tout émuler sur macOS

Sur un Mac à puce Apple, Windows Server et ESXi ne tournent pas à une vitesse
utilisable. Ce sont des systèmes pour processeurs Intel et AMD.

Recommandation : sur Mac, l'app sort avec les labs qui tournent sur ARM
(Linux, réseau, une partie de la sécurité), et dit à l'écran pourquoi les
autres manquent.

### ESXi sous Windows

Sous Windows, ESXi tournerait dans KVM, lui-même dans WSL2, et devrait encore
lancer ses propres VM. Aucune source ne dit que ça marche. VMware Workstation
ne peut pas servir de secours tant que WSL2 est actif.

Recommandation : le mesurer en premier sur le PC du mainteneur. Si ça ne
marche pas, ESXi sort d'abord sous Linux.

### Une version publique sur trois systèmes

Sans signature, Windows affiche un avertissement à l'installation et macOS
refuse d'ouvrir l'app. La signature Windows peut être gratuite pour un projet
libre, ce qui demande un dépôt public et une licence choisie. Apple demande
99 $ par an.

Recommandation : demander la signature Windows gratuite dès que le dépôt est
public, et expliquer sur la page de téléchargement comment ouvrir l'app sur
Mac.

### Le pilotage

Le mainteneur a dit : l'émulation d'abord, le pilotage ensuite. Il n'a pas
dit quels logiciels, ni si c'est avant ou après juillet 2027.

Recommandation : décider quand l'émulation tourne.

## Ce qui vient de la première fiche et n'a pas été rediscuté

Le mainteneur a précisé que la fiche du 15 novembre ne vaut pas décision. Ces
points y figurent. Ils n'ont été ni confirmés ni refusés :

- quatre espaces dans l'app : Apprendre, Lab, Pannes et défis, Ma
  progression ;
- quatre modes pour un lab : Cours, Construire, Panne, Examen ;
- le schéma du lab en direct, qui montre en rouge ce qui diffère du lab
  prévu ;
- les paquets du lab rejoués en animation ;
- le tableau blanc où l'IA dessine ;
- le replay d'une session et le compte rendu à la fin ;
- la progression : compétences validées et révisions espacées ;
- les parcours : débutant, certification, titre ou diplôme, curieux ;
- le lien futur avec OpenContact et Opn-Link.

Recommandation : les passer en revue avec le brief de conception, écran par
écran.

## À vérifier

### L'abonnement Claude

Le mainteneur veut l'abonnement dans l'app, comme OpenContact le fait avec
Codex. Les règles d'Anthropic ont été lues le 16 novembre
([mesures.md](mesures.md), partie 3). Une app tierce ne peut ni proposer la
connexion claude.ai, ni faire passer des requêtes par l'abonnement. Lancer
`claude -p` depuis l'app n'est pas interdit en toutes lettres, mais Anthropic
annonce que ce mode n'utilisera plus l'abonnement par défaut.

Recommandation : pour Claude, l'abonnement passe par le serveur MCP de l'app,
et l'IA parle alors dans la fenêtre de Claude. Dans le panneau de l'app : une
clé API, un modèle local, ou l'abonnement ChatGPT par Codex. Si le mainteneur
veut l'abonnement Claude dans le panneau, poser la question à Anthropic avant
de le construire.

### Le réglage de la RAM de WSL2

Par défaut, WSL2 ne prend que la moitié de la RAM du PC, soit 16 Go sur 32.
La limite se change dans le fichier `.wslconfig`, qui règle tout WSL2 sur le
PC et pas seulement Opn-Display.

Recommandation : l'app propose la nouvelle limite à l'installation, montre la
ligne qu'elle va écrire, et ne touche au fichier qu'après un oui.

### Les images que l'IA télécharge seule

Recommandation : l'IA télécharge seule ce qui est libre (Linux, OPNsense,
Kali). Pour Windows Server, elle ouvre la page d'évaluation de Microsoft et
l'utilisateur accepte les conditions. Pour ESXi, l'utilisateur télécharge
avec son compte Broadcom et donne le fichier à l'app.

### Les outils d'attaque dans une app publique

L'article 323-3-1 du Code pénal encadre la mise à disposition d'outils
d'attaque. La formation est un motif admis, lu strictement.

Recommandation : l'app ne contient pas Kali, elle le télécharge depuis sa
source comme les autres images, et l'attaque reste enfermée dans le lab.

## À mesurer sur le PC du mainteneur

Rien n'a encore été mesuré sur son PC (Windows 11 Famille, 32 Go). La liste
est dans [mesures.md](mesures.md), partie 8. Les deux mesures qui pèsent le
plus sur la suite : KVM dans WSL2, et ESXi dans KVM sous WSL2.

Recommandation : commencer par les trois lectures qui prennent deux minutes
(version du noyau, KVM, WiFi simulé), avec un script qui ne change rien au
PC.

## Le nom et le dépôt

- Opn-Display est un nom de travail. Le nom définitif est à choisir avant la
  version publique.
- Le dépôt est privé. Le mainteneur décidera plus tard quand l'ouvrir.
