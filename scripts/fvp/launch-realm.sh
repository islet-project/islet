#!/bin/sh

cd /shared

if [ $# -gt 0 ]; then
	case "$1" in
		net)
			echo "Running network configuration script"
			./configure-net.sh &
			shift
			;;
	esac
fi

./lkvm run \
	--debug \
	--restricted_mem \
	--realm \
	--measurement-algo="sha256" \
	--disable-sve \
	--console serial \
	--irqchip=gicv3 \
	--network virtio \
	--9p /shared,FMR \
	-m 256M \
	-c 1 \
	-k linux.realm \
	-i rootfs-realm.cpio.gz \
	-p "earlycon=ttyS0 printk.devkmsg=on" \
	"$@"
