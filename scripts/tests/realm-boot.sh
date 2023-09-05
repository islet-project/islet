#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)
UART=$ROOT/out/uart0.log

[ -e $UART ] && rm $UART

check_result()
{
	# cleanup
	ps -ef | grep fvp-cca | grep -v grep | awk '{print $2}' | xargs kill
	ps -ef | grep FVP_Base_RevC-2xAEMvA | grep -v grep | awk '{print $2}' | xargs kill

	# report
	LOGIN=$(tail -30 $UART | grep "login" | awk '{print $2}')
	LOGIN="${LOGIN//[$'\t\r\n ']/}"

	if [ "$LOGIN" == "" ]; then
		echo "[-] Test result: Realm booting failed with the following log"
		tail -30 $UART
		echo "Try increasing the time for sleep"
		exit 1
	fi

	echo "[!] Tests result: Realm booting succeeded"
}

$ROOT/scripts/fvp-cca -bo -nw=linux --realm=linux -rmm=islet --realm-launch
$ROOT/scripts/fvp-cca -ro -nw=linux --realm=linux -rmm=islet --realm-launch &

sleep 480

check_result
