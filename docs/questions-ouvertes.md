# Questions ouvertes

Le 16 novembre 2026, le mainteneur a répondu à neuf séries de questions. Ses
décisions sont dans [carnet.md](carnet.md) et reportées dans
[projet.md](projet.md). Ce qui pouvait se mesurer à distance l'a été le même
jour ([mesures.md](mesures.md)). Ce fichier garde ce qui reste, avec une
recommandation pour chaque point.

## Ce que Claude doit proposer

### L'ordre de construction

Décidé le 16 novembre 2026 : le mainteneur a répondu « Oui, cet ordre » à la
proposition ci-dessous. Elle reste ici pour mémoire. À chaque étape, l'app
marche et peut se montrer.

1. Les maquettes : un brief de conception, puis les écrans en captures, avec
   le matériel d'OpenContact.
2. L'app s'ouvre et vérifie le PC : WSL2, le noyau, KVM, le WiFi simulé, la
   RAM. Elle installe son environnement Linux. Cette étape donne les mesures
   qui manquent.
3. Un premier lab : deux ou trois machines Linux en conteneurs, reliées, avec
   leurs terminaux dans l'app.
4. L'IA : elle se branche, construit ce lab, pose une panne et vérifie la
   réparation. Avec ses réglages et ses garde-fous.
5. Le réseau : routeurs, switchs, VLAN, routage, NAT.
6. Les VM : Windows Server et Active Directory. Dès qu'une VM tourne, on
   essaie ESXi, pour savoir tôt s'il marche sous Windows.
7. Les pare-feu et la sécurité : OPNsense, VPN, Wazuh, Kali.
8. Le stockage et le WiFi simulé.
9. ESXi, si l'essai de l'étape 6 a marché.
10. macOS, l'installeur et la version publique.
11. Le pilotage.

Chaque famille de machines arrive avec ses pannes et ses cours. La version
Linux de l'app suit dès l'étape 2 : le moteur y est le plus simple.

### La licence

Le mainteneur a répondu « Je décide plus tard » (16 novembre 2026). La
question reste ouverte. Il veut un projet gratuit et libre.

Recommandation, proposée le 16 novembre 2026 : MIT, ou Apache-2.0 qui est
presque pareille avec une clause de plus sur les brevets. C'est la licence
d'Opn-Link, de Rust et de Tauri.

- L'app lance QEMU et les autres outils sous GPL comme des programmes
  séparés, ce qui est permis.
- Hermes Agent est sous MIT. On peut lui reprendre du code.
- On ne peut pas copier les 858 pannes de GNS3, qui sont sous GPL-3.0. Elles
  n'ont pas de vérification, et notre catalogue a son propre format. La perte
  est faible.
- Le matériel « Utilitaire 98 » vient d'OpenContact, qui est sous licence
  propriétaire. Le mainteneur en est l'auteur, il peut le publier aussi sous
  la licence d'Opn-Display. C'est à lui de le dire.

### Le design

Le matériel est celui d'OpenContact, « Utilitaire 98 » (décidé le
16 novembre 2026). Restent les écrans et la navigation d'Opn-Display, à
dessiner avec ce matériel.

Recommandation : écrire un brief de conception, comme pour Opn-Link, puis
montrer des maquettes en captures avant de coder un écran.

Reste aussi à dire comment le matériel arrive dans ce dépôt. Opn-Link le
prend par un script, et ici il n'y a pas de script.

Recommandation : le décider au moment des maquettes. Une piste : un
sous-module git qui pointe sur le dépôt d'OpenContact.

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

La seule voie permise en toutes lettres est le serveur MCP : l'utilisateur y
branche son Claude, et l'IA parle dans la fenêtre de Claude. Le mainteneur a
répondu qu'il veut une vraie application, et que le serveur MCP viendra
peut-être plus tard. L'abonnement Claude n'a donc pas de voie sûre dans la
première version.

Décidé le 16 novembre : l'app branche l'IA par une clé API, par un modèle
local, ou par l'abonnement ChatGPT avec Codex. Pour l'abonnement Claude dans
le panneau de l'app, la question est posée à Anthropic avant de construire
quoi que ce soit. Sans accord, il attend le serveur MCP.

Recommandation : Claude prépare le message, le mainteneur l'envoie.

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

Le mainteneur ne veut pas de script à lancer sur son PC. L'app fera ces
mesures elle-même.

Recommandation : le premier écran construit est celui où l'app vérifie le PC
(version du noyau, KVM, WiFi simulé) et affiche ce qu'elle trouve. Il sert à
tous les utilisateurs, et il donne les mesures qui manquent.

## Le nom et le dépôt

- Opn-Display est un nom de travail. Le nom définitif est à choisir avant la
  version publique.
- Le dépôt est privé. Le mainteneur décidera plus tard quand l'ouvrir.
