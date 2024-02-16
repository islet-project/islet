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
	--expected-measurement="ff06b36491465f9019d3408d2ce301cd937a311aca3ba62b5fd7fa5357334ae2" \
	-m 256M \
	-c 1 \
	-k linux.realm \
	-i rootfs-realm.cpio.gz \
	-p "earlycon=ttyS0 printk.devkmsg=on"
