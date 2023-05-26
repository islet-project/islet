#!/bin/sh

MODEL=$1

./model_provider.exe --print_all=true --operation=send-model --data_dir=./data/ --measurement_file="example_app.measurement" \
      --policy_store_file=policy_store --model_file=${MODEL}
