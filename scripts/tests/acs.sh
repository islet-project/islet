#!/bin/bash

set -e

# Control these variables
EXPECTED=71
TIMEOUT=30

ROOT=$(git rev-parse --show-toplevel)
UART=$ROOT/out/uart2.log

[ -e $UART ] && rm $UART

$ROOT/scripts/fvp-cca -bo -nw=acs --rmm-log-level=error --excluded-tests=skipped-tests.txt
$ROOT/scripts/fvp-cca -ro -nw=acs --no-telnet &

sleep 10

echo "[!] Starting ACS test..."

while inotifywait -q -t $TIMEOUT -e modify $UART >/dev/null 2>&1; do
	echo -n "."
	if grep -q "REGRESSION REPORT:" "$UART"; then
		break
	fi
done

echo ""

# Cleanup
ps -ef | grep fvp-cca | grep -v grep | awk '{print $2}' | xargs kill
ps -ef | grep FVP_Base_RevC-2xAEMvA | grep -v grep | awk '{print $2}' | xargs kill

tail -11 $UART
passed=$(tail -11 $UART | grep "TOTAL PASSED" | awk '{print $5}')

if [ "$passed" -ge "$EXPECTED" ]; then
	echo "[!] Test succeeded!"
else
	echo "[!] Test failed!"
	exit 1
fi
