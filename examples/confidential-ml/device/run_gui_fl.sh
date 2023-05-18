#!/bin/sh

rm -f checkpoint/model.ckpt

HOST=$1
PORT=$2
GUI_RX_PORT=$3
GUI_TX_PORT=$4
./device.exe --print_all=true --operation=run-gui-fl --data_dir=./data/ \
      --policy_store_file=policy_store --runtime_host="${HOST}" --runtime_data_port=${PORT} --gui_rx_port=${GUI_RX_PORT} --gui_tx_port=${GUI_TX_PORT}
