#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)
UART=$ROOT/out/uart0.log

[ -e $UART ] && rm $UART

$ROOT/scripts/fvp-cca -bo -nw=tf-a-tests
$ROOT/scripts/fvp-cca -ro -nw=tf-a-tests &

sleep 10

# cleanup
ps -ef | grep fvp-cca | grep -v grep | awk '{print $2}' | xargs kill
ps -ef | grep "FVP terminal" | grep -v grep | awk '{print $2}' | xargs kill
ps -ef | grep FVP_Base_RevC-2xAEMvA | grep -v grep | awk '{print $2}' | xargs kill

# report
PASSED=$(tail -10 $UART | grep "Tests Passed" | awk '{print $4}')
FAILED=$(tail -10 $UART | grep "Tests Failed" | awk '{print $4}')
PASSED="${PASSED//[$'\t\r\n ']/}"
FAILED="${FAILED//[$'\t\r\n ']/}"

echo "[!] Tests result: $PASSED passed, $FAILED failed."

if [ $FAILED -ne 0 ]; then
	tail -10 $UART
	exit $FAILED
fi
