#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
CERTIFIER=$ROOT/third-party/certifier
HERE=$ROOT/examples/confidential-ml/model-provider
PROVISION_DIR=$HERE/../certifier-data/model-provider
MEASUREMENT_FILE=model_provider.measurement
HOST_IP=192.168.10.1

./model_provider.exe --print_all=true \
      --operation=cold-init --data_dir=$PROVISION_DIR/ --measurement_file=${MEASUREMENT_FILE} \
      --policy_store_file=policy_store --runtime_host=$HOST_IP --policy_host=$HOST_IP

./model_provider.exe --print_all=true \
      --operation=get-certifier --data_dir=$PROVISION_DIR/ --measurement_file=${MEASUREMENT_FILE} \
      --policy_store_file=policy_store --runtime_host=$HOST_IP --policy_host=$HOST_IP

./model_provider.exe --print_all=true --operation=send-model --data_dir=$PROVISION_DIR/ --measurement_file=${MEASUREMENT_FILE} \
      --policy_store_file=policy_store --model_file=model_code.tflite --runtime_host=${HOST_IP}
