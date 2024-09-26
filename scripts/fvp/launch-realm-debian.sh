#!/bin/sh

cd /shared
echo "start copying debian12.img into the rootfs.."
cp -f /shared/debian12.img /
echo "copy done"

./configure-net.sh &
./lkvm run \
	--realm \
	--measurement-algo="sha256" \
	--disable-sve \
	--console serial \
	--irqchip=gicv3 \
	--network mode=tap tapif=tap0 \
	--9p /shared,FMR \
	-m 1024M \
	-c 1 \
	-k linux.realm \
	-d /debian12.img \
	-p "earlycon=ttyS0 printk.devkmsg=on root=/dev/vda"
