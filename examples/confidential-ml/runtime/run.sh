#!/bin/sh

rm -f checkpoint/model.ckpt

MODEL_TYPE=$1
IS_FL=$2
MEASUREMENT_FILE="example_app.measurement"
if [ "$3" ]; then
  MEASUREMENT_FILE=$3
fi
GUI_SERVER_PORT=-1
if [ "$4" ]; then
  GUI_SERVER_PORT=$4
fi

./runtime.exe --print_all=true --operation=run-runtime-server --data_dir=./data/ \
      --policy_store_file=policy_store --runtime_host="0.0.0.0" \
      --measurement_file=${MEASUREMENT_FILE} --gui_server_port=${GUI_SERVER_PORT} \
      --model_type="${MODEL_TYPE}" --is_fl=${IS_FL}
