#!/bin/sh

set -e

cd /shared
./certifier-client \
	--data_dir=./client/ \
	--operation=cold-init \
	--measurement_file="client.measurement" \
	--policy_store_file=policy_store \
	--policy_host=193.168.10.15 \
	--server_app_host=193.168.10.15 \
	--print_all=true

./certifier-client \
	--data_dir=./client/ \
	--operation=get-certified \
	--measurement_file="client.measurement" \
	--policy_store_file=policy_store \
	--policy_store_file=policy_store \
	--policy_host=193.168.10.15 \
	--server_app_host=193.168.10.15 \
	--print_all=true

./certifier-client \
	--data_dir=./client/ \
	--operation=run-app-as-client \
	--policy_store_file=policy_store \
	--policy_host=193.168.10.15 \
	--server_app_host=193.168.10.15 \
	--print_all=true
