#!/bin/bash
set -xeuo pipefail
IFS=$'\n\t'
curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2016-08-10
make -s -C 3rdparty binutils

