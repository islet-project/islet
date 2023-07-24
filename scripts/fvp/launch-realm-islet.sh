#!/bin/sh

./configure-net.sh &

./lkvm run \
	--debug \
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
	-p "console=pl011,mmio,0x1c0a0000 console=ttyAMA0 printk.devkmsg=on"
