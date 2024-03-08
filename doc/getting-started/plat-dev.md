# Platform Developer
Platform developers are who want to develop Confidential Computing Platform Components.
Platform components include from Realm Management Monitor(RMM) to Realm.

`Islet` provides Rust-based RMM and scripts to compose Confidential Computing Platform.
You can explore CCA platform with our scripts and
powerful [third-party projects](https://github.com/islet-project/islet/tree/main/third-party).

## Setting build environment

The first step is to prepare to build our project.

```bash
./scripts/init.sh
```

## Running a linux realm
```bash
// Start FVP on host
$ ./scripts/fvp-cca --normal-world=linux --realm=linux --rmm=islet

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
See [examples](https://github.com/islet-project/islet/tree/main/examples).
To get details about its network configuration, see [network.md](https://github.com/islet-project/islet/blob/main/doc/network.md)

## Testing the realm features
```bash
// Start FVP on fvp
$ ./scripts/fvp-cca --normal-world=linux --realm=linux --rmm=islet

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
# Islet RMM
$ ./scripts/fvp-cca --normal-world=acs --rmm=islet

# TF RMM
$ ./scripts/fvp-cca --normal-world=acs --rmm=tf-rmm
```
