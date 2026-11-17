# Les mesures

Les parties 1 à 9 ont été faites le 16 novembre 2026, avant de choisir la
pile, l'ordre de construction et la licence. Les parties 10 et 11 datent du
17.

Tout a été fait depuis l'environnement cloud : une machine Linux de 4
processeurs et 15 Go, sans KVM. Rien n'a été mesuré sur le PC du mainteneur.
Ce qui reste à y mesurer est en partie 8, et c'est l'app qui le fera.

Le dépôt ne garde aucun script de mesure, le mainteneur n'en veut pas. La
partie 6 dit comment chaque essai a été fait.

Une mesure vaut pour le jour où elle a été faite.

## 1. Le noyau de WSL2

Lu dans le dépôt de Microsoft (`microsoft/WSL2-Linux-Kernel`, fichier
`arch/x86/configs/config-wsl`), pour les deux branches du noyau.

| Ce qu'il faut | Noyau 6.6 | Noyau 6.18 |
|---|---|---|
| KVM, pour lancer des VM dans WSL2 | oui | oui |
| Open vSwitch, bridges, VLAN, veth | oui | oui |
| netem, pour dégrader un lien | oui | oui |
| VXLAN, WireGuard | oui | oui |
| Cible iSCSI, pour le SAN | oui | oui |
| Serveur NFS | oui | oui |
| WiFi simulé (`mac80211_hwsim`) | non | oui |
| MPLS | non | non |

- Les deux notes de recherche qui se contredisaient sur le WiFi simulé
  avaient raison toutes les deux. Il manque dans le noyau 6.6 et il est dans
  le 6.18.
- La presse spécialisée date le passage de WSL2 au noyau 6.18 d'avril 2026.
  Le noyau installé sur le PC du mainteneur reste à lire (partie 8).
- La plupart de ces fonctions sont des modules. L'app doit les faire charger
  au démarrage de WSL2, par le réglage `loadKernelModules`.

## 2. WSL2 sur Windows 11 Famille

Lu dans la documentation officielle de WSL (`MicrosoftDocs/WSL`).

- WSL2 marche sur Windows 11 Famille. La FAQ le dit : « WSL 2 is available on
  all Desktop SKUs where WSL is available, including Windows 10 Home and
  Windows 11 Home ». La note de recherche qui disait le contraire citait une
  réponse d'utilisateur.
- La virtualisation imbriquée est activée par défaut, et seulement sous
  Windows 11.
- Par défaut, WSL2 ne prend que la moitié de la RAM du PC. Sur 32 Go, le lab
  serait limité à 16 Go. L'app doit monter cette limite dans le fichier
  `.wslconfig`.
- Ce fichier règle tout WSL2 sur le PC, pas seulement Opn-Display. L'app doit
  demander avant d'y toucher.

## 3. L'abonnement Claude

Lu dans la documentation de Claude Code, pages
[Legal and compliance](https://code.claude.com/docs/en/legal-and-compliance),
[Agent SDK overview](https://code.claude.com/docs/en/agent-sdk/overview) et
[Run Claude Code programmatically](https://code.claude.com/docs/en/headless).

Ce qui est interdit à une app tierce :

- proposer la connexion claude.ai dans l'app ;
- faire passer des requêtes par l'abonnement d'un utilisateur (« route
  requests through Free, Pro, Max, or Ultra plan credentials on behalf of
  their users ») ;
- garder ou transmettre ses identifiants ou ses jetons.

Ce qui est permis :

- l'utilisateur se connecte lui-même à Claude Code, non modifié, avec son
  abonnement ;
- il branche son Claude Code ou son Claude Desktop sur un serveur MCP que
  l'app expose. L'IA parle alors dans la fenêtre de Claude, pas dans le
  panneau de l'app.

Ce qui n'est pas tranché par le texte : l'app lance elle-même `claude -p`,
comme OpenContact lance Codex. Deux faits vont contre :

- faire tourner Claude Code dans un produit demande d'accepter les conditions
  commerciales d'Anthropic, et les limites des abonnements sont prévues pour
  « ordinary, individual usage » ;
- la documentation annonce que `claude -p` passera en mode `--bare` par
  défaut. Dans ce mode, Claude Code n'utilise plus la connexion de
  l'abonnement, seulement une clé API.

Les prix de l'API au 3 novembre 2026, par million de jetons en entrée et en
sortie : Opus 5.5 à 4 $ et 20 $, Sonnet 5.5 à 2 $ et 10 $, Haiku 5.5 à 0,10 $
et 0,50 $.

## 4. Les pannes de GNS3

Lu dans le dépôt `GNS3/gns3-skills`, version du 3 novembre 2026, licence
GPL-3.0.

- 51 fichiers et 858 pannes, comptées une par une. Le rapport de recherche
  disait 50 pannes : c'était le nombre de fichiers. Le dépôt lui-même annonce
  815 à un endroit et 840 à un autre.
- Tout est du réseau : BGP, OSPF, VLAN, MPLS, VPN, etc.
- Une panne est une description : un nom, une gravité, une difficulté, les
  symptômes, des pistes de dépannage, le matériel concerné.
- 25 pannes sur 858 disent comment les poser et comment revenir en arrière.
  Pour les autres, l'IA de GNS3 se débrouille à partir de la description.
- Aucune panne ne dit comment vérifier la réparation.

## 5. Hermes Agent

Lu dans son dépôt (`website/docs/user-guide/security.md`). Version 0.19.0 du
20 juillet 2026, licence MIT.

- Trois modes d'accord : `smart` (par défaut), `manual`, `off`.
- Une liste de commandes reste bloquée même en mode `off`, sans aucun moyen
  de la contourner. L'utilisateur peut y ajouter ses propres règles.

## 6. Les essais sous Linux

Sur la machine cloud, avec les paquets d'Ubuntu 24.04 (FRR 8.4.4, QEMU
8.2.2).

Le réseau virtuel. Chaque machine vide est un espace de noms réseau, relié
par une paire veth à un bridge. À la fin, la première machine envoie un ping
à la dernière.

| Machines | Montage | RAM prise | Démontage |
|---|---|---|---|
| 50 | 0,6 s | 34 Mo | 0,1 s |
| 300 | 4 s | 103 Mo | 0,6 s |

Les routeurs. Une chaîne de routeurs FRR en OSPF, un par espace de noms, avec
un hello d'une seconde. Le premier routeur envoie un ping au dernier.

| Routeurs | Lancement | RAM | Routes arrivées |
|---|---|---|---|
| 5 | 0,3 s | 91 Mo, 18 Mo par routeur | 11 s |
| 20 | 1,4 s | 367 Mo, 18 Mo par routeur | 11 s |

- Un câble coupé au milieu : la route disparaît en 0,2 à 10 secondes.
- Le câble remis : le ping revient en 10 à 11 secondes.
- Une panne se pose donc d'une commande, et un test simple (un ping, une
  route) la voit.

La remise à zéro d'un disque de VM, avec `qemu-img`. Une image de référence
qcow2 de 4 Go en lecture seule, et une couche jetable par-dessus.

| Geste | Temps |
|---|---|
| Créer la couche d'une machine | 12 ms |
| Prendre un instantané du disque | 24 ms |
| Revenir à l'instantané | 0,3 s |
| Remettre à zéro (jeter la couche, la recréer) | 0,5 s |

Pas mesuré ici : les conteneurs (pas de service Docker dans cet
environnement), les VM (pas de KVM), netem et Open vSwitch (absents du noyau
de cette machine).

## 7. La RAM et les versions

La RAM, d'après les documentations :

- Wazuh, tout sur une machine : 4 processeurs et 8 Go jusqu'à 25 machines
  surveillées (guide de démarrage, lu par extraits) ;
- un lab AD, client, OPNsense, Wazuh et Kali : environ 20 Go sur le papier.
  Ça tient dans 32 Go si la limite de WSL2 est montée (partie 2). À mesurer.

Les versions relevées sur crates.io et PyPI : Tauri 2.12.6, bollard 0.21.1,
rmcp 3.10.0, genai 0.7.5, rig-core 0.46.0, russh 0.65.1, gns3-server 3.0.6.

La documentation d'OPNsense n'a pas pu être lue (site non autorisé dans cet
environnement).

## 8. Ce qui reste à mesurer sur le PC du mainteneur

Le mainteneur ne veut pas de script à lancer sur son PC. Ces mesures se
feront dans l'app, quand elle existera.

- La version de WSL et de son noyau.
- KVM répond-il dans WSL2 (`/dev/kvm`) ?
- Le module de WiFi simulé se charge-t-il ?
- La RAM que prend l'environnement Linux à vide.
- Un Windows Server dans KVM sous WSL2 : le temps d'installation, la vitesse
  à l'usage.
- ESXi 8.0U3e dans KVM sous WSL2 : démarre-t-il, peut-il lancer une VM ?
  Aucun témoignage trouvé pour cet empilement.
- AD, client, OPNsense, Wazuh et Kali ensemble : la RAM réelle.
- La taille de l'installeur.
- Une panne inventée par un modèle local : le temps qu'elle prend, et quel
  modèle la pose bien.

Les trois premières sont des lectures, sans rien installer de plus que WSL2.
Les autres demandent de télécharger des images.

## 9. Ce qu'on en tire

- Le chemin par WSL2 tient sur le papier pour Windows 11 Famille : KVM, le
  réseau virtuel et l'iSCSI sont dans le noyau. Le WiFi simulé demande le
  noyau 6.18, l'app devra vérifier la version.
- Le réseau virtuel et les routeurs coûtent peu : 20 routeurs prennent
  367 Mo. La RAM ira aux VM.
- La remise à zéro d'un disque prend moins d'une seconde.
- Pour l'abonnement Claude, la voie sûre est le serveur MCP. Le mainteneur
  garde le serveur MCP pour plus tard, peut-être (voir
  [carnet.md](carnet.md)). Lancer `claude -p` depuis l'app repose sur un
  comportement qu'Anthropic annonce vouloir changer.
- Le catalogue de GNS3 est une liste de descriptions, sans vérification. Sa
  licence GPL-3.0 interdit de le copier dans une app sous MIT ou Apache. On
  peut s'inspirer de sa forme.
- Hermes garde une liste d'actions bloquées même en mode libre. C'est le
  garde-fou retenu pour Opn-Display.

## 10. Le réseau façon Cisco

Lu le 17 novembre 2026, après une question du mainteneur : l'app fait-elle
HSRP, EtherChannel, PVST+, RSTP, et le modèle cœur, distribution, accès ?

HSRP, PVST+ et PAgP sont propres à Cisco. LACP, VRRP, RSTP et MSTP sont des
normes. Le modèle cœur, distribution, accès est une façon de câbler, il ne
dépend d'aucun protocole.

Ce que dit Cisco :

- Les images de Cisco Modeling Labs (CML) ne servent pas hors de CML sans
  licence : « Downloading these VM images and using them outside Cisco
  Modeling Labs without a proper license is prohibited »
  ([VM Images for CML Labs](https://developer.cisco.com/docs/modeling-labs/vm-images-for-cml-labs/)).
- CML-Free existe depuis la version 2.8 : gratuit, sans licence, 5 machines
  à la fois. Ses images comprennent IOL, IOL-L2 et ASAv
  ([CML-Free](https://developer.cisco.com/docs/modeling-labs/cml-free/)).

Ce que fait le logiciel libre :

- VRRP : FRR le fait avec vrrpd, en versions 2 et 3, à partir du noyau
  Linux 5.1 ([doc de FRR](https://docs.frrouting.org/en/latest/vrrp.html)).
  Le noyau de WSL2 est en 6.6 ou 6.18 (partie 1).
- RSTP et MSTP : mstpd les ajoute aux bridges Linux. Son auteur prévient
  que la partie MSTP est peu testée ([mstpd](https://github.com/mstpd/mstpd)).
- PVST+ : une distribution, ALT Linux, l'annonce pour mstpd. Un ticket du
  projet montre qu'il ne parle pas avec un switch Cisco en Rapid-PVST
  ([ticket 80](https://github.com/mstpd/mstpd/issues/80)).
- Open vSwitch a RSTP dans son code
  ([rstp.h](https://sources.debian.org/src/openvswitch/3.6.0-5/lib/rstp.h)),
  et sa FAQ dit que sa partie spanning tree est peu testée.
- LACP : le bonding de Linux et Open vSwitch le font.

À mesurer à l'étape du réseau :

- RSTP et MSTP avec mstpd, sur un lab de plusieurs switchs ;
- VRRP avec FRR, dans le noyau de WSL2 ;
- PVST+ face à un switch Cisco, quand le pilotage de Packet Tracer ou de CML
  sera là.

Ce qu'on en tire :

- Dans l'app, le réseau se fait en libre, avec les normes. L'IA donne
  l'équivalent Cisco : VRRP pour HSRP, MSTP pour PVST+, LACP pour PAgP.
- Le vrai Cisco passe par Packet Tracer ou CML, pilotés. CML-Free suffit
  pour un petit lab Cisco de 5 machines.

## 11. Les machines du lab : Docker ou Incus

Mesuré le 17 novembre 2026 dans l'environnement cloud : Linux 6.18,
4 processeurs, 15 Go, sans KVM. Pour choisir comment faire tourner les
machines Linux du lab, à l'étape 3.

| | Docker 29.9 | Incus 6.0 |
|---|---|---|
| Ce que c'est | des conteneurs d'application : un programme par conteneur | des conteneurs système : une machine Linux entière, avec systemd |
| Le service au repos | 85 Mo (dockerd et containerd) | 75 Mo (incusd) |
| 3 machines démarrées | 0,8 s | 2,8 s, systemd prêt dans les trois |
| Mémoire par machine | environ 11 Mo | environ 12 Mo |
| `systemctl` dans la machine | non, le premier processus est le programme lui-même | oui, systemd est le premier processus |
| Instantané, puis retour | pas prévu pour | 1,1 s chacun, avec le stockage le plus simple (une copie) |
| Les VM | non | oui, par QEMU et KVM (pas essayé ici, faute de KVM) |
| Les images | Docker Hub, joignable d'ici | images.linuxcontainers.org, pas joignable d'ici. Image fabriquée ici : Ubuntu 24.04 avec systemd et nginx, 101 Mo |

Ce qu'on a vu en plus :

- Deux machines Incus reliées par un bridge à nous : le ping passe
  (0,05 ms) et nginx répond (HTTP 200).
- Docker met la chaîne FORWARD d'iptables à DROP, et fait passer les
  bridges par iptables. Les câbles du lab, sur nos propres bridges, sont
  alors coupés : le ping ne passe plus. Il repasse quand ce filtrage est
  coupé (`net.bridge.bridge-nf-call-iptables=0`).
- nginx ne démarrait pas dans la machine, parce qu'IPv6 manque dans cet
  environnement cloud. Sans la ligne IPv6 de sa configuration, il démarre.
- Debian 13 a Incus dans ses paquets, en version 6.0 LTS
  ([wiki de Debian](https://wiki.debian.org/Incus)).

Ce qu'on en tire :

- Dans un lab d'administration, on répare des services avec `systemctl`.
  Une machine Incus se comporte comme un vrai serveur, un conteneur Docker
  non.
- Incus fait aussi les VM et les instantanés : un seul outil pourrait
  servir des étapes 3 à 10.
- Docker et nos bridges ne vont pas ensemble sans règles en plus.
- À mesurer sur le PC du mainteneur : Incus dans WSL2, ses images, ses VM
  et un stockage plus rapide (btrfs).

## 12. Le premier lab dans Incus

Mesuré le 17 novembre 2026 dans l'environnement cloud, avec Incus 6.0.0 et
une image fabriquée ici : Ubuntu 24.04 avec systemd, réglée comme l'image
Debian 13 d'Incus. Pour l'étape 3.

Les câbles :

- Un câble est une paire veth. Chaque bout entre dans une machine comme une
  carte réseau (`nictype=physical`), sous le nom eth0 ou eth1.
- Le ping passe de pc1 à pc2 à travers le routeur, alors que les règles de
  Docker (FORWARD à DROP) sont en place : une paire veth ne passe par aucun
  bridge, ni par iptables.
- Une machine qui s'arrête rend son bout de câble au système, sous son nom.
  En face, l'interface dit NO-CARRIER, comme un câble branché sur une
  machine éteinte.
- Une machine qui redémarre d'elle-même (`reboot`) garde ses câbles.
- Les paires veth disparaissent quand le système redémarre, WSL compris :
  l'app les repose avant de démarrer les machines (essayé en les retirant à
  la main).

Les temps :

| Geste | Temps |
|---|---|
| Créer une machine | 0,3 s |
| Lancer le premier lab, avec la création des trois machines | 4,1 s |
| Arrêter une machine | 0,8 s |
| Arrêter le lab | 1 s |
| Redémarrer depuis la machine | environ 4 s |
| Remettre le lab à zéro | 8 s |
| Fermer l'app, les machines s'arrêtent avec elle | 2,8 s |

Le reste :

- Une machine du lab prend environ 14 Mo de mémoire en marche.
- `incus file push` depuis l'entrée crée le fichier en 0600, et
  systemd-networkd ne le lit plus. Avec `--mode 0644`, il le lit.
- Un terminal ouvert par `incus exec` dans un pseudo-terminal : la variable
  TERM passe, et la taille du terminal suit celle de la fenêtre.
- L'image Debian 13 d'Incus règle son réseau avec systemd-networkd, en DHCP
  sur eth0, et ses noms avec systemd-resolved. Lu dans la définition des
  images d'Incus,
  [lxc-ci, images/debian.yaml](https://github.com/lxc/lxc-ci/blob/main/images/debian.yaml).
- Le paquet incus 6.0.0-1, celui qu'Ubuntu 24.04 a repris de Debian, ne
  tire pas QEMU : les VM demanderont un paquet de plus. Le site des paquets
  de Debian n'est pas joignable d'ici, la version de Debian 13 (6.0.4) reste
  à vérifier.

À mesurer sur le PC du mainteneur : Incus et systemd dans WSL2, le
téléchargement de l'image Debian 13 et le temps de création d'une machine.

Lu le 17 novembre 2026 dans le code de wsl.exe
([microsoft/WSL, WslClient.cpp](https://github.com/microsoft/WSL/blob/master/src/windows/common/WslClient.cpp)) :
sans `--exec`, la commande passe par le shell de l'utilisateur
(`$SHELL -c`), qui relit ses `$` et ses guillemets. Avec `--exec`, les
arguments sont découpés comme sous Windows (`CommandLineToArgvW`) et passés
tels quels. Microsoft lance lui-même `wsl.exe -u root --exec /usr/bin/id`.
L'app passe toujours par `--exec`.

## 13. L'agent de l'IA

Mesuré le 17 novembre 2026 dans l'environnement cloud, pour l'étape 4.

Le poids de la boucle d'agent, dans un petit programme qui fait un appel à
un modèle, compilé en version finale :

| | Écrite à la main (ureq et serde_json) | genai 0.7.5 | rig-core 0.46 |
|---|---|---|---|
| Paquets tirés | 44 | 186 | 130 |
| Taille du programme | 1,2 Mo | 8,2 Mo | ne se compile pas ici : il demande Rust 1.97.1 |
| Première compilation | 15 s | 78 s | |

Un vrai modèle local, ici, pour essayer la boucle :

- Ollama 0.40.16, pris sur Docker Hub, avec Qwen3 1.7B (Q4_K_M, 1,1 Go),
  pris aussi sur Docker Hub. Ollama le reconnaît capable d'appeler des
  outils.
- Par l'API compatible OpenAI d'Ollama, avec un outil `lancer_commande` :
  le modèle appelle l'outil (`pc1`, `ping 10.0.2.10`) en 31 s sur les
  quatre processeurs d'ici, en réfléchissant d'abord. Sans réflexion, il
  répond en 8 s, mais décrit ce qu'il ferait au lieu d'appeler l'outil.
- Le ping demandé n'a pas de fin (`ping` sans `-c`). Une commande de l'IA
  doit donc être coupée par l'app au bout d'un temps donné.
- D'ici, seule l'API d'Anthropic répond. Celles d'OpenAI et de Mistral, et
  le registre d'Ollama, sont bloqués.

L'app avec ce modèle, lancée pour de vrai ici :

- « Brancher et essayer » : Ollama répond « prête » en quelques secondes.
- En Prof, « Fais le ping toi-même depuis pc1 vers 10.0.2.10 » : l'IA
  demande `ping -c 3 10.0.2.10` sur pc1, l'app montre la carte d'accord, et
  après « Autoriser » l'IA conclut que le routage entre les deux réseaux
  est bon. Environ deux minutes sur les quatre processeurs d'ici.
- En Coach, la même demande reçoit une question en retour (« Quel est l'IP
  de pc2 ? »), comme le veut ce niveau.
- Le petit modèle vouvoie parfois et met un emoji, malgré sa consigne. Un
  modèle plus gros la suit mieux ; l'emoji est maintenant défendu dans la
  consigne.

Codex 0.169.0, installé ici par npm : il accepte les réglages de l'app (son
shell coupé, le relais MCP branché) et lance le relais avec sa variable.
Sans compte ChatGPT ni accès au réseau d'OpenAI, il s'arrête à l'appel du
modèle. Ses messages « Reconnecting… » ne sont pas des échecs : l'app ne
prend que `turn.failed` pour un échec.

## 14. Les pannes du premier lab

Mesuré le 17 novembre 2026 dans l'environnement cloud, avec un vrai Incus,
pour la suite de l'étape 4.

- Les cinq pannes du catalogue se posent en 8 à 14 s : l'instantané, les
  commandes, puis les tests. La plus longue démarre aussi le lab. Juste
  après la pose, chacun de leurs tests échoue. Le lab revenu à
  l'instantané, ils passent tous.
- L'instantané des trois machines prend 4 s (11 s au premier essai), le
  retour 4 s, l'effacement 2,5 s. Le retour redémarre une machine qui
  tourne : ses terminaux se ferment, et l'app propose de les rouvrir.
- Le lab démarré, ses machines ont fini de démarrer 0,8 s plus tard
  (`systemctl is-system-running --wait`), et leur réseau répond aussitôt.
  Juste après `incus start`, systemctl ne joint pas encore systemd : il
  faut réessayer.
- Dans l'app lancée pour de vrai, la passerelle de pc2 changée : « Pas
  encore », 0 test sur 3. La route corrigée à chaud : 2 sur 3, le test du
  fichier de systemd-networkd échoue toujours. Le fichier corrigé : 3 sur 3.
- Qwen3 1.7B, sur les quatre processeurs d'ici, à qui l'app demande une
  panne inventée : 1 min 30 pour lire le lab, 1 min 50 pour poser une panne
  qui ne cassait rien (l'app l'a retirée), puis plus de 5 min pour la
  réponse suivante, que l'app a coupée. Ce modèle est trop lent pour
  inventer une panne ici.
- Le même chemin avec un faux modèle, qui donne toujours la même réponse :
  une panne inventée posée en 12 s, puis réparée et vérifiée dans l'app.
  Avec trois poses qui ne cassent rien, l'app arrête l'IA au bout de 43 s.
  « Arrêter » pendant qu'un modèle met deux minutes à répondre : l'app rend
  la main en une demi-seconde.

## 15. Le switch simple

Mesuré le 17 novembre 2026 dans l'environnement cloud, avec un vrai Incus,
avant de choisir ce que l'IA peut mettre dans un lab.

- Le switch est un conteneur Debian dont les ports (eth1, eth2, eth3)
  entrent dans un pont Linux, `br0`, réglé par systemd-networkd.
- Trois postes y sont branchés par des paires veth, dans 10.0.5.0/24 : ils
  se joignent tous.
- Docker tourne sur la même machine. Sa règle `FORWARD DROP` vit dans le
  réseau de l'hôte, pas dans celui du switch : elle ne le gêne pas.
- Le switch prend 13 Mo de RAM, comme un poste.
