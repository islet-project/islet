#!/bin/bash

# input arguments
host_ip=$1
fvp_ip=$2
route_ip=$3

# 1. check if tap is already configured
out=$(ifconfig | grep ARM)
user=$(whoami)
if [[ $out == *"ARM"* ]]; then
	if [[ $out == *"${user}"* ]]; then
		echo "tap network already configured!"
		exit 0
	fi
fi

# create a tap device
sudo ip tuntap add dev ARM${user} mode tap user ${user}
sudo ip addr add ${host_ip}/24 dev ARM${user}
sudo ip link set dev ARM${user} up
sudo ip route add ${route_ip}/24 via ${fvp_ip}
