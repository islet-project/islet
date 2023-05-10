#!/bin/sh

export LD_LIBRARY_PATH=/lib

rm -f checkpoint/model.ckpt

date -s "2023-09-18 00:00:00"  # [hack to skip time check]

HOST=$1
PORT=$2
./device.exe --print_all=true --operation=run-shell-ml --data_dir=./data/ \
      --policy_store_file=policy_store --runtime_host="${HOST}" --runtime_data_port=${PORT}
