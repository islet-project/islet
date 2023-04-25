#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)
UART=$ROOT/out/uart0.log

[ -e $UART ] && rm $UART

check_result()
{
	# cleanup
	ps -ef | grep fvp-cca | grep -v grep | awk '{print $2}' | xargs kill
	ps -ef | grep "FVP terminal" | grep -v grep | awk '{print $2}' | xargs kill
	ps -ef | grep FVP_Base_RevC-2xAEMvA | grep -v grep | awk '{print $2}' | xargs kill

	# report
	PASSED=$(tail -10 $UART | grep "Tests Passed" | awk '{print $4}')
	FAILED=$(tail -10 $UART | grep "Tests Failed" | awk '{print $4}')
	PASSED="${PASSED//[$'\t\r\n ']/}"
	FAILED="${FAILED//[$'\t\r\n ']/}"

	if [ "$PASSED" == "" ]; then
		echo "[-] Test failed! (There are no proper result logs)"
		exit 1
	fi

	echo "[!] Tests result: $PASSED passed, $FAILED failed."

	if [ $FAILED -ne 0 ]; then
		tail -10 $UART
		exit $FAILED
	fi
}

# tf-rmm tests
$ROOT/scripts/fvp-cca -bo -nw=tf-a-tests -rmm=tf-rmm
$ROOT/scripts/fvp-cca -ro -nw=tf-a-tests -rmm=tf-rmm &

sleep 20

check_result

# tf-rmm tests with rsi-test realm
$ROOT/scripts/fvp-cca -bo -nw=tf-a-tests -rmm=tf-rmm -rm=rsi-test
$ROOT/scripts/fvp-cca -ro -nw=tf-a-tests -rmm=tf-rmm -rm=rsi-test &

sleep 20

check_result

# islet-rmm tests
$ROOT/scripts/fvp-cca -bo -nw=tf-a-tests -rmm=islet
$ROOT/scripts/fvp-cca -ro -nw=tf-a-tests -rmm=islet &

sleep 20

check_result

# islet-rmm tests with rsi-test realm
$ROOT/scripts/fvp-cca -bo -nw=tf-a-tests -rmm=islet -rm=rsi-test
$ROOT/scripts/fvp-cca -ro -nw=tf-a-tests -rmm=islet -rm=rsi-test &

sleep 20

check_result
