#!/bin/bash

set -e

# Control these variables
EXPECTED=11
TIMEOUT=1000

ROOT=$(git rev-parse --show-toplevel)
UART=$ROOT/out/uart2.log

[ -e $UART ] && rm $UART

$ROOT/scripts/fvp-cca -bo -nw=acs
$ROOT/scripts/fvp-cca -ro -nw=acs --no-telnet &

sleep 30
elapsed=30

while ! grep -q "REGRESSION REPORT:" "$UART"; do
	sleep 5
	elapsed=$((elapsed + 5))

	if [ ${elapsed} -gt ${TIMEOUT} ]; then
		echo "[!] Timeout occured."
		break
	fi

	echo -n "."
done

# Cleanup
ps -ef | grep fvp-cca | grep -v grep | awk '{print $2}' | xargs kill
ps -ef | grep "FVP terminal" | grep -v grep | awk '{print $2}' | xargs kill
ps -ef | grep FVP_Base_RevC-2xAEMvA | grep -v grep | awk '{print $2}' | xargs kill

tail -11 $UART
passed=$(tail -11 $UART | grep "TOTAL PASSED" | awk '{print $5}')

if [ "$EXPECTED" -eq "$passed" ]; then
	echo "[!] Test succeeded!"
else
	echo "[!] Test failed!"
	exit 1
fi
