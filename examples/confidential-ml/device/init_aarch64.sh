#!/bin/sh

rm -rf data
mkdir data
cp -f ../certifier-data/* data/

HOST="0.0.0.0"
if [ "$1" ]; then
  HOST=$1
fi

date -s "2023-09-18 00:00:00" # [hack to skip time check]
ln -s /shared/examples/lib/libtensorflowlite.so /lib/libtensorflowlite.so
ln -s /shared/examples/lib/libtensorflowlite_flex.so /lib/libtensorflowlite_flex.so

./device.exe --print_all=true \
      --operation=cold-init --data_dir=./data/ --measurement_file="example_app.measurement" \
      --policy_store_file=policy_store --policy_host="${HOST}"

./device.exe --print_all=true \
      --operation=get-certifier --data_dir=./data/ --measurement_file="example_app.measurement" \
      --policy_store_file=policy_store --policy_host="${HOST}"

