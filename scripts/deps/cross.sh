#!/bin/bash

set -e

cargo install cross --git https://github.com/cross-rs/cross
sudo systemctl start docker
