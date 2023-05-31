#!/bin/sh

killall dhcpcd
ip addr flush dev eth0
ip addr add REALM_IP/24 dev eth0
ip route add default via FVP_TAP_IP
echo "done" >/shared/realm_net_done.txt
