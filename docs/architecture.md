# Architecture

Une proposition. Rien n'est mesuré, rien n'est codé.

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

- Rien à installer à la main. L'app installe et règle son environnement Linux
  toute seule.
- Tous les domaines, pas seulement le réseau.
- L'IA, qui a les mêmes commandes que l'interface.

### Sur chaque système

| Système | Environnement Linux | Ce qu'on sait |
|---|---|---|
| Windows 11 | Une distribution WSL2 propre à l'app, avec KVM imbriqué | L'imbrication existe sous Windows 11. À mesurer sur l'édition Famille |
| Linux | Le système lui-même, avec KVM | Le cas le plus simple |
| macOS | Une VM Linux lancée par l'app (Lima) | Sur puce Apple, seules les machines ARM tournent à vitesse normale |

### Les machines

- Des conteneurs pour ce qui est léger : services Linux, routeurs FRR,
  clients. Ils démarrent en quelques secondes.
- Des VM QEMU pour le reste : Windows Server, OPNsense, Kali, ESXi.
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

### ESXi, le stockage, le WiFi

- ESXi : une VM QEMU avec la virtualisation imbriquée. Sous Windows, ça fait
  trois niveaux : Windows, WSL2, puis ESXi et ses propres VM. Personne ne dit
  si ça marche, c'est à mesurer. L'autre voie est de piloter VMware
  Workstation, mais Workstation ne sait pas imbriquer quand Hyper-V est
  actif, et WSL2 en a besoin.
- Le stockage : la cible iSCSI du noyau Linux pour le SAN, Samba et NFS en
  conteneurs pour le NAS. TrueNAS demande 8 Go, il reste possible avec 32 Go.
- Le WiFi : le module `mac80211_hwsim` du noyau simule des cartes WiFi,
  hostapd fait le point d'accès, FreeRADIUS l'authentification. Deux notes de
  recherche se contredisent sur la présence de ce module dans le noyau de
  WSL2. À mesurer.

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
  commandes dangereuses et de fichiers protégés reste active.

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
| Abonnement | L'app lance l'outil officiel installé sur le PC (Codex pour ChatGPT, Claude Code pour Claude) | Fait dans `natif/` d'OpenContact avec Codex. À vérifier pour Claude |
| Assistant extérieur | L'app expose un serveur MCP, et l'utilisateur y branche son propre assistant | Fait dans `natif/` d'OpenContact |

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
- Cisco Modeling Labs : une API REST ;
- Packet Tracer : un canal prévu pour les extensions. L'IA peut y charger un
  lab, mais voit mal ce que l'utilisateur y fait, le logiciel est fermé ;
- le cloud : les outils en ligne de commande des hébergeurs, avec les accès
  de l'utilisateur.

## 5. La pile proposée

Ce sont les pistes des recherches. Rien n'est choisi.

| Partie | Piste | Pourquoi |
|---|---|---|
| App de bureau | Tauri 2, en Rust | Déjà utilisé dans `natif/` d'OpenContact et prévu pour Opn-Link. Installeur léger |
| Interface | Une page web dans Tauri | Les briques de terminal et de schéma existent pour le web |
| Environnement Linux | WSL2 sous Windows, Lima sous macOS | Aucune VM à gérer soi-même |
| Conteneurs | Docker ou containerd, commandé depuis Rust (bollard) | Connu, documenté |
| VM | QEMU, commandé par QMP et l'agent invité | Ce que font GNS3 et EVE-NG |
| Agent | Une boucle en Rust (rig-core ou genai), MCP avec rmcp | Pas de Python à embarquer |
| Terminaux | xterm.js | Le plus répandu |
| Écran des VM | noVNC, branché sur QEMU | Marche pour Windows comme pour Linux |
| Données locales | SQLite | Un seul fichier, sur le PC |
| Distribution | GitHub Releases | Gratuit. La signature Windows peut l'être aussi, Apple coûte 99 $ par an |

## 6. Avant de choisir

L'ordre de construction, la licence et la pile dépendent des mesures. Elles
sont listées dans [questions-ouvertes.md](questions-ouvertes.md), et leurs
résultats iront dans `mesures.md`.
