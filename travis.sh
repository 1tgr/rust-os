#!/bin/bash
set -xeuo pipefail
IFS=$'\n\t'
export PATH=$PATH:$(pwd)/3rdparty/binutils/bin:~/.cargo/bin
(
    cd src
    tup generate build.sh
    ./build.sh
)
python3 test.py qemu-system-x86_64 -no-kvm -display vnc=:1 -no-reboot -kernel src/kernel/kernel -initrd src/boot/initrd.tar
