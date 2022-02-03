#!/bin/bash
ROOT=$(git rev-parse --show-toplevel)
SCRIPT_NAME=$0

function fn_make_test_badge() {
	if [ "${2}" -eq "0" ]; then
		RESULT="success"
	else
		RESULT="critical"
	fi

	curl -s -o out/test.svg "https://img.shields.io/badge/tests-${1}%20passed,%20${2}%20failed-${RESULT}.svg"
}

function fn_get_junit_result() {
	cargo install cargo2junit

	cd ${ROOT}
	mkdir out
	cargo test --lib --target x86_64-unknown-linux-gnu -- --test-threads=1 \
		-Z unstable-options --format json > out/test.json
	cat out/test.json | cargo2junit > out/test.xml

	PASSED=`jq -r "select(.type == \"suite\" and .event != \"started\") | .passed" out/test.json`
	FAILED=`jq -r "select(.type == \"suite\" and .event != \"started\") | .failed" out/test.json`

	fn_make_test_badge ${PASSED} ${FAILED}

	echo "${PASSED} passed, ${FAILED} failed."
	exit ${FAILED}
}

function fn_measure_coverage() {
	echo "Not implemented yet"
}

function fn_usage() {
	echo "./${SCRIPT_NAME} [OPTIONS]"
cat <<EOF
no option:
	Just unit-test and print the results
options:
	--junit		Get test results as a JUnit xml file to out/test-result.xml
	--coverage	Measure coverage tests and get results in out/coverage
EOF
}

function fn_test() {
	cd ${ROOT}
	cargo test --lib --target x86_64-unknown-linux-gnu -- --test-threads=1
}

if [ $# -lt 1 ]; then
	fn_test
fi

while [ $# -gt 0 ]; do

	case "$1" in
		--junit) fn_get_junit_result
			;;
		--coverage) fn_measure_coverage
			;;
		*) fn_usage; exit
			;;
	esac
	shift
done


