#!/bin/sh

PROVISION_DIR=/shared/device
MEASUREMENT_FILE=device.measurement
HOST_IP=192.168.10.1

#LD_PRELOAD=./libc.so.6:./ld-linux-aarch64.so.1 
./device.exe --print_all=true \
      --operation=cold-init --data_dir=$PROVISION_DIR/ --measurement_file=${MEASUREMENT_FILE} \
      --policy_store_file=policy_store --runtime_host=$HOST_IP --runtime_data_port=8125 --policy_host=$HOST_IP \

#LD_PRELOAD=./libc.so.6:./ld-linux-aarch64.so.1 
./device.exe --print_all=true \
      --operation=get-certifier --data_dir=$PROVISION_DIR/ --measurement_file=${MEASUREMENT_FILE} \
      --policy_store_file=policy_store --runtime_host=$HOST_IP --runtime_data_port=8125 --policy_host=$HOST_IP \

#LD_PRELOAD=./libc.so.6:./ld-linux-aarch64.so.1 
./device.exe --print_all=true --operation=run-shell --data_dir=$PROVISION_DIR/ --measurement_file=${MEASUREMENT_FILE} \
      --policy_store_file=policy_store --runtime_host=$HOST_IP --runtime_data_port=8125 --policy_host=$HOST_IP \
      --model_type=code --is_fl=0 \
      --gui_rx_port=-1 --gui_tx_port=-1

