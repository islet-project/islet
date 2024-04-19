#!/bin/bash

set -e

cargo install cross --git https://github.com/bitboom/cross
sudo systemctl start docker
