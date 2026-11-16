#!/usr/bin/env bash
# Mesure la remise à zéro d'un disque de VM : une image de référence en
# lecture seule, une couche jetable par-dessus. On mesure la création de la
# couche, la remise à zéro (jeter et recréer), et un instantané du disque
# (machine arrêtée) avec son retour en arrière.
# Usage : ./remise-a-zero.sh [dossier de travail]
set -euo pipefail

DOSSIER=${1:-$(mktemp -d)}
mkdir -p "$DOSSIER"
BASE=$DOSSIER/reference.qcow2
COUCHE=$DOSSIER/machine.qcow2

chrono_ms() {
  local debut; debut=$(date +%s%3N)
  "$@" > /dev/null
  echo $(( $(date +%s%3N) - debut ))
}

qemu-img create -q -f qcow2 "$BASE" 40G
for bloc in 0 1 2 3; do
  qemu-io -c "write -P 0xab ${bloc}G 1G" "$BASE" > /dev/null
done
chmod a-w "$BASE"

creer() { qemu-img create -q -f qcow2 -b "$BASE" -F qcow2 "$COUCHE"; }
remettre_a_zero() { rm -f "$COUCHE"; creer; }

creation_ms=$(chrono_ms creer)
qemu-io -c "write -P 0xcd 1G 1G" "$COUCHE" > /dev/null
taille_couche=$(du -m "$COUCHE" | cut -f1)

instantane_ms=$(chrono_ms qemu-img snapshot -c avant-panne "$COUCHE")
qemu-io -c "write -P 0xef 2G 512M" "$COUCHE" > /dev/null
retour_ms=$(chrono_ms qemu-img snapshot -a avant-panne "$COUCHE")

zero_ms=$(chrono_ms remettre_a_zero)

echo "image de référence     : 40 Go annoncés, $(du -m "$BASE" | cut -f1) Mo écrits"
echo "création d'une couche  : ${creation_ms} ms"
echo "couche après 1 Go écrit: ${taille_couche} Mo"
echo "instantané du disque   : ${instantane_ms} ms"
echo "retour à l'instantané  : ${retour_ms} ms"
echo "remise à zéro          : ${zero_ms} ms"

chmod u+w "$BASE"
rm -rf "$DOSSIER"
