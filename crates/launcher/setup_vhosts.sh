#!/bin/bash
NS_SERVEUR="server"
NS_BATEAU="boat"
BRIDGE="br0"

# 1. Nettoyage agressif
# On supprime les namespaces (cela détruit normalement les veth à l'intérieur)
sudo ip netns del $NS_SERVEUR 2>/dev/null
sudo ip netns del $NS_BATEAU 2>/dev/null

# On supprime explicitement les interfaces veth coté "host" au cas où
sudo ip link del veth-serv-br 2>/dev/null
sudo ip link del veth-bat-br 2>/dev/null

# On supprime le bridge
sudo ip link del $BRIDGE 2>/dev/null

# Petit temps de pause pour laisser le noyau nettoyer les tables réseau
sleep 0.5

# 2. Création du Bridge
sudo ip link add $BRIDGE type bridge || exit 1
sudo ip addr add 10.0.0.1/24 dev $BRIDGE
sudo ip link set $BRIDGE up

# 3. Configuration SERVEUR
sudo ip netns add $NS_SERVEUR || exit 1
# On crée la paire veth
sudo ip link add veth-serv type veth peer name veth-serv-br || exit 1
# On déplace un côté dans le namespace
sudo ip link set veth-serv netns $NS_SERVEUR
# On attache l'autre côté au bridge
sudo ip link set veth-serv-br master $BRIDGE
# Setup IP et UP
sudo ip netns exec $NS_SERVEUR ip addr add 10.0.0.2/24 dev veth-serv
sudo ip netns exec $NS_SERVEUR ip link set veth-serv up
sudo ip netns exec $NS_SERVEUR ip link set lo up
sudo ip link set veth-serv-br up

# 4. Configuration BATEAU
sudo ip netns add $NS_BATEAU || exit 1
sudo ip link add veth-bat type veth peer name veth-bat-br || exit 1
sudo ip link set veth-bat netns $NS_BATEAU
sudo ip link set veth-bat-br master $BRIDGE
sudo ip netns exec $NS_BATEAU ip addr add 10.0.0.3/24 dev veth-bat
sudo ip netns exec $NS_BATEAU ip link set veth-bat up
sudo ip netns exec $NS_BATEAU ip link set lo up
sudo ip link set veth-bat-br up

echo "Configuration des vhosts effectuée avec succès (serveur: 10.0.0.2, capitainerie: 10.0.0.1, bateau: 10.0.0.3)."