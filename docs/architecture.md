# Architecture

Une proposition, sauf la pile de la partie 5, choisie le 17 novembre 2026,
et ce qui est codé (partie 7). Les premières mesures sont dans
[mesures.md](mesures.md). Rien n'est encore mesuré sur le PC du mainteneur.

Les pistes viennent des recherches du 13 novembre 2026 et des trois outils
que le mainteneur donne en référence : GNS3 et EVE-NG pour l'émulation,
Hermes Agent pour l'IA. Ce qui doit être mesuré avant de choisir est dans
[questions-ouvertes.md](questions-ouvertes.md).

## 1. Les trois parties

- L'interface : ce que l'utilisateur voit. Le lab, les terminaux, le panneau
  de l'IA.
- Le moteur : il fait tourner les machines et le réseau du lab.
- L'agent : l'IA, ses outils et ses réglages.

L'interface ne touche à rien directement. Elle demande au moteur et à
l'agent. GNS3 fait pareil : son interface commande un serveur par une API.

## 2. Le moteur

### Comment font GNS3 et EVE-NG

- Les deux font tourner des VM avec QEMU et KVM, et des conteneurs avec
  Docker. Tout ça se passe dans Linux.
- GNS3 : un serveur pilote QEMU, Docker et Dynamips. L'interface, lourde ou
  web, commande ce serveur par une API REST. Un petit programme, uBridge,
  relie les machines entre elles. Sous Windows et macOS, le serveur tourne
  dans une VM Linux, la « GNS3 VM ».
- EVE-NG : un serveur Ubuntu, installé sur une machine ou dans une VM, avec
  une interface web.

Sur un PC Windows ou un Mac, ces deux outils ont donc besoin d'une VM Linux,
et de la virtualisation imbriquée pour y lancer d'autres VM.

### Ce qu'on reprend

- Un environnement Linux propre à l'app, où tournent QEMU, KVM, les
  conteneurs et le réseau virtuel.
- De vraies machines, avec leur vrai système. Rien n'est imité.
- Un lab décrit dans un fichier texte, qu'on peut partager.
- Aucune image propriétaire dans l'app.

### Ce qu'on change

- Rien à installer à la main, aucun script à lancer. L'app installe et règle
  son environnement Linux toute seule, et vérifie elle-même ce que le PC sait
  faire.
- Tous les domaines, pas seulement le réseau.
- L'IA, qui a les mêmes commandes que l'interface.

### Sur chaque système

| Système | Environnement Linux | Ce qu'on sait |
|---|---|---|
| Windows 11 | Une distribution WSL2 propre à l'app, avec KVM imbriqué | WSL2 marche sur l'édition Famille et l'imbrication y est activée par défaut (doc officielle). À essayer sur le PC du mainteneur |
| Linux | Le système lui-même, avec KVM | Le cas le plus simple |
| macOS | Une VM Linux lancée par l'app (Lima) | Sur puce Apple, seules les machines ARM tournent à vitesse normale |

Par défaut, WSL2 ne prend que la moitié de la RAM du PC. L'app doit monter
cette limite dans `.wslconfig`, avec l'accord de l'utilisateur, et faire
charger les modules du noyau dont le lab a besoin.

### Les machines

- Des conteneurs pour ce qui est léger : services Linux, routeurs FRR,
  clients. Ils démarrent en quelques secondes.
- Des VM QEMU pour le reste : Windows Server, OPNsense, Kali, ESXi, et les
  images d'autres constructeurs que l'utilisateur fournit, quand la licence
  le permet.
- Windows Server s'installe sans intervention depuis l'ISO, avec un fichier
  de réponses et les pilotes virtio.
- L'app commande une VM par QMP et par l'agent invité de QEMU. L'agent invité
  répond même quand le réseau de la machine est cassé, ce qui compte pour
  poser une panne et la vérifier. SSH sert en second.
- La remise à zéro : une image de référence en lecture seule, et une couche
  jetable par machine. On jette la couche, la machine revient à neuf.
- La sauvegarde avant une panne inventée : un instantané du lab.

### Le réseau

Comme containerlab et GNS3, avec ce que le noyau Linux fournit : une paire
veth par câble, des bridges ou Open vSwitch pour les switchs, tc et netem
pour dégrader un lien. Par défaut, le réseau du lab n'a aucune route vers le
vrai réseau.

Les protocoles : le bonding de Linux ou Open vSwitch pour LACP, mstpd pour
RSTP et MSTP, FRR pour VRRP, OSPF et BGP. Ce qu'on sait de chacun est dans
[mesures.md](mesures.md), partie 10.

Les switchs, à l'étape 5 (23 novembre 2026) : un pont Linux qui filtre les
VLAN, dans un conteneur Debian, avec le STP du noyau (802.1D) pour les
liens en double. Ses ports s'appellent swp1, swp2… : 8, 24 ou 48 ports en
cuivre à 1 Gb/s, et 4 ports SFP+ à 10 Gb/s sur les modèles de 24 et 48. Un
port est un VLAN d'accès ou un trunk 802.1Q. Le routage entre VLAN se fait
sur un switch de niveau 3, une interface par VLAN, ou sur un routeur
Debian relié par un trunk, une sous-interface par VLAN. Le noyau de WSL2 a
tout cela ([mesures.md](mesures.md), partie 16).

RSTP et MSTP attendent : dans un conteneur, le noyau ne laisse mstpd
piloter un pont qu'à partir de Linux 7.1 (`BR_STP_MODE_USER`), et le noyau
de WSL2 est en 6.6 ou 6.18.

Sur chaque câble, comme GNS3, EVE-NG et CML : la capture (tcpdump dans le
réseau d'une des deux machines), la coupure (un bout éteint), la
dégradation (tc et netem : délai, gigue, perte, débit). La vue des baies
reprend les mêmes machines et les mêmes câbles
([infra-complete.md](infra-complete.md)).

Les interfaces web du lab s'ouvrent dans l'app. Sous Windows, le réseau du
lab vit dans WSL2 et Windows ne le voit pas : l'app fait le pont. Piste à
mesurer à l'étape 6 : un appareil `proxy` d'Incus, qui ouvre un port de
Debian vers la machine, et le renvoi de WSL2 vers Windows.

### ESXi, le stockage, le WiFi

- ESXi : une VM QEMU avec la virtualisation imbriquée. Sous Windows, ça fait
  trois niveaux : Windows, WSL2, puis ESXi et ses propres VM. Personne ne dit
  si ça marche, c'est à mesurer. L'autre voie est de piloter VMware
  Workstation, mais Workstation ne sait pas imbriquer quand Hyper-V est
  actif, et WSL2 en a besoin.
- Le stockage : la cible iSCSI du noyau Linux pour le SAN, Samba et NFS en
  conteneurs pour le NAS. TrueNAS demande 8 Go, il reste possible avec 32 Go.
- Le WiFi : le module `mac80211_hwsim` du noyau simule des cartes WiFi,
  hostapd fait le point d'accès, FreeRADIUS l'authentification. Ce module est
  dans le noyau 6.18 de WSL2, pas dans le 6.6. L'app doit vérifier la version.

## 3. L'agent

### Comment fait Hermes Agent

- C'est une app complète en Python, sous licence MIT. Elle accepte n'importe
  quel modèle.
- L'agent agit par un terminal, qui peut être la machine locale, un
  conteneur Docker ou une machine distante en SSH.
- Sa mémoire tient dans deux petits fichiers de taille fixe, un pour ses
  notes et un pour le profil de l'utilisateur. Ils sont relus au début de
  chaque session. Il peut aussi chercher dans les anciennes sessions.
- Il écrit des procédures après une tâche réussie, les corrige à l'usage, et
  range celles qui ne servent plus.
- Pour l'accord, trois modes : `manual` (il demande), `smart` (un petit
  modèle juge le risque et ne demande que dans le doute), `off`. Une liste de
  commandes reste bloquée même en mode `off`.

### Ce qu'on reprend

- N'importe quel modèle.
- Les modes d'accord, qui deviennent nos réglages : libre, avec accord,
  limité.
- La mémoire de taille fixe, que l'utilisateur peut lire et corriger.
- Plusieurs terminaux pour un même agent : le PC, et chaque machine du lab.
- Les procédures apprises, plus tard.

### Ce qu'on ne reprend pas

Hermes lui-même. C'est une app Python entière, pas une brique à embarquer.
Les recherches conseillent une boucle d'agent courte, écrite en Rust dans
l'app.

### Brancher l'IA

| Façon | Comment | État |
|---|---|---|
| Clé API | L'app appelle le fournisseur. La clé reste dans le trousseau du système | Connu |
| Modèle local | L'app parle à Ollama sur le PC | Connu. Le modèle prend de la RAM au lab |
| Abonnement ChatGPT | L'app lance Codex, l'outil officiel installé sur le PC | Fait dans `natif/` d'OpenContact |
| Abonnement Claude | L'utilisateur branche son Claude Code ou son Claude Desktop sur le serveur MCP de l'app | La seule voie sûre d'après les règles d'Anthropic. L'IA parle dans la fenêtre de Claude. Plus tard, peut-être |
| Assistant extérieur | L'app expose un serveur MCP, et l'utilisateur y branche son propre assistant | Fait dans `natif/` d'OpenContact. Plus tard, peut-être |

Le mainteneur veut une vraie application : l'IA parle dans le panneau de
l'app. Le serveur MCP n'est pas dans la première version (16 novembre 2026).

### Les outils de l'IA

Dans le lab : créer ou changer le lab, lancer une commande sur une machine,
lire l'état, capturer du trafic, poser une panne, lancer un test.

Sur le PC : installer un outil, télécharger une image, lancer et commander un
logiciel, lancer une commande, lire et écrire un fichier.

Chaque appel passe par les réglages de l'utilisateur avant d'être exécuté.
C'est l'app qui applique le réglage. Le demander à l'IA dans sa consigne ne
suffit pas, elle peut l'oublier ou se le faire retirer par un texte piégé.

L'app tient un journal de ce que l'IA a fait, sur le PC et dans le lab.

### Les pannes

- Une panne du catalogue est un fichier : comment la poser, ce que
  l'utilisateur doit observer, les tests, les indices, l'explication, le
  retour en arrière.
- Une panne inventée s'écrit dans le même format, au moment où l'IA la pose.
  L'app sauvegarde le lab juste avant.
- Pendant la pose, l'app n'affiche pas ce que fait l'IA. GNS3 3.1 affiche les
  étapes, ce qui donne la réponse.
- Dans GNS3, une panne est une description. Sur 858, 25 disent comment la
  poser, et aucune ne dit comment vérifier la réparation.

### La vérification

1. À la pose de la panne, l'IA écrit ses tests : ce qui doit échouer
   maintenant, et ce qui devra passer une fois la panne réparée.
2. Elle les lance. S'ils n'échouent pas, la panne n'a pas pris.
3. Quand l'utilisateur dit qu'il a réparé, elle les relance. Elle peut en
   ajouter.
4. Elle conclut, et l'app montre chaque test avec son résultat.

### Les garde-fous

- Sur le PC, le réglage par défaut est « avec accord ».
- Supprimer des fichiers ou toucher aux protections de Windows demande
  toujours un oui, même en mode libre.
- Kali et l'IA n'attaquent que les machines du lab.
- Ce que l'IA lit dans le lab ou dans un fichier (un journal, une page web,
  un PDF) est une donnée, jamais un ordre.
- Les clés API ne sont jamais montrées à l'IA.

## 4. Le pilotage

Il vient après l'émulation. Chaque logiciel a déjà de quoi être commandé :

- VMware Workstation : `vmrun` et une API REST ;
- Cisco Modeling Labs : une API REST. CML-Free est gratuit, avec 5 machines
  Cisco à la fois ;
- Packet Tracer : un canal prévu pour les extensions. L'IA peut y charger un
  lab, mais voit mal ce que l'utilisateur y fait, le logiciel est fermé ;
- le cloud : les outils en ligne de commande des hébergeurs, avec les accès
  de l'utilisateur.

## 5. La pile

Choisie le 17 novembre 2026 :

| Partie | Choix | Pourquoi |
|---|---|---|
| App de bureau, moteur et IA | Tauri 2, en Rust | Déjà utilisé dans `natif/` d'OpenContact et prévu pour Opn-Link. Installeur léger. Un seul langage pour l'app, le moteur et l'IA |
| Écrans | React et TypeScript, dans la page web de Tauri | Les composants du kit d'OpenContact sont en React. React Flow dessine le schéma du lab. Le mainteneur a laissé Claude choisir « le meilleur, avec le plus de potentiel » |
| Linux de l'app | Debian, dans WSL2 sous Windows et dans une VM Lima sous macOS. Sous Linux, le système lui-même | Stable, sans compte. FRR, QEMU, Open vSwitch et mstpd sont dans ses paquets |
| Matériel d'OpenContact | Copié dans le code, avec sa version notée | Comme pour les maquettes. Aucun script |
| Machines du lab | Incus, dans le Debian de l'app. Docker plus tard, pour les images qui n'existent qu'en Docker | Une machine Incus est un vrai Linux avec systemd. Incus fait aussi les instantanés et les VM ([mesures.md](mesures.md), partie 11) |
| Terminaux | xterm.js 6 dans l'écran, un pseudo-terminal côté Rust (portable-pty, ConPTY sous Windows) | Le plus répandu. Choisi à l'étape 3 ([mesures.md](mesures.md), partie 12) |
| Agent de l'IA | Une boucle écrite à la main en Rust, avec ureq : le format d'OpenAI (que parlent aussi Ollama, Mistral et OpenRouter), celui d'Anthropic, et Codex pour l'abonnement ChatGPT | 44 paquets et 1,2 Mo, contre 186 et 8,2 Mo pour genai ([mesures.md](mesures.md), partie 13). Choisi par le mainteneur le 17 novembre 2026 |

Encore à choisir, chacun à son étape, après mesure :

| Partie | Piste | Pourquoi |
|---|---|---|
| VM | QEMU, commandé par QMP et l'agent invité | Ce que font GNS3 et EVE-NG |
| Écran des VM | noVNC, branché sur QEMU | Marche pour Windows comme pour Linux |
| Données locales | SQLite | Un seul fichier, sur le PC |
| Distribution | GitHub Releases | Gratuit. La signature Windows peut l'être aussi, Apple coûte 99 $ par an |

## 6. Avant de choisir

Le reste de la pile se choisit à chaque étape, après mesure. Ce qui est
mesuré est dans [mesures.md](mesures.md). Ce qui reste se mesure sur le PC
du mainteneur, par l'app elle-même.

## 7. Ce qui est codé

L'étape 2 de l'ordre de construction, commencée le 17 novembre 2026, dans
`app/` :

- le moteur (`app/moteur/`) regarde le PC : Windows et son édition, la
  mémoire, la virtualisation, WSL2 et son noyau. Une fois Debian installé,
  il regarde dedans : KVM, le WiFi simulé, la mémoire de WSL2. Sous Linux,
  il regarde le système lui-même ;
- sous Windows, il installe Debian dans WSL2. L'image vient de la liste des
  distributions de WSL tenue par Microsoft, elle est fabriquée par le projet
  Debian, et son empreinte SHA-256 est vérifiée avant l'import ;
- la mémoire de WSL2 ne change que si l'utilisateur coche la case. L'app
  garde une copie de `.wslconfig` avant d'y toucher ;
- les écrans de la première ouverture reprennent les maquettes : la
  vérification, l'installation, le PC qui bloque, et le rapport final, que
  l'utilisateur peut copier ;
- l'installation peut s'arrêter en cours de route, et Debian peut être
  retiré de WSL2 depuis le rapport final, après une question.

Testé dans l'environnement cloud : le moteur sous Linux (21 tests), l'app
lancée pour de vrai sous Linux, et les écrans propres à Windows avec des
réponses simulées. La partie Windows se compile ici, mais ne tourne que sur
un PC Windows.

L'étape 3, commencée le 17 novembre 2026 :

- l'installation pose aussi Incus. Sous Windows : systemd allumé dans
  Debian (`/etc/wsl.conf`), puis `apt-get install incus`. Sous Linux,
  l'Incus du système sert. Puis la place des machines (un pool `dir`) et
  l'image Debian 13 d'Incus, rangée sous le nom `opn-display-debian-13` ;
- le premier lab est écrit dans le moteur : pc1 et pc2, chacun dans son
  réseau, et un routeur entre les deux, les adresses posées. Il a son
  projet Incus, `opn-display-premier-lab` ;
- un câble est une paire veth, posée par l'app, dont chaque bout entre dans
  une machine. Les réglages réseau sont des fichiers de systemd-networkd,
  écrits dans la machine avant son premier démarrage ;
- l'écran du lab reprend la maquette : le schéma, les terminaux en onglets,
  la barre d'état avec la RAM du lab. Le panneau de l'IA arrivera avec
  elle, à l'étape 4 ;
- une veille écoute Incus (`incus monitor`) : une machine éteinte depuis son
  terminal passe en trame sur le schéma. Sous Windows, elle garde Debian
  éveillé tant que l'app est ouverte ;
- fermer l'app arrête les machines du lab.

Testé dans l'environnement cloud avec un vrai Incus : le moteur (37 tests),
l'app lancée pour de vrai, le lab créé, les pings à travers le routeur, une
machine éteinte puis rallumée, la remise à zéro. Les écrans propres à
Windows sont vus avec des réponses simulées.

L'étape 4, commencée le 17 novembre 2026 :

- l'IA se branche de trois façons : une clé API (OpenAI, Anthropic,
  Mistral, OpenRouter, ou une autre adresse au format d'OpenAI), Ollama sur
  le PC, ou l'abonnement ChatGPT par Codex. La clé reste dans le trousseau
  du système ;
- la boucle de l'agent est écrite à la main. Avec Codex, c'est lui qui fait
  la boucle : son shell, ses fichiers et le web sont coupés, et il n'a que
  les outils de l'app, par un relais MCP. L'app lancée avec `OPN_OUTILS`
  fait ce relais, vers l'app ouverte, par 127.0.0.1 et un jeton ;
- ses outils dans le lab : lire l'état du lab, lancer une commande dans une
  machine, démarrer ou arrêter une machine. Une commande est coupée au bout
  de 30 secondes, et sa sortie est bornée ;
- « Avec accord », l'app montre chaque action avant de la faire, dans une
  carte du panneau de l'IA. C'est l'app qui applique ce réglage ;
- le panneau de l'IA est à droite du lab : le niveau, la discussion, ce
  qu'elle a fait, et les cartes d'accord. Réglages a sa liste à gauche :
  Brancher l'IA, Régler l'IA, Ton PC.

Testé ici avec un vrai modèle local (Ollama et Qwen3 1.7B) : brancher, lui
parler dans le lab, son accord demandé, la commande lancée dans pc1 et sa
conclusion. Codex 0.169.0 accepte les réglages de l'app et lance le relais
MCP ; le reste se vérifie avec un compte ChatGPT.

La suite de l'étape 4, les pannes :

- le premier lab a un catalogue de cinq pannes réseau
  (`app/moteur/pannes/premier-lab.json`) : ce que l'utilisateur constate,
  les commandes qui la posent, trois tests, trois indices et l'explication ;
- avant chaque panne, l'app prend un instantané Incus du lab
  (`avant-panne`). « Abandonner » y revient, une panne réparée l'efface. Une
  panne dont tous les tests passent juste après la pose n'a rien cassé :
  l'app revient à l'instantané ;
- l'écran Pannes montre le catalogue, et propose une panne inventée par
  l'IA, au niveau choisi. L'IA la pose avec l'outil `poser_une_panne`, dans
  une conversation à part, sans rien montrer de ce qu'elle fait. Après trois
  poses qui n'ont rien cassé, elle s'arrête ;
- la panne posée est épinglée en haut du panneau de l'IA, avec « Un
  indice », « C'est réparé » et « Abandonner ». Les noms des tests restent
  cachés jusqu'à la réparation : ils donneraient la réponse ;
- « C'est réparé » relance les tests écrits à la pose, et l'app montre
  chacun avec son résultat. L'IA n'en ajoute pas encore au moment de
  vérifier ;
- l'IA connaît la panne posée et sa cause. En Coach, elle ne la donne pas.
  En Silencieuse, elle n'en parle pas, et « Un indice » n'apparaît pas ;
- « Arrêter » coupe l'IA tout de suite, même au milieu d'une réponse du
  modèle ;
- la panne posée est gardée dans `panne.json`, à côté de `ia.json`, jusqu'à
  sa fin : fermer l'app ne la perd pas. « Remettre à zéro » l'emporte ;
- avant une pose ou une vérification, l'app démarre le lab s'il est arrêté,
  et attend que ses machines aient fini de démarrer.

Testé ici avec un vrai Incus : les cinq pannes posées, leurs tests en
échec, puis le retour à l'instantané. Dans l'app lancée pour de vrai : une
panne posée, un indice, la vérification avant et après la réparation,
« Abandonner », et une panne inventée avec un faux modèle
([mesures.md](mesures.md), partie 14).

La fin de l'étape 4, l'IA construit le lab :

- « Nouveau lab » reprend la maquette validée : on décrit le lab, l'IA le
  propose avec l'outil `proposer_un_lab`, on voit son schéma, ses machines
  et sa RAM, on retire une machine ou on redemande, puis « Lancer le lab ».
  L'IA n'y a que cet outil, dans une conversation à part qui se souvient du
  lab proposé ;
- l'IA décrit les machines (poste, serveur, routeur, switch), les câbles
  avec l'adresse de chaque bout, les passerelles et les routes. L'app nomme
  les interfaces dans l'ordre des câbles, et vérifie tout : des noms
  valables, une adresse par machine, une plage par réseau, une passerelle
  qui est un routeur de son réseau. Un refus repart vers l'IA, qui
  corrige. Elle s'arrête après trois refus ;
- l'app place les machines sur le schéma : les routeurs et les switchs sur
  une rangée, les autres dessous. Les machines d'un switch descendent d'une
  même ligne, qui porte leur réseau une seule fois ;
- un switch est un conteneur Debian dont les ports entrent dans un pont
  Linux, `br0`. Pas de VLAN à cette étape ;
- chaque lab construit a son fichier dans le dossier des données de l'app
  (`labs/lab-2.json`…), son numéro, qui entre dans le nom de ses câbles, et
  son projet Incus. Mes labs le montre à côté du Premier lab ;
- le lab ouvert est celui où travaillent l'IA du panneau et les pannes.
  Changer de lab repart d'une discussion vide. Une panne garde son lab ;
- un lab construit se supprime depuis Mes labs, quand il est arrêté, après
  une question. Le Premier lab ne se supprime pas.

Testé ici avec un vrai Incus et un faux modèle qui répond toujours pareil.
Sa première proposition avait une mauvaise passerelle : l'app l'a refusée,
et il a corrigé. Le lab proposé (un routeur, deux switchs, quatre
machines), lancé : pc1 joint pc2 par le switch, et srv1 par le routeur.
Une panne inventée s'y pose et s'y abandonne. Supprimé, son projet Incus,
ses dix bouts de câble et son fichier partent.

Le 18 novembre 2026, « Ajouter une machine » et les réglages de l'IA :

- « Ajouter une machine » demande un poste, un serveur, un routeur ou un
  switch à l'IA, dans la conversation du lab proposé. L'app lui envoie le
  lab tel qu'il est à l'écran, écrit comme elle l'écrit
  (`proposition::en_proposition`) : une machine retirée ne revient pas ;
- Mes labs dit quand chaque lab a été ouvert. L'app le note dans
  `ouverts.json`, à chaque ouverture ;
- Régler l'IA a ses trois façons : par niveau, un par un, ou une IA qui
  s'adapte (`moteur/src/ia/reglage.rs`). Toutes portent sur trois
  comportements : sa liberté dans le lab, ce qu'elle fait quand on demande
  la solution, et quand elle parle d'elle-même. C'est la solution qui écrit
  sa façon d'aider dans sa consigne, et ce qu'elle sait de la panne posée ;
- l'IA qui s'adapte change de niveau à la fin d'une panne, réparée ou
  abandonnée. L'app garde son niveau et la raison dans `ia.json` ;
- quand l'utilisateur bloque, c'est le panneau qui compte les 10 minutes.
  Il demande alors un tour à l'IA (`aider_ia`) ; pendant ce tour, l'app
  refuse chacun de ses outils. Ils restent décrits au modèle : Anthropic
  refuse une conversation qui a appelé des outils sans qu'ils le soient ;
- sa mémoire est dans `memoire.json` (`moteur/src/ia/memoire.rs`) : deux
  listes, 30 lignes en tout. Elle y écrit avec l'outil `retenir`, et la
  relit au début de chaque conversation ;
- son journal est dans `journal.json` (`moteur/src/ia/journal.rs`). Les
  outils l'écrivent à chaque action dans le lab, avec la réponse à
  l'accord ; l'app y ajoute les labs proposés et les pannes inventées.
  Pour Codex, le relais MCP passe ces nouveaux modes par la variable
  `OPN_OUTILS`, comme les autres.

Essayé ici dans l'app lancée pour de vrai, avec un vrai Incus et un faux
modèle : la mémoire écrite puis relue dans la consigne, le journal, « Ajouter
une machine » après un retrait, l'IA qui s'adapte après une panne réparée,
et l'aide proposée au bout de 10 minutes.
