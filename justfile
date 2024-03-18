alias r := run

kernel_path := "./target/riscv64gc-unknown-none-elf/debug/xv6"

kernel:
    cargo build

run *EXTRA_ARGS:
    qemu-system-riscv64 {{EXTRA_ARGS}} -M virt -m 2G -nographic \
        -kernel {{kernel_path}} -bios none

debug: (run "-gdb tcp::1234 -S")
gdb:
    riscv64-linux-gnu-gdb {{kernel_path}} \
        -ex 'target remote localhost:1234'
