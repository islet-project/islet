![Build](https://github.sec.samsung.net/SYSSEC/arm-cca/actions/workflows/build.yml/badge.svg?branch=main)
![License](https://img.shields.io/badge/license-Samsung%20Inner%20Source-informational.svg)
![Test](https://art.sec.samsung.net/artifactory/syssec_generic/arm-cca/test.png)
![Coverage](https://art.sec.samsung.net/artifactory/syssec_generic/arm-cca/coverage.png)

# arm-cca
This repository contains code for confidential computing on the ARM CCA architecture.

## How to prepare build
```bash
./scripts/init.sh
```

Or, use a docker image as the below

```bash
sudo docker run --rm -it art.sec.samsung.net/syssec_docker/cca_build /bin/bash

```

## How to build
```bash
./scripts/build.sh
```

## How to run
```bash
./scripts/run.sh
```

## How to do unit-tests
```bash
./scripts/test.sh --unit-test
```

## How to measure line coverage of unit-tests
```bash
./scripts/test.sh --coverage
```

## How to connect T32
```bash
./scripts/run.sh --cadi-server
```

Then, execute the t32 application (e.g., ./t32marm-qt)
and run the script ./debug/t32.cmm via "File -> Run Script".

## Coding style
For bash scripts,
```bash
assets/formatter/shfmt -w -ci -bn -fn <TARGET>
```

For rust,
```bash
cargo fmt
```

Pre-commit script is ready for convenience.

After installing pre-commit, every commit will be checked automatically
before creation.

```bash
pip3 install pre-commit
pre-commit install
```

.editorconfig is also ready as well.

This file helps use proper indentation when you use editor (e.g., vim, vscode).

You can set the editor configuration like the below if you use vim.

[How to use .editorconfig for vim](https://github.com/editorconfig/editorconfig-vim)

## See also
[Detailed Description](https://pages.github.sec.samsung.net/SYSSEC/arm-cca/)


## List of Maintainers
- Beomheyn Kim (beomheyn.kim@samsung.com)
- Bokdeuk Jeong (bd.jeong@samsung.com)
- Sangwan Kwon (sangwan.kwon@samsung.com)
- Sungbae Yoo (sungbae.yoo@samsung.com)


## Governance
All decisions in this project are made by consensus, respecting the principles and rules of the community.  Please refer to the [Samsung Inner Source Governance](docs/Governance.md) in more detail.
