#!/bin/sh

cd /shared

./lkvm run \
	--debug \
	--realm \
	--measurement-algo="sha256" \
	--disable-sve \
	--console serial \
	--irqchip=gicv3 \
	--realm-pv="gateway" \
	--vcpu-affinity 2-3 \
	-m 256M \
	-c 1 \
	-k acs_realm.bin \
	-i rootfs-realm.cpio.gz \
	-p "earlycon=ttyS0 printk.devkmsg=on"
