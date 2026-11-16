#!/usr/bin/env bash
# Mesure ce que coûte un réseau virtuel fait de namespaces, de paires veth et
# d'un bridge : le temps pour le monter, la RAM prise, le temps pour le défaire.
# Usage : sudo ./reseau-virtuel.sh [nombre de machines]
set -euo pipefail

N=${1:-50}
PREFIXE=odm

ram_libre_ko() { awk '/MemAvailable/ {print $2}' /proc/meminfo; }
maintenant_ms() { date +%s%3N; }

nettoyer() {
  for ns in $(ip netns list | awk '{print $1}' | grep "^${PREFIXE}" || true); do
    ip netns del "$ns"
  done
}
trap nettoyer EXIT
nettoyer

ram_avant=$(ram_libre_ko)
debut=$(maintenant_ms)

ip netns add ${PREFIXE}sw
ip -n ${PREFIXE}sw link add br0 type bridge
ip -n ${PREFIXE}sw link set br0 up

for i in $(seq 1 "$N"); do
  ns=${PREFIXE}$i
  ip netns add "$ns"
  ip link add "v$i" netns "$ns" type veth peer name "p$i" netns ${PREFIXE}sw
  ip -n ${PREFIXE}sw link set "p$i" master br0 up
  ip -n "$ns" addr add "10.77.$((i / 250)).$((i % 250 + 1))/16" dev "v$i"
  ip -n "$ns" link set "v$i" up
  ip -n "$ns" link set lo up
done

monte_ms=$(( $(maintenant_ms) - debut ))
ram_apres=$(ram_libre_ko)

dernier="10.77.$((N / 250)).$((N % 250 + 1))"
ping_ok=non
if ip netns exec ${PREFIXE}1 ping -c 3 -W 2 -q "$dernier" > /tmp/odm-ping.txt 2>&1; then
  ping_ok=oui
fi
rtt=$(awk -F/ '/rtt|round-trip/ {print $5}' /tmp/odm-ping.txt)

debut=$(maintenant_ms)
nettoyer
defait_ms=$(( $(maintenant_ms) - debut ))

echo "machines            : $N"
echo "montage             : ${monte_ms} ms"
echo "RAM prise           : $(( (ram_avant - ram_apres) / 1024 )) Mo, soit $(( (ram_avant - ram_apres) / N )) Ko par machine"
echo "ping 1 vers $N      : $ping_ok, ${rtt:-?} ms en moyenne"
echo "démontage           : ${defait_ms} ms"
