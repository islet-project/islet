#!/bin/sh

rm -f checkpoint/model.ckpt

GUI_SERVER_PORT=-1
if [ "$1" ]; then
  GUI_SERVER_PORT=$1
fi

./runtime.exe --print_all=true --operation=run-runtime-ml-server --data_dir=./data/ \
      --policy_store_file=policy_store --runtime_host="0.0.0.0" --gui_server_port=${GUI_SERVER_PORT}
