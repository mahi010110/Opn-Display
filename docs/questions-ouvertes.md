# Questions ouvertes

Le 16 novembre 2026, le mainteneur a répondu à neuf séries de questions. Ses
décisions sont dans [carnet.md](carnet.md) et reportées dans
[projet.md](projet.md). Ce fichier garde ce qui reste, avec une
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

La coquille native d'OpenContact utilise l'abonnement ChatGPT en lançant
Codex, l'outil officiel. Pour Claude, la voie serait de lancer Claude Code.
Les recherches du 13 novembre disent qu'Anthropic interdit aux apps tierces
de se connecter à l'abonnement, et conseillent de passer par un serveur MCP.

Recommandation : lire les conditions d'Anthropic, et garder les deux voies
(outil officiel et serveur MCP) tant que ce n'est pas clair.

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

### Les chiffres qui se contredisent dans les recherches

- Les pannes de GNS3 : 50 selon le rapport, 840 dans 51 fichiers selon une
  note.
- Le module de WiFi simulé dans le noyau de WSL2 : absent selon une note,
  présent selon une autre.
- WSL2 sur Windows 11 Famille : une réponse de Microsoft dit que la
  fonctionnalité nécessaire n'y est pas, l'usage courant dit le contraire.

## À mesurer

Sur le PC du mainteneur (Windows 11 Famille, 32 Go) :

- WSL2 s'installe-t-il, et KVM y est-il disponible ?
- la RAM que prend l'environnement Linux à vide ;
- le temps d'installation d'un Windows Server dans KVM sous WSL2, et sa
  vitesse à l'usage ;
- ESXi 8.0U3e démarre-t-il dans KVM sous WSL2, et peut-il lancer une VM ?
- le module de WiFi simulé se charge-t-il ?
- combien de machines tiennent ensemble dans 32 Go : AD, client, OPNsense,
  Wazuh, Kali ;
- la taille de l'installeur.

À distance, depuis l'environnement cloud :

- ce qu'Anthropic autorise pour l'abonnement ;
- les versions et les licences à jour des outils de la pile ;
- le format des pannes de GNS3 ;
- ce qui peut s'essayer sous Linux sans le PC du mainteneur : conteneurs,
  réseau virtuel, QEMU.

Les résultats iront dans `mesures.md`.

## Le nom et le dépôt

- Opn-Display est un nom de travail. Le nom définitif est à choisir avant la
  version publique.
- Le dépôt est privé. Le mainteneur décidera plus tard quand l'ouvrir.
