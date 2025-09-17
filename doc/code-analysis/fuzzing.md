# Fuzz Testing Guide for Islet RMM

## Overview
Islet is built in Rust, which inherently ensures memory safety by design. To further enhance security, we employ tools like **Miri** to verify that `unsafe` code adheres to safety rules and **Kani**, a model checker, for formal verification. Beyond these measures, we also incorporate fuzz testing, a proven method for discovering vulnerabilities, into the Islet RMM development process. We utilize **cargo fuzz**, which leverages **libFuzzer**, to perform fuzz testing effectively.


## Running Fuzz Tests

To execute fuzz tests, use the following command:
```bash
./scripts/fuzz.sh {fuzz test binary}
```
If no fuzz test binary is provided as a command-line argument, a list of available binaries will be displayed. You can then select one from the list and run it.

(to be written)
## Analyzing Fuzz Test Results
(path to the crash inputs and generated inputs, how to reproduce crashes)

(to be written)
## Writing Additional Fuzz Test Harnesses
```
