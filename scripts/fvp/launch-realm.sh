#!/bin/sh

cd /shared

if [ $# -gt 0 ]; then
	case "$1" in
		net)
			./configure-net.sh &
			;;
	esac
fi

#./eventfd_manager
#sleep 1

./lkvm run \
	--debug \
	--realm \
	--measurement-algo="sha256" \
	--disable-sve \
	--console serial \
	--irqchip=gicv3 \
	--network virtio \
	--force-pci \
	--9p /shared,FMR \
	--socket-path /tmp/eventfd_manager_socket \
	--shm-id 1 \
	-m 256M \
	-c 1 \
	-k linux.realm \
	-i rootfs-realm.cpio.gz \
	-p "earlycon=ttyS0 printk.devkmsg=on"
