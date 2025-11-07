#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
CERTIFIER=$ROOT/third-party/certifier
HERE=$ROOT/examples/confidential-ml/device
PROVISION_DIR=$HERE/../certifier-data/device
MEASUREMENT_FILE=device.measurement
HOST_IP=192.168.10.1

rm -f checkpoint/model.ckpt

./device.exe --print_all=true \
      --operation=cold-init --data_dir=$PROVISION_DIR/ --measurement_file=${MEASUREMENT_FILE} \
      --policy_store_file=policy_store --runtime_host=$HOST_IP --runtime_data_port=8125 --policy_host=$HOST_IP \

./device.exe --print_all=true \
      --operation=get-certifier --data_dir=$PROVISION_DIR/ --measurement_file=${MEASUREMENT_FILE} \
      --policy_store_file=policy_store --runtime_host=$HOST_IP --runtime_data_port=8125 --policy_host=$HOST_IP \

./device.exe --print_all=true --operation=run-shell --data_dir=$PROVISION_DIR/ --measurement_file=${MEASUREMENT_FILE} \
      --policy_store_file=policy_store --runtime_host=$HOST_IP --runtime_data_port=8125 --policy_host=$HOST_IP \
      --model_type=code --is_fl=0 \
      --gui_rx_port=-1 --gui_tx_port=-1

