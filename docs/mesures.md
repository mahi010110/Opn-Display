# Les mesures du 16 novembre 2026

Faites avant de choisir la pile, l'ordre de construction et la licence. Les
sondes sont dans `mesures/` et se relancent (voir
[../mesures/README.md](../mesures/README.md)).

Tout a été fait depuis l'environnement cloud : une machine Linux de 4
processeurs et 15 Go, sans KVM. Rien n'a été mesuré sur le PC du mainteneur.
Ce qui lui revient est en partie 8.

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

Le réseau virtuel (`mesures/reseau-virtuel.sh`), des machines vides reliées à
un switch :

| Machines | Montage | RAM prise | Démontage |
|---|---|---|---|
| 50 | 0,6 s | 34 Mo | 0,1 s |
| 300 | 4 s | 103 Mo | 0,6 s |

Les routeurs (`mesures/routeurs-frr.sh`), une chaîne de routeurs FRR en
OSPF :

| Routeurs | Lancement | RAM | Routes arrivées |
|---|---|---|---|
| 5 | 0,3 s | 91 Mo, 18 Mo par routeur | 11 s |
| 20 | 1,4 s | 367 Mo, 18 Mo par routeur | 11 s |

- Un câble coupé au milieu : la route disparaît en 0,2 à 10 secondes.
- Le câble remis : le ping revient en 10 à 11 secondes.
- Une panne se pose donc d'une commande, et un test simple (un ping, une
  route) la voit.

La remise à zéro d'un disque de VM (`mesures/remise-a-zero.sh`), une image de
référence de 4 Go et une couche jetable :

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

- La version de WSL et de son noyau (`uname -r`).
- KVM répond-il dans WSL2 (`/dev/kvm`) ?
- Le module de WiFi simulé se charge-t-il ?
- La RAM que prend l'environnement Linux à vide.
- Un Windows Server dans KVM sous WSL2 : le temps d'installation, la vitesse
  à l'usage.
- ESXi 8.0U3e dans KVM sous WSL2 : démarre-t-il, peut-il lancer une VM ?
  Aucun témoignage trouvé pour cet empilement.
- AD, client, OPNsense, Wazuh et Kali ensemble : la RAM réelle.
- La taille de l'installeur.

Les trois premières se lisent en deux minutes, sans rien installer de plus
que WSL2. Les autres demandent de télécharger des images.

## 9. Ce qu'on en tire

- Le chemin par WSL2 tient sur le papier pour Windows 11 Famille : KVM, le
  réseau virtuel et l'iSCSI sont dans le noyau. Le WiFi simulé demande le
  noyau 6.18, l'app devra vérifier la version.
- Le réseau virtuel et les routeurs coûtent peu : 20 routeurs prennent
  367 Mo. La RAM ira aux VM.
- La remise à zéro d'un disque prend moins d'une seconde.
- Pour l'abonnement Claude, la voie sûre est le serveur MCP. Lancer
  `claude -p` depuis l'app repose sur un comportement qu'Anthropic annonce
  vouloir changer.
- Le catalogue de GNS3 est une liste de descriptions, sans vérification. Sa
  licence GPL-3.0 interdit de le copier dans une app sous MIT ou Apache. On
  peut s'inspirer de sa forme.
- Hermes garde une liste d'actions bloquées même en mode libre. C'est le
  garde-fou retenu pour Opn-Display.
