#!/bin/sh

cd /shared

if [ $# -gt 0 ]; then
	case "$1" in
		net)
			./configure-net.sh &
			;;
	esac
fi

./lkvm run \
	--debug \
	--realm \
	--measurement-algo="sha256" \
	--disable-sve \
	--console serial \
	--irqchip=gicv3 \
	--realm-pv="no_shared_region" \
	-m 256M \
	-c 1 \
	-k linux.realm \
	-i rootfs-realm.cpio.gz \
	-p "earlycon=ttyS0 printk.devkmsg=on"
