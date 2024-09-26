#!/bin/sh

# 1. enable packet forwarding
echo 1 >/proc/sys/net/ipv4/ip_forward

# 2. wait for tap0 to be shown up
while [ 1 ]; do
	ifout="$(ifconfig)"
	if [ $(echo ${ifout} | grep -c "tap0") -gt 0 ]; then
		break
	fi
	sleep 1
done

# 3. configure tap interface
ip addr flush tap0
ip addr add FVP_TAP_IP/24 dev tap0

# 4. set SNAT for packet forwarding between tap0 and eth0
/shared/iptables -A FORWARD -i eth0 -j ACCEPT
/shared/iptables -A FORWARD -o eth0 -j ACCEPT
/shared/iptables -t nat -A POSTROUTING -s FVP_TAP_IP/24 -o eth0 -j SNAT --to FVP_IP
