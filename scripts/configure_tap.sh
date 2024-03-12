#!/bin/bash

# input arguments
host_ip=$1
fvp_ip=$2
route_ip=$3
gateway=$4
ifname=$5

# 1. check if armbr0 is already configured
out=$(brctl show | grep armbr0)
user=$(whoami)
if [[ $out == *"armbr0"* ]]; then
	if [[ $out == *"${user}"* ]]; then
		echo "tap network already configured!"
		exit 0
	fi
fi

# 2. create a bridge network
sudo ip link add armbr0 type bridge
sudo ip link set armbr0 up

# 3. reassign IP address to the bridge
sudo ip link set ${ifname} up
sudo ip link set ${ifname} master armbr0

# Drop existing IP from eth0
sudo ip addr flush dev ${ifname}

# Assign IP to armbr0
sudo ip addr add ${host_ip}/24 brd + dev armbr0

# 4. create a tap device
sudo ip tuntap add dev ARM${user} mode tap user ${user}
sudo ip link set dev ARM${user} up
sudo ip link set ARM${user} master armbr0
sudo ip route add ${route_ip}/24 via ${fvp_ip}

# 5. add NAT functionality to properly interact with remote hosts
#echo 1 | sudo tee /proc/sys/net/ipv4/ip_forward
#sudo iptables -t nat -A POSTROUTING -j MASQUERADE
