.PHONY: boot build

boot: build
	qemu-system-x86_64 -serial stdio -drive format=raw,file=target/x86_64-xv6/debug/bootimage-xv6.bin

boot-gdb: build
	qemu-system-x86_64 -serial stdio -drive format=raw,file=target/x86_64-xv6/debug/bootimage-xv6.bin -gdb tcp::1234 -S

build:
	cargo bootimage
