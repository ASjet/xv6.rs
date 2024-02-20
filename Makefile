.PHONY: boot build

boot: build
	qemu-system-x86_64 -drive format=raw,file=target/x86_64-xv6/debug/bootimage-xv6.bin

build:
	cargo bootimage
