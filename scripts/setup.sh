#!/bin/bash

# Download rust components
rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
rustup component add llvm-tools-preview

# Install bootloader
cd && cargo install bootimage

# Install QEMU
yay -S qemu-full
