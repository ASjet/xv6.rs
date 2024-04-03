MEMORY
{
    /* 0x80000000 is where QEMU's -kernel jumps with -bios none */
    RAM : ORIGIN = 0x80000000, LENGTH = 128M
}

REGION_ALIAS("REGION_TEXT", RAM);
REGION_ALIAS("REGION_RODATA", RAM);
REGION_ALIAS("REGION_DATA", RAM);
REGION_ALIAS("REGION_BSS", RAM);
REGION_ALIAS("REGION_HEAP", RAM);
REGION_ALIAS("REGION_STACK", RAM);

_hart_stack_size = 1M;
_heap_size = 64M;
_max_hart_id = 8;

PROVIDE(_etext = _stext + SIZEOF(.text));
