#!/bin/sh

rm -f checkpoint/model.ckpt

./runtime.exe --print_all=true --operation=run-runtime-fl-server --data_dir=./data/ \
      --policy_store_file=policy_store --runtime_host="0.0.0.0"
