# Unit Test Guidance for Islet RMM

## Overview
Islet RMM adheres to the Arm Realm Management Monitor (RMM) specification, which defines interfaces between the Host and RMM, as well as between the Realm and RMM. The RMM specification is publicly available at [Arm's official documentation](https://developer.arm.com/documentation/den0137/latest/). Islet currently supports version 1.0-REL0 of this specification.

## Test Tools

To ensure compliance with the RMM specification, Islet utilizes unit tests provided by Arm. These tests include the **Architecture Compliance Suite for Realm Management Monitor (ACS test)** and **The Trusted Firmware-A Tests (tf-a-test)**. Any modifications to Islet RMM must pass these tests as part of the Continuous Integration (CI) process, and successful test results are required for PR merges.

### ACS Test
The following content is extracted from the [the ACS Test repository](https://github.com/ARM-software/cca-rmm-acs):  
> The **Architecture Compliance Suite (ACS)** contains a set of functional tests, demonstrating the invariant behaviors that are specified in the architecture specification. It is used to ensure architecture compliance of the implementations to Realm Management Monitor specification.

### Trusted Firmware-A Tests (tf-a-tests)
The **[Trusted Firmware-A Tests(tf-a-test)](https://review.trustedfirmware.org/TF-A/tf-a-tests.git)** is another set of tests providing compliance checks with RMM specification.

## Running Tests

### ACS Test
To run the ACS test, execute the following commands:
```bash
./scripts/tests/acs.sh
```
Alternatively, you can run the test directly using:
```bash
./scripts/fvp-cca -bo -nw=acs -rmm=islet --rmm-log-level=error
./scripts/fvp-cca -ro -nw=acs -rmm=islet
```

### Trusted Firmware-A Tests (tf-a-test)
To run the Trusted Firmware-A tests, execute the following commands:
```bash
./scripts/tests/tf-a-tests.sh
```
Alternatively, you can run the test directly using:
```bash
./scripts/fvp-cca --clean tf-a-tests
./scripts/fvp-cca -bo -nw=tf-a-tests -rmm=islet
./scripts/fvp-cca -ro -nw=tf-a-tests -rmm=islet
```

## CI Integration
All PRs modifying Islet RMM are automatically tested in the CI pipeline. The CI process runs both the ACS test and tf-a-tests to ensure compliance with the RMM specification. PRs must pass these tests before they can be merged.

By following this guidance, Islet ensures that its RMM implementation remains compliant with Arm's RMM specification and maintains high-quality standards.
