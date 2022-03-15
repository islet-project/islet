#!/bin/bash
ROOT=$(git rev-parse --show-toplevel)
SCRIPT_NAME=$0

function fn_make_test_badge()
{
	if [ "${2}" -eq "0" ]; then
		RESULT="success"
	else
		RESULT="critical"
	fi

	curl -s -o out/test.svg "https://img.shields.io/badge/tests-${1}%20passed,%20${2}%20failed-${RESULT}.svg"
}

function fn_unit_test
{
	cargo install cargo2junit

	cd ${ROOT}/rmm/core
	cargo test --lib -- --test-threads=1 \
		-Z unstable-options --format json >${ROOT}/out/test.json

	cd ${ROOT}
	cat out/test.json | cargo2junit >out/test.xml

	PASSED=$(jq -r "select(.type == \"suite\" and .event != \"started\") | .passed" out/test.json)
	FAILED=$(jq -r "select(.type == \"suite\" and .event != \"started\") | .failed" out/test.json)

	fn_make_test_badge ${PASSED} ${FAILED}

	echo "${PASSED} passed, ${FAILED} failed."
	exit ${FAILED}
}

function fn_make_coverage_badge()
{
	if [ "${1}" -lt "60" ]; then
		RESULT="orange"
	elif [ "${1}" -lt "80" ]; then
		RESULT="yellow"
	else
		RESULT="brightgreen"
	fi

	curl -s -o out/coverage.svg "https://img.shields.io/badge/coverage-${1}-${RESULT}.svg"
}

function fn_measure_coverage()
{
	cargo install cargo-tarpaulin --version 0.18.5

	cd ${ROOT}/rmm/core
	cargo tarpaulin --lib --exclude-files bin/* -v --ignore-tests --out Lcov --output-dir ${ROOT}/out \
		-- --test-threads=1

	cd ${ROOT}

	genhtml --output-directory out/coverage --show-details --highlight \
		--ignore-errors source --legend out/lcov.info

	COVERAGE=$(grep "headerCovTableEntry[A-Za-z]" out/coverage/index.html | cut -d ">" -f2 | cut -d "%" -f1 | cut -d "." -f1)

	mv out/lcov.info out/coverage/.

	fn_make_coverage_badge $COVERAGE
}

function fn_usage()
{
	echo "./${SCRIPT_NAME} [OPTIONS]"
	cat <<EOF
no option:
    Do unit-test and print the results
options:
    --unit-test  Get test results as a JUnit xml file to out/test-result.xml
    --coverage   Measure coverage tests and get results in out/coverage
EOF
}

if [ $# -lt 1 ]; then
	fn_usage
fi

while [ $# -gt 0 ]; do

	case "$1" in
		--unit-test)
			fn_unit_test
			;;
		--coverage)
			fn_measure_coverage
			;;
		*)
			fn_usage
			exit
			;;
	esac
	shift
done
