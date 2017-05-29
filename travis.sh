#!/bin/bash
set -xeuo pipefail
IFS=$'\n\t'
export PATH=$PATH:$HOME/.cargo/bin:$(pwd)/3rdparty/bin
make -C src
python3 test.py qemu-system-x86_64 -no-kvm -display vnc=:1 -no-reboot -kernel src/kernel/kernel -initrd src/boot/initrd.tar
