# Build

## Setting build environment

The first step is to prepare to build our project.
We provide simple script `./scripts/init.sh`, installing rust and
related packages.
We tested the scripts on Ubuntu 18.04. So, we recommend to use the same
Linux distributions.


```bash
./scripts/init.sh
```

This is only needed at the first time when you start and requires sudo
password.

## Build whole project

We provide `./scripts/build.sh`, building whole related binaries like
firmware, RMM, normal world environment (kernel, bootloader, root filesystem).

```bash
./scripts/build.sh
```
