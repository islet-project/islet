#!/bin/sh

HOST_IP=$1
MODEL=$2

./model_provider.exe --print_all=true --operation=send-model --data_dir=./data/ --measurement_file="example_app.measurement" \
      --policy_store_file=policy_store --model_file=${MODEL} --runtime_host=${HOST_IP}
