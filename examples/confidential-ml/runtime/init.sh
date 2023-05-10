#!/bin/sh

rm -rf data
mkdir data
cp -f ../certifier-data/* data/

./runtime.exe --print_all=true \
      --operation=cold-init --data_dir=./data/ --measurement_file="example_app.measurement" \
      --policy_store_file=policy_store --runtime_host="0.0.0.0"

./runtime.exe --print_all=true \
      --operation=get-certifier --data_dir=./data/ --measurement_file="example_app.measurement" \
      --policy_store_file=policy_store --runtime_host="0.0.0.0"

