#!/bin/sh

rm -rf data
mkdir -p data
cp -f ../certifier-data/* data/

HOST="0.0.0.0"
if [ "$1" ]; then
  HOST=$1
fi

./model_provider.exe --print_all=true \
      --operation=cold-init --data_dir=./data/ --measurement_file="example_app.measurement" \
      --policy_store_file=policy_store --runtime_host="${HOST}" --policy_host="${HOST}"

./model_provider.exe --print_all=true \
      --operation=get-certifier --data_dir=./data/ --measurement_file="example_app.measurement" \
      --policy_store_file=policy_store --runtime_host="${HOST}" --policy_host="${HOST}"