A Rust operating system
=======================

Based on the [Rust barebones kernel](https://github.com/thepowersgang/rust-barebones-kernel).

Rustdocs for the kernel: http://1tgr.github.io/rust-os/kernel/

Features:
 - Targets 64-bit x86
 - Kernel has embedded unit tests
 - Scheduler
  - Threads
  - Processes
 - Memory manager
  - Demand paging
  - Memory protection
  - Shared memory
 - Input and output
  - Keyboard input
  - Text-mode output
  - Linear frame buffer (on QEMU/Bochs/VirtualBox)
 - User mode
  - Separation between user mode (ring 3) and kernel mode (ring 0)
  - Syscall interface (via SYSCALL/SYSRET instructions)
  - C runtime (Newlib)
  - Cairo graphics

Works in progress:
 - Graphics compositor

Todo:
 - Mouse
 - Window system
 - File system
 - Networking
