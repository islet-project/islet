#!/bin/bash

# input arguments
host_ip=$1
host_tap_ip=$2
fvp_ip=$3
route_ip=$4
gateway=$5
ifname=$6

# 0. check if the tap is already configured
out=$(ifconfig | grep ARM)
user=$(whoami)
if [[ $out == *"ARM"* ]]; then
	if [[ $out == *"${user}"* ]]; then
		echo "tap network already configured!"
		exit 0
	fi
fi

# 1. create a tap device that is connected to FVP
sudo ip tuntap add dev ARM${user} mode tap
sudo ip addr add ${host_tap_ip}/24 dev ARM${user}
sudo ip link set ARM${user} up promisc on

# 2. enable ip forwarding
sudo sysctl net.ipv4.ip_forward=1
sudo sysctl net.ipv6.conf.default.forwarding=1
sudo sysctl net.ipv6.conf.all.forwarding=1

# 3. set SNAT to allow FVP/Realm to connect internet through the host's network interface
sudo iptables -A FORWARD -i ${ifname} -j ACCEPT
sudo iptables -A FORWARD -o ${ifname} -j ACCEPT
sudo iptables -t nat -A POSTROUTING -s ${host_tap_ip}/24 -o ${ifname} -j SNAT --to ${host_ip}
