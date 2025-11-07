#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
CERTIFIER=$ROOT/third-party/certifier
HERE=$ROOT/examples/confidential-ml/runtime
PROVISION_DIR=$HERE/../certifier-data/runtime
MEASUREMENT_FILE=runtime.measurement
HOST_IP=192.168.10.1

rm -f checkpoint/model.ckpt

export LD_LIBRARY_PATH=$HERE/../tflite_libs/:$LD_LIBRARY_PATH

./runtime.exe --print_all=true \
      --operation=cold-init --data_dir=$PROVISION_DIR/ --measurement_file=${MEASUREMENT_FILE} \
      --policy_store_file=policy_store --runtime_host=$HOST_IP --policy_host=$HOST_IP --server_app_host=$HOST_IP

./runtime.exe --print_all=true \
      --operation=get-certifier --data_dir=$PROVISION_DIR/ --measurement_file=${MEASUREMENT_FILE} \
      --policy_store_file=policy_store --runtime_host=$HOST_IP --policy_host=$HOST_IP --server_app_host=$HOST_IP

./runtime.exe \
      --print_all=true --operation=run-runtime-server --data_dir=$PROVISION_DIR/ \
      --policy_store_file=policy_store --runtime_host=$HOST_IP --policy_host=$HOST_IP --server_app_host=$HOST_IP \
      --measurement_file=${MEASUREMENT_FILE} --gui_server_port=-1 \
      --model_type=code --is_fl=0 --is_malicious=-1
