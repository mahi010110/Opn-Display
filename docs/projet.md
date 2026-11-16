# Opn-Display

Opn-Display est un nom de travail. Rien n'est codé.

Ce fichier dit ce que le mainteneur a décidé le 16 novembre 2026. Ce qui
reste à discuter est dans [questions-ouvertes.md](questions-ouvertes.md),
l'historique dans [carnet.md](carnet.md), la façon de construire l'app dans
[architecture.md](architecture.md), et ce qui a été mesuré dans
[mesures.md](mesures.md).

## 1. En une phrase

Une app de bureau qui fait tourner une infrastructure informatique entière
sur ton PC, avec une IA qui la construit, la casse, l'explique et vérifie tes
réparations.

C'est le projet fil rouge du mainteneur pour le titre AIS, en juillet 2027.
L'app doit servir à tout le monde, pas seulement aux apprenants de cette
formation.

## 2. D'où vient l'idée

En formation, on travaille sur des outils séparés : Packet Tracer, VMware
Workstation, les bacs à sable du cloud. On s'entraîne peu au dépannage, parce
qu'il faut quelqu'un pour casser les choses.

Deux familles d'outils existent déjà :

- GNS3 et EVE-NG font tourner de vraies machines (routeurs, serveurs,
  pare-feu) reliées par un réseau virtuel. C'est l'émulation ;
- Hermes Agent, de Nous Research, est un agent IA qui agit sur un ordinateur,
  accepte n'importe quel modèle et se souvient de son utilisateur.

Opn-Display réunit les deux, et ajoute les cours, les pannes à réparer et la
vérification.

| | GNS3 | EVE-NG | Hermes Agent | Opn-Display (visé) |
|---|---|---|---|---|
| Fait tourner des machines | Oui : VM, conteneurs, anciens Cisco | Oui : VM, conteneurs | Non | Oui : VM et conteneurs |
| Domaines | Surtout le réseau | Surtout le réseau | Aucun en particulier | Toute l'infrastructure |
| IA | Un assistant et des pannes réseau, en version alpha (3.1) | Aucune | C'est un agent IA | Elle construit, casse, vérifie et enseigne |
| L'IA agit sur le PC | Non | Non | Oui, avec des modes d'accord | Oui, réglable |
| Pannes à réparer | 858 pannes de réseau décrites, et les étapes de la pose s'affichent | Non | Non | Un catalogue et des pannes inventées |
| Vérifie la réparation | Pas documenté | Non | Non | Oui, l'IA avec ses tests |
| Cours | Non | Non | Non | Oui |
| Prix | Gratuit, GPL-3.0 | 7 machines en gratuit, 150 € par an en Pro | Gratuit, MIT | Gratuit et libre |

Les faits sur GNS3, EVE-NG et Hermes Agent viennent des recherches du
13 novembre 2026 (partie 9). Une partie a été revérifiée le 16 novembre
([mesures.md](mesures.md)).

## 3. Ce que fait l'app

### Elle émule

L'app fait tourner elle-même les machines du lab, comme GNS3 et EVE-NG. Pour
juillet 2027, le mainteneur veut tout ce qui peut s'émuler :

- des serveurs Linux et leurs services (web, DNS, DHCP, SSH, partages) ;
- le réseau : routeurs, switchs, VLAN, routage, NAT ;
- Windows Server et Active Directory ;
- les pare-feu et la sécurité : OPNsense, VPN, Wazuh, une machine Kali ;
- ESXi ;
- un SAN et un NAS (iSCSI, NFS, SMB) ;
- le WiFi simulé.

Le lab tourne sur le PC de l'utilisateur.

### Elle pilote le reste

Ce que l'app ne peut pas émuler, elle le fait faire à un logiciel installé
sur le PC, et elle le commande : Packet Tracer, VMware Workstation, Cisco
Modeling Labs, les bacs à sable du cloud. Le pilotage se construit après
l'émulation.

### L'IA supervise

- Elle construit le lab avec toi, ou à ta place à partir d'une description.
- Elle glisse des pannes que tu dois trouver et réparer.
- Elle dit si ta réparation est bonne.
- Elle donne des cours et explique.
- Elle agit sur le PC quand il le faut : installer un outil, récupérer une
  image, lancer un logiciel.

## 4. L'IA

### La brancher

En juillet 2027, l'app accepte :

- un abonnement (Claude, ChatGPT), par l'outil officiel de l'éditeur installé
  sur le PC. La coquille native d'OpenContact le fait déjà avec Codex. Pour
  Claude, les règles d'Anthropic limitent cette voie (voir
  [questions-ouvertes.md](questions-ouvertes.md)) ;
- une clé API ;
- un modèle local (Ollama).

### Ce qu'elle peut faire

Sur le PC : installer des outils, piloter les logiciels, lancer des
commandes, lire et écrire des fichiers.

Dans le lab : taper des commandes sur les machines, changer une
configuration.

### Les réglages

Ces comportements se règlent :

- sa liberté sur le PC : libre, avec accord, ou limitée aux outils de lab ;
- sa liberté dans le lab : libre, ou avec accord ;
- ce qu'elle fait quand tu demandes la solution d'une panne ;
- si elle te parle d'elle-même, ou seulement quand tu lui parles ;
- sa mémoire d'une session à l'autre : la voir, la corriger, la couper,
  l'effacer.

L'utilisateur choisit entre trois façons de régler :

- trois niveaux tout faits : Prof (elle explique tout), Coach (seulement des
  indices), Silencieuse (mode examen) ;
- des réglages séparés, un par comportement ;
- une IA qui s'adapte seule, selon ton niveau et ce que tu fais.

### Les pannes

Elles viennent d'un catalogue écrit à l'avance, et l'IA peut aussi en
inventer en direct.

### La vérification

C'est l'IA qui dit si la panne est réparée. Elle s'appuie sur les tests
qu'elle lance : un ping, l'état d'un service, une connexion.

### Les garde-fous

Proposés par Claude et retenus le 16 novembre 2026. Le mainteneur a répondu
« je te fais confiance », il peut revenir dessus.

- L'IA écrit ses tests au moment où elle pose la panne, pas après. L'app
  montre les tests lancés et leurs résultats.
- L'app sauvegarde le lab avant chaque panne inventée, pour pouvoir revenir
  en arrière.
- Sur le PC, le réglage par défaut est « avec accord ».
- Certaines actions demandent toujours un oui, même en mode libre : supprimer
  des fichiers, toucher aux protections de Windows.
- Kali et l'IA n'attaquent que les machines du lab, jamais un vrai réseau.

## 5. Les cours et les images

Les cours : des cours prêts pour les sujets courants, livrés avec l'app, et
des cours que l'IA écrit en direct pour le reste.

Les images : l'app ne peut pas contenir Windows Server ni ESXi, leurs
licences l'interdisent. L'IA récupère l'image sur le site officiel de
l'éditeur quand c'est possible, et l'utilisateur accepte les conditions.
Sinon, elle le guide pour fournir l'image lui-même.

## 6. Juillet 2027, et après

En juillet 2027 :

- une version publique, que tout le monde peut télécharger ;
- en français ;
- sur Windows 11, Linux et macOS ;
- avec tout ce qui peut s'émuler (partie 3) ;
- avec les trois façons de brancher l'IA ;
- avec un design propre à Opn-Display, qui ne reprend pas celui
  d'OpenContact.

Après l'examen :

- le multijoueur (duel, coop, attaque et défense entre joueurs) ;
- la voix ;
- le lab dans le navigateur ;
- le lab sur un serveur distant.

## 7. Les limites connues

Elles viennent des recherches. Aucune n'a été mesurée sur le PC du
mainteneur.

- L'app ne peut contenir ni Windows, ni ESXi, ni les images Cisco, ni Packet
  Tracer.
- Le vrai WiFi radio et le Fibre Channel ne s'émulent pas. Le WiFi simulé ne
  marche qu'entre machines Linux, et sous Windows il demande le noyau 6.18 de
  WSL2.
- Sur un Mac à puce Apple, Windows Server et ESXi sont trop lents pour
  servir.
- Tout ne tourne pas en même temps. Avec 32 Go, un domaine AD, un pare-feu et
  du stockage tiennent ensemble. Un vSphere avec vCenter demande 30 Go à lui
  seul.
- ESXi 9 n'est pas gratuit. Seul ESXi 8.0U3e l'est, avec un compte Broadcom.
- L'IA peut se tromper, sur un cours comme sur une vérification.

## 8. L'examen

- Le titre professionnel Administrateur d'infrastructures sécurisées (AIS),
  niveau 6, RNCP 37680. L'examen est en juillet 2027.
- Le formateur a vu le sujet et l'a validé.
- Le centre a confirmé que l'examen se passe sur le référentiel actuel.
- La formation se fait en centre, avec un stage.
- Le PC du développement et de la démo : Windows 11 Famille, 32 Go de RAM.

## 9. Les recherches

Elles sont dans le dépôt `mahi010110/Opn...`, branche
`claude/sweet-hypatia-p4qlsl` :

- `reports/Labs informatiques locaux pilotés par IA.md` : ce qui existe déjà,
  ce qui est faisable et légal sur un PC ;
- `research_notes/Labs informatiques locaux pilotés par IA/` : les notes de
  cette première recherche ;
- `research_notes/Technologies et architecture du lab IA/` : cinq notes sur
  le moteur, l'IA, l'interface, le multijoueur et les contenus. Leur compte
  rendu de synthèse n'a jamais été écrit ;
- `docs/projets/projet-2-lab-ia.md` : la première fiche du projet. Elle sert
  à comprendre l'idée. Elle ne dit pas ce qui est décidé, et plusieurs de ses
  choix ont été changés le 16 novembre 2026 (voir [carnet.md](carnet.md)).
