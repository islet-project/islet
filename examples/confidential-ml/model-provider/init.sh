#!/bin/sh

rm -rf data
mkdir -p data
cp -f ../certifier-data/* data/

./model_provider.exe --print_all=true \
      --operation=cold-init --data_dir=./data/ --measurement_file="example_app.measurement" \
      --policy_store_file=policy_store

./model_provider.exe --print_all=true \
      --operation=get-certifier --data_dir=./data/ --measurement_file="example_app.measurement" \
      --policy_store_file=policy_store

