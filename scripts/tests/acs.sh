#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
REPORT=$ROOT/third-party/cca-rmm-acs/build/output/regression_report.log

$ROOT/scripts/fvp-cca -bo -nw=acs -rmm=tf-rmm
$ROOT/scripts/fvp-cca -ro -nw=acs -rmm=tf-rmm &

flag=0
done=1
echo -n "Running ACS test"

sleep 10

while ! grep -q "REGRESSION REPORT:" "$REPORT"; do
	sleep 10
	echo -n "."
done

tail -11 $REPORT

# cleanup
ps -ef | grep fvp-cca | grep -v grep | awk '{print $2}' | xargs kill
ps -ef | grep "FVP terminal" | grep -v grep | awk '{print $2}' | xargs kill

cp $REPORT $ROOT/out
