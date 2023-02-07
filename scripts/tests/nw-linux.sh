#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)
UART=$ROOT/out/uart0.log

[ -e $UART ] && rm $UART

$ROOT/scripts/fvp-cca -bo -nw=linux
echo "[bp] build done."
$ROOT/scripts/fvp-cca -ro -nw=linux &

sleep 10
echo "[bp] sleep 10 done."

prev=0
curr=1

while [ "$prev" -ne "$curr" ]; do
	prev=$curr

#	tail -10 $UART
	echo "[bp] before sleep 10 $(date)"
	sleep 30
	echo "[bp] after sleep 10 $(date)"

#	tail -10 $UART
	curr=$(wc -l $UART | awk '{print $1}')

	echo "[bp] prev = $prev, curr = $curr"
done

cat $UART

# cleanup
ps -ef | grep fvp-cca | grep -v grep | awk '{print $2}' | xargs kill
ps -ef | grep "FVP terminal" | grep -v grep | awk '{print $2}' | xargs kill
ps -ef | grep FVP_Base_RevC-2xAEMvA | grep -v grep | awk '{print $2}' | xargs kill

if ! grep -q "Welcome to islet (normal world linux)!" "$UART"; then
	echo "[-] Test failed! (There are no proper result logs)"
	exit 1
fi

echo "[!] Tests success!"
