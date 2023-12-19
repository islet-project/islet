#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
CERTIFIER=$ROOT/third-party/certifier
EXAMPLE_DIR=$CERTIFIER/sample_apps/simple_app_under_islet

cd $EXAMPLE_DIR
$EXAMPLE_DIR/certifier-server \
	--data_dir=./server/ \
	--operation=cold-init \
	--measurement_file="server.measurement" \
	--policy_store_file=policy_store \
	--policy_host=193.168.10.15 --server_app_host=193.168.10.15 \
	--print_all=true

$EXAMPLE_DIR/certifier-server \
	--data_dir=./server/ \
	--operation=get-certified \
	--measurement_file="server.measurement" \
	--policy_store_file=policy_store \
	--policy_host=193.168.10.15 --server_app_host=193.168.10.15 \
	--print_all=true

$EXAMPLE_DIR/certifier-server \
	--data_dir=./server/ \
	--operation=run-app-as-server \
	--policy_store_file=policy_store \
	--policy_host=193.168.10.15 \
	--server_app_host=193.168.10.15 \
	--print_all=true
