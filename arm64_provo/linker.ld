/* Bare-metal memormapo por `qemu-system-aarch64 -M virt -kernel`.
 * QEMU ŝargas la ELF-segmentojn laŭ ilia fizika adreso; 0x4000_0000
 * estas la komenco de la RAM de la «virt» maŝino. */
ENTRY(_start)

SECTIONS
{
    . = 0x40000000;

    .text : ALIGN(4K)
    {
        *(.text._start)
        *(.text .text.*)
    }

    .rodata : ALIGN(4K)
    {
        *(.rodata .rodata.*)
    }

    .data : ALIGN(4K)
    {
        *(.data .data.*)
    }

    .bss : ALIGN(4K)
    {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }

    . = ALIGN(16);
    . = . + 0x10000;
    __stack_top = .;

    /DISCARD/ :
    {
        *(.eh_frame .eh_frame_hdr)
        *(.note .note.*)
        *(.comment)
    }
}
