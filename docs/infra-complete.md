# Une vraie infra dans l'app

Le mainteneur, le 17 novembre 2026 : « il faut que ce soit comme les
concurrents, voire mieux, de façon générale et dans les détails », et
« qu'une véritable infra puisse tourner et être émulée, que ce soit au niveau
de l'équilibrage de charge, la redondance, la sécurité, les ports, les racks
et tout ce que j'ai oublié ».

Ce fichier fait l'inventaire, famille par famille : ce qu'une infra
contient, avec quoi l'app l'émule, et ce qui fait la qualité dans les
détails. Les quatre points posés au mainteneur ont été tranchés le même
jour ([questions-ouvertes.md](questions-ouvertes.md)). Ce qui reste marqué
« proposé » n'est pas encore décidé.

## 1. Ce que font les concurrents

Relevé le 17 novembre 2026 dans leurs documentations (sources en bas de
page). Un tiret veut dire « pas relevé », pas « absent ».

| | GNS3 | EVE-NG | Packet Tracer | CML |
|---|---|---|---|---|
| Capturer le trafic d'un câble | oui, dans Wireshark, ou dans le navigateur depuis la 3.1 | oui, Wireshark intégré, Ethernet seulement | le mode simulation montre les paquets | oui, dans les deux sens, limité en nombre de paquets ou en temps |
| Dégrader un câble | latence, gigue, perte, corruption | débit, délai, gigue, perte | - | débit, latence, gigue, perte, et un nœud « WAN » à poser sur un lien |
| Couper un câble sans le retirer | oui (« suspend ») | - | - | un lien s'arrête |
| Brancher un câble sur une machine en marche | - | oui | - | - |
| Vue physique | - | - | villes, bâtiments, armoires, baies et tables, longueur des câbles | - |
| Longueur des câbles | - | - | oui : au-delà de 100 m, l'Ethernet ne passe plus | - |

L'app reprend tout ce tableau. Ce qu'elle fait de plus :

- l'IA, qui construit le lab, pose les pannes et vérifie la réparation ;
- une vue des baies reliée au schéma : les mêmes machines, les mêmes câbles,
  et l'état de chaque port des deux côtés ;
- des pannes physiques : un câble coupé, une alimentation qui lâche, un
  onduleur à plat ;
- le temps de bascule mesuré quand une redondance prend le relais ;
- les documents du dossier tirés du lab : plan d'adressage, plan des baies,
  matrice des flux ;
- tout en français.

## 2. Les familles d'une vraie infra

L'étape dit où chaque famille arrive dans l'ordre de construction
([questions-ouvertes.md](questions-ouvertes.md)). Les machines Linux tournent
en conteneurs Incus, le reste en VM.

### Le réseau (étape 5)

- Des switchs avec leurs vrais ports : 8, 24 ou 48, un câble par port.
  Linux bridge ou Open vSwitch.
- VLAN et trunks 802.1Q, routage entre VLAN.
- RSTP et MSTP avec mstpd, LACP avec le bonding de Linux ou Open vSwitch.
- Routage statique, OSPF et BGP avec FRR, ECMP pour plusieurs chemins.
- NAT et PAT avec nftables, relais DHCP, IPv6 à côté d'IPv4.
- La qualité de service avec tc.
- La recopie de port (SPAN) avec tc ou Open vSwitch, pour un IDS ou une
  capture.
- VXLAN et EVPN avec FRR, pour le réseau d'un datacenter.

### Les ports

- Chaque machine a ses cartes réseau, chaque switch ses ports numérotés. Un
  câble se branche sur un port choisi.
- L'état de chaque port se voit sur le schéma et dans la baie : branché,
  éteint, bloqué par STP, membre d'un agrégat.
- Les ports TCP et UDP aussi : la fiche d'une machine montre ce qui écoute,
  les règles du pare-feu les ouvrent ou les ferment, et un scan nmap depuis
  une machine du lab les retrouve.
- La sécurité des ports : nombre d'adresses MAC limité, ports isolés. À
  mesurer avec Open vSwitch.

### Les baies (étape 5)

- Une vue physique, à côté du schéma : des salles, des baies 42U, chaque
  machine à sa hauteur en U, sa face avant et ses ports.
- Des panneaux de brassage, des câbles étiquetés, avec leur longueur.
- L'alimentation : deux arrivées A et B, des machines à une ou deux
  alimentations, un onduleur (NUT sait simuler le sien).
- Des pannes physiques : couper l'arrivée A, et seules les machines à une
  alimentation s'éteignent.
- Le plan des baies s'exporte pour le dossier.

### L'équilibrage de charge (étape 6)

- HAProxy, en couche 4 et en couche 7, avec ses tests de santé.
- nginx en répartiteur, keepalived avec IPVS (LVS), le tourniquet DNS.
- Les sessions collées à un serveur, le TLS terminé au répartiteur.
- Des clients qui envoient du trafic, pour voir la charge se répartir.
- La vérification : un serveur tombe, le trafic continue, et l'app compte
  les requêtes perdues.

### La redondance et la haute disponibilité (étape 6)

- Réseau : VRRP avec keepalived ou FRR, LACP, RSTP et MSTP, deux liens
  montants, deux accès Internet.
- Serveurs : Pacemaker et Corosync, une adresse qui passe d'une machine à
  l'autre.
- Données : la réplication de MariaDB (Galera) ou de PostgreSQL, le RAID
  logiciel dans une VM.
- Services : un DNS secondaire, le DHCP en paire (Kea), deux contrôleurs de
  domaine.
- Pare-feu : deux OPNsense en CARP.
- La vérification : l'app coupe, éteint ou débranche, puis mesure le temps
  de bascule (les pings perdus).

### La sécurité (étape 8)

- La segmentation : VLAN, DMZ, zones de confiance.
- Les pare-feu : nftables, OPNsense.
- La détection : Suricata en IDS ou IPS, Wazuh pour les journaux de
  sécurité.
- Le web : un pare-feu applicatif (ModSecurity ou Coraza) devant les sites.
- Les VPN : WireGuard, OpenVPN, IPsec avec strongSwan.
- Une autorité de certification interne, et les services du domaine
  Windows quand il y en a un.
- 802.1X sur les ports et en WiFi, avec FreeRADIUS.
- Un bastion pour administrer, un proxy web, un DNS qui filtre.
- Les audits : Lynis pour le durcissement, Greenbone pour les failles.
- L'attaque : Kali, qui ne vise que les machines du lab (garde-fou 4).

### Les services d'infrastructure (étape 6)

- DNS (BIND, Unbound), DHCP (Kea), l'heure (chrony).
- L'annuaire : Active Directory sur Windows Server, Samba AD en conteneur
  quand Windows n'est pas là, FreeIPA.
- Web (nginx, Apache), bases de données (MariaDB, PostgreSQL), messagerie
  (Postfix, Dovecot), partages (Samba, NFS).
- L'inventaire et les tickets avec GLPI.
- Le déploiement par le réseau (PXE), la configuration avec Ansible.
- Les conteneurs : Docker, et Kubernetes avec k3s.

### La supervision et les journaux (étape 6)

- Zabbix, Centreon, Prometheus et Grafana, LibreNMS par SNMP.
- Les journaux centralisés : rsyslog, Graylog ou OpenSearch.
- Les alertes servent aussi à l'IA, pour expliquer un incident.

### La sauvegarde et la reprise (étape 6)

- Bareos ou Bacula, restic ou BorgBackup, Proxmox Backup Server en VM.
- La règle 3-2-1, et des restaurations qu'on essaie pour de vrai.
- Le plan de reprise : un second site relié par un WAN lent. Le premier
  tombe, on repart sur le second.

### La virtualisation et le stockage (étapes 7, 9 et 10)

- ESXi, déjà prévu, et Proxmox VE, gratuit et libre.
- Un SAN en iSCSI, un NAS (TrueNAS, Samba, NFS).
- Un cluster Proxmox avec Ceph, si la mémoire le permet. À mesurer.

### Les postes et les utilisateurs

- Des postes Windows (image fournie par l'utilisateur) et Linux.
- Des utilisateurs simulés qui naviguent, se connectent et échangent des
  fichiers : une infra qui vit, pour la supervision, la sécurité et la
  charge.

### Le WAN et un Internet simulé

- Des sites reliés par des liens lents, avec latence et perte.
- Un faux Internet : des routeurs aux adresses publiques, un DNS racine, des
  sites web, deux fournisseurs d'accès pour la redondance.
- L'accès au vrai Internet reste coupé. Il s'ouvre sur demande, le temps
  d'installer des paquets (proposé).

### Le WiFi (étape 9)

- Déjà prévu : mac80211_hwsim, hostapd, FreeRADIUS. WPA2 et WPA3 Entreprise,
  un portail captif.

## 3. Les détails qui font la qualité

- Capturer sur n'importe quel câble, lire la capture dans l'app, ou l'ouvrir
  dans Wireshark s'il est installé.
- Couper, rebrancher et dégrader un câble. L'IA s'en sert aussi pour les
  pannes.
- Brancher un câble sur une machine en marche, sans la redémarrer.
- Des noms de ports lisibles, des adresses MAC fixes (fait à l'étape 3).
- Plusieurs terminaux par machine, le copier-coller, la recherche dans la
  sortie, et le journal de chaque session pour le replay.
- L'écran des machines qui en ont un (Windows), et les interfaces web des
  outils du lab (OPNsense, Zabbix, GLPI, Wazuh) ouvertes depuis l'app.
  Sous Windows, le réseau du lab n'est pas joignable depuis Windows : l'app
  doit faire le pont.
- Les machines démarrent dans l'ordre : l'annuaire avant les serveurs qui en
  dépendent.
- Un lab se sauvegarde et se partage en un fichier. Des modèles de labs.
- Des zones sur le schéma : les sites, les VLAN en couleur, des notes.
- Les documents du dossier tirés du lab : plan d'adressage, plan des baies,
  matrice des flux.
- Un lab trop gros pour la mémoire le dit avant de démarrer (maquette
  `nouveau-lab-trop`).

## 4. Ce qui ne s'émule pas

- Les images Cisco hors de CML : interdit par Cisco. Elles passent par le
  pilotage ([mesures.md](mesures.md), partie 10).
- Les images d'autres constructeurs (MikroTik, VyOS, Fortinet, Palo Alto) :
  GNS3 et EVE-NG les font tourner quand l'utilisateur les fournit. L'app
  fera pareil, quand la licence le permet (décidé le 17 novembre 2026).
- La radio WiFi, le Fibre Channel, le courant et la clim : simulés dans la
  vue des baies, pas émulés.

## 5. Ce qui reste à vérifier

- Le référentiel officiel du titre AIS (RNCP 37680) : ses trois blocs sont
  connus (administrer et sécuriser les infrastructures, concevoir et mettre
  en œuvre une solution, participer à la gestion de la cybersécurité). Le
  détail des compétences n'a pas pu être lu d'ici. Il dira quelles familles
  passent en premier.
- Chaque logiciel cité se mesure à son étape : sa place en mémoire, et s'il
  tourne en conteneur ou demande une VM.

## Sources

- GNS3 : [les filtres de paquets](https://docs.gns3.com/docs-3.1-en/web-ui/use-packet-filter),
  [la capture d'un lien (API)](https://gns3-server.readthedocs.io/en/latest/api/v2/controller/link/projectsprojectidlinkslinkidstartcapture.html)
- EVE-NG : [ses fonctions](https://www.eve-ng.net/index.php/features/)
- Packet Tracer : [la vue physique](https://tutorials.ptnetacad.net/help/default/workspace_physical.htm),
  [les longueurs de câble](https://tutorials.ptnetacad.net/help/default/workspace_physical_distanceMeasurements.htm)
- CML : [la dégradation des liens](https://developer.cisco.com/docs/modeling-labs/link-conditioning/),
  [la capture](https://developer.cisco.com/docs/modeling-labs/link-packet-capture),
  [le nœud WAN](https://developer.cisco.com/docs/modeling-labs/wan-emulator)
- Le titre AIS : [sa fiche chez Intercariforef](https://www.intercariforef.org/formations/certification-114912.html)
