#!/usr/bin/env bash
# Mesure une chaîne de routeurs FRR en OSPF, chacun dans son namespace :
# la RAM par routeur, le temps pour que les routes arrivent, puis une panne
# (un câble coupé) : le temps pour qu'elle se voie et le temps pour revenir.
# Usage : sudo ./routeurs-frr.sh [nombre de routeurs]
set -euo pipefail

R=${1:-5}
FRR=/usr/lib/frr
PREFIXE=odr

maintenant_ms() { date +%s%3N; }

nettoyer() {
  for i in $(seq 1 64); do
    for d in ospfd zebra; do
      f=/var/run/frr/${PREFIXE}$i/$d.pid
      [ -f "$f" ] && kill "$(cat "$f")" 2>/dev/null || true
    done
  done
  sleep 0.5
  for ns in $(ip netns list | awk '{print $1}' | grep "^${PREFIXE}" || true); do
    ip netns del "$ns"
  done
  rm -rf /etc/frr/${PREFIXE}* /var/run/frr/${PREFIXE}*
}
trap nettoyer EXIT
nettoyer

attendre() {
  local limite_s=$1; shift
  local debut; debut=$(maintenant_ms)
  until "$@" > /dev/null 2>&1; do
    if [ $(( $(maintenant_ms) - debut )) -gt $(( limite_s * 1000 )) ]; then
      echo "-1"; return 0
    fi
    sleep 0.2
  done
  echo $(( $(maintenant_ms) - debut ))
}

for i in $(seq 1 "$R"); do
  ip netns add ${PREFIXE}$i
  ip -n ${PREFIXE}$i link set lo up
  ip -n ${PREFIXE}$i addr add 10.255.0.$i/32 dev lo
  ip netns exec ${PREFIXE}$i sysctl -qw net.ipv4.ip_forward=1
done
for i in $(seq 1 $((R - 1))); do
  j=$((i + 1))
  ip link add e$j netns ${PREFIXE}$i type veth peer name e$i netns ${PREFIXE}$j
  ip -n ${PREFIXE}$i addr add 10.0.$i.1/30 dev e$j
  ip -n ${PREFIXE}$j addr add 10.0.$i.2/30 dev e$i
  ip -n ${PREFIXE}$i link set e$j up
  ip -n ${PREFIXE}$j link set e$i up
done

debut=$(maintenant_ms)
for i in $(seq 1 "$R"); do
  n=${PREFIXE}$i
  mkdir -p /etc/frr/$n /var/run/frr/$n
  chown -R frr:frr /etc/frr/$n /var/run/frr/$n
  {
    echo "hostname $n"
    for itf in $(ip -n $n -o link show | awk -F': ' '{print $2}' | cut -d@ -f1 | grep '^e'); do
      echo "interface $itf"
      echo " ip ospf network point-to-point"
      echo " ip ospf hello-interval 1"
      echo " ip ospf dead-interval 4"
    done
    echo "router ospf"
    echo " ospf router-id 10.255.0.$i"
    echo " network 10.0.0.0/8 area 0"
  } > /etc/frr/$n/frr.conf
  cp /etc/frr/$n/frr.conf /etc/frr/$n/zebra.conf
  cp /etc/frr/$n/frr.conf /etc/frr/$n/ospfd.conf
  chown frr:frr /etc/frr/$n/*.conf
  ip netns exec $n $FRR/zebra -d -N $n -f /etc/frr/$n/zebra.conf
  ip netns exec $n $FRR/ospfd -d -N $n -f /etc/frr/$n/ospfd.conf
done
lance_ms=$(( $(maintenant_ms) - debut ))

route_vers_dernier() { ip -n ${PREFIXE}1 route get 10.255.0.$R 2>/dev/null | grep -q " via "; }
ping_dernier() { ip netns exec ${PREFIXE}1 ping -c 1 -W 1 10.255.0.$R; }

converge_ms=$(attendre 60 ping_dernier)

rss_ko=0
for i in $(seq 1 "$R"); do
  for d in zebra ospfd; do
    pid=$(cat /var/run/frr/${PREFIXE}$i/$d.pid)
    rss_ko=$(( rss_ko + $(awk '/VmRSS/ {print $2}' /proc/$pid/status) ))
  done
done

milieu=$(( (R + 1) / 2 ))
ip -n ${PREFIXE}$milieu link set e$((milieu + 1)) down
plus_de_route() { ! route_vers_dernier; }
panne_vue_ms=$(attendre 30 plus_de_route)

ip -n ${PREFIXE}$milieu link set e$((milieu + 1)) up
repare_ms=$(attendre 60 ping_dernier)

echo "routeurs                         : $R"
echo "lancement de FRR                 : ${lance_ms} ms"
echo "routes arrivées, ping de bout    : ${converge_ms} ms"
echo "RAM (zebra + ospfd)              : $(( rss_ko / 1024 )) Mo, soit $(( rss_ko / R / 1024 )) Mo par routeur"
echo "câble coupé, route perdue en     : ${panne_vue_ms} ms"
echo "câble remis, ping revenu en      : ${repare_ms} ms"
