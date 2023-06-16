## Setting build environment

The first step is to prepare to build our project.
We provide simple script `./scripts/init.sh`, installing rust and
related packages.
We tested the scripts on Ubuntu 18.04. So, we recommend to use the same
Linux distributions.

```bash
./scripts/init.sh
```

## Running a linux realm
```bash
// Start FVP on host
$ ./scripts/fvp-cca --normal-world=linux --realm=linux --rmm=tf-rmm

// Run a linux realm on fvp
$ ./launch-realm.sh
```

## Running SDK sample apps after [running a linux realm](#running-a-linux-realm)
```bash
// Move to shared dir on realm
$ cd /shared

// Insert RSI kernel module
$ inmod rsi.ko

// Run the sample app (rust)
$ ./sdk-example

// Run the sample app (c)
$ LD_LIBRARY_PATH=./ ./sdk-example-c
```

## Running a linux realm with a networking support and prebuilt examples
See [examples](./examples/README.md).
To get details about its network configuration, see [network.md](./docs/network.md).

## Testing the realm features
```bash
// Start FVP on fvp
$ ./scripts/fvp-cca --normal-world=linux --realm=linux --rmm=tf-rmm

// Test the realm features on fvp
$ ./test-realm.sh [attest]
```

## Testing RMMs with tf-a-tests
```
# Islet RMM
$ ./scripts/fvp-cca --normal-world=tf-a-tests --rmm=islet

# TF RMM
$ ./scripts/fvp-cca --normal-world=tf-a-tests --rmm=tf-rmm
```

## Testing RMMs with [ACS](https://github.com/ARM-software/cca-rmm-acs)
```
# TF RMM
$ ./scripts/fvp-cca --normal-world=acs --rmm=tf-rmm
```
