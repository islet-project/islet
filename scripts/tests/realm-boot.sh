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

tar -xf $ROOT/assets/prebuilt/out.tar.bz2 -C $ROOT
$ROOT/scripts/fvp-cca -bo -rmm=islet --use-prebuilt --rmm-log-level=error
$ROOT/scripts/fvp-cca -ro -nw=linux --realm=linux -rmm=islet --no-telnet &

sleep 640

check_result
