#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)
UART=$ROOT/out/uart0.log

[ -e $UART ] && rm $UART

MAX_TRIAL=10
SLEEP_TIME=60

check_result()
{

	trial=0
	while [ $trial -lt $MAX_TRIAL ]; do
		# report
		PASSED=$(tail -10 $UART | grep "Tests Passed" | awk '{print $4}')
		FAILED=$(tail -10 $UART | grep "Tests Failed" | awk '{print $4}')
		PASSED="${PASSED//[$'\t\r\n ']/}"
		FAILED="${FAILED//[$'\t\r\n ']/}"

		if [ "$PASSED" == "" ]; then
			echo "[$((trial * SLEEP_TIME + SLEEP_TIME))secs] Waiting for test results..."
			sleep $SLEEP_TIME
			trial=$((trial + 1))
			continue
		else
			break
		fi
	done

	# cleanup
	ps -ef | grep fvp-cca | grep -v grep | awk '{print $2}' | xargs kill
	ps -ef | grep FVP_Base_RevC-2xAEMvA | grep -v grep | awk '{print $2}' | xargs kill

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
#$ROOT/scripts/fvp-cca -bo -nw=tf-a-tests -rmm=tf-rmm
#$ROOT/scripts/fvp-cca -ro -nw=tf-a-tests -rmm=tf-rmm &

#sleep 20

#check_result

# tf-rmm tests with rsi-test realm
#$ROOT/scripts/fvp-cca -bo -nw=tf-a-tests -rmm=tf-rmm -rm=rsi-test
#$ROOT/scripts/fvp-cca -ro -nw=tf-a-tests -rmm=tf-rmm -rm=rsi-test &

#sleep 20

#check_result

# islet-rmm tests
$ROOT/scripts/fvp-cca -bo -nw=tf-a-tests -rmm=islet --rmm-log-level=error
$ROOT/scripts/fvp-cca -ro -nw=tf-a-tests -rmm=islet --no-telnet &

sleep $SLEEP_TIME

check_result

# islet-rmm tests with rsi-test realm
#$ROOT/scripts/fvp-cca -bo -nw=tf-a-tests -rmm=islet -rm=rsi-test --rmm-log-level=error
#$ROOT/scripts/fvp-cca -ro -nw=tf-a-tests -rmm=islet -rm=rsi-test &

#sleep 20

#check_result
