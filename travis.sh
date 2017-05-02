#!/bin/bash
set -xeuo pipefail
IFS=$'\n\t'
PATH=$PATH:$(pwd)/3rdparty/binutils/bin:~/.cargo/bin make -C src
python3 test.py qemu-system-x86_64 -no-kvm -display vnc=:1 -no-reboot -kernel src/kernel/kernel -initrd src/boot/initrd.tar
