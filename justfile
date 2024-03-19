project_name := "xv6"
target_path := "./target/riscv64gc-unknown-none-elf/"
build_type := "debug"

alias r := run

kernel_path := target_path + build_type + "/" + project_name

kernel:
    cargo build

run *EXTRA_ARGS: kernel
    qemu-system-riscv64 {{EXTRA_ARGS}} -M virt -m 2G -nographic \
    -kernel {{kernel_path}} -bios none

debug port="1234": (run "-gdb tcp::" + port + " -S")
gdb: kernel
    riscv64-linux-gnu-gdb {{kernel_path}} \
        -ex 'target remote localhost:1234'
