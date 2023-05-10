#!/bin/sh

export LD_LIBRARY_PATH=/lib

rm -rf data
mkdir data
cp -f ../certifier-data/* data/

date -s "2023-09-18 00:00:00"  # [hack to skip time check]

ln -s /shared/examples/lib/libtensorflowlite.so /lib/libtensorflowlite.so
ln -s /shared/examples/lib/libtensorflowlite_flex.so /lib/libtensorflowlite_flex.so

HOST=$1
./device.exe --print_all=true \
      --operation=cold-init --data_dir=./data/ --measurement_file="example_app.measurement" \
      --policy_store_file=policy_store --policy_host="${HOST}"

./device.exe --print_all=true \
      --operation=get-certifier --data_dir=./data/ --measurement_file="example_app.measurement" \
      --policy_store_file=policy_store --policy_host="${HOST}"

