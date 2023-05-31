#!/bin/sh

# 1. enable packet forwarding
echo 1 >/proc/sys/net/ipv4/ip_forward

# 2. wait for Realm's ip configuration
rm -f /shared/realm_net_done.txt
while [ 1 ]; do
	if [ -f /shared/realm_net_done.txt ]; then
		break
	fi
	sleep 1
done

# 3. configure tap interface
ip addr flush tap0
ip addr add FVP_TAP_IP/24 dev tap0
