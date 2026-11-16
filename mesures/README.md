# Les sondes

Trois scripts qui mesurent ce que coûte le moteur sous Linux. Les résultats
du 16 novembre 2026 sont dans [../docs/mesures.md](../docs/mesures.md).

Ils demandent Linux et les droits root. Chacun nettoie ce qu'il a créé.

| Script | Ce qu'il mesure | Paquets Ubuntu |
|---|---|---|
| `reseau-virtuel.sh [machines]` | Le temps et la RAM pour monter un réseau de machines vides reliées à un switch | `iproute2`, `iputils-ping` |
| `routeurs-frr.sh [routeurs]` | La RAM d'un routeur FRR, le temps pour que les routes arrivent, puis un câble coupé et remis | `iproute2`, `iputils-ping`, `frr`, `procps` |
| `remise-a-zero.sh [dossier]` | La création d'une couche de disque, un instantané, le retour en arrière, la remise à zéro | `qemu-utils` |

```
sudo ./reseau-virtuel.sh 50
sudo ./routeurs-frr.sh 5
./remise-a-zero.sh
```
