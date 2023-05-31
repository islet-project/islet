#!/bin/sh

rm -f checkpoint/model.ckpt

HOST_IP=$1
MODEL_TYPE=$2
IS_FL=$3
MEASUREMENT_FILE="example_app.measurement"
if [ "$4" ]; then
  MEASUREMENT_FILE=$4
fi
GUI_SERVER_PORT=-1
if [ "$5" ]; then
  GUI_SERVER_PORT=$5
fi
IS_MALICIOUS=0
if [ "$6" ]; then
  IS_MALICIOUS=$6
fi

./runtime.exe --print_all=true --operation=run-runtime-server --data_dir=./data/ \
      --policy_store_file=policy_store --runtime_host="${HOST_IP}" \
      --measurement_file=${MEASUREMENT_FILE} --gui_server_port=${GUI_SERVER_PORT} \
      --model_type="${MODEL_TYPE}" --is_fl=${IS_FL} --is_malicious=${IS_MALICIOUS}