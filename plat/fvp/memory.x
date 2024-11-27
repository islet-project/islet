SIZE_4MB = 0x400000;
SIZE_32MB = 0x2000000;
SIZE_4KB = 0x1000;
SIZE_2GB = 0x80000000;

ENTRY(rmm_entry);

/* Memory layout description
 *
 * Partition            | Origin      | Length
 * ---------------------|-------------|-----------
 * NS PAS (Non-Secure)  | 0x80000000  | 0x7C000000 (2GB - 64MB)
 * Secure PAS           | 0xFC000000  | 0x01C00000 (28MB)
 * Realm PAS            | 0xFDC00000  | 0x02000000 (32MB)
 * Root PAS             | 0xFFC00000  | 0x00400000 (4MB)
 * ---------------------|-------------|-----------
 */

MEMORY {
    RAM (rwx)      : ORIGIN = 0x80000000, LENGTH = SIZE_2GB
    ROOT_PAS (rwx) : ORIGIN = ORIGIN(RAM) + LENGTH(RAM) - SIZE_4MB, LENGTH = SIZE_4MB
    REALM_PAS (rxw): ORIGIN = ORIGIN(ROOT_PAS) - SIZE_32MB, LENGTH = SIZE_32MB
}

SECTIONS {
    . = ORIGIN(REALM_PAS);
    __RMM_BASE__ = .;

    .text : {
        KEEP(*(.head.text));
        . = ALIGN(16);
        *(.text*);
    } >REALM_PAS

    .rodata : {
        . = ALIGN(SIZE_4KB);
        *(.rodata*);
    } >REALM_PAS

    .data : {
        . = ALIGN(SIZE_4KB);
        __RW_START__ = .;
        *(.data*);
    } >REALM_PAS

    .bss (NOLOAD) : {
        . = ALIGN(16);
        __BSS_START__ = .;
        *(.bss*);
        . = ALIGN(16);
        __BSS_END__ = .;
    } >REALM_PAS
    __BSS_SIZE__ = SIZEOF(.bss);

    __RW_END__ = .;

    .stacks ALIGN(SIZE_4KB) (NOLOAD) : {
        __RMM_STACK_BASE__ = .;
        KEEP(*(.stack));
    } >REALM_PAS

    /DISCARD/ : {
        *(.comment*);
        *(.dynamic*);
        *(.dynstr*);
        *(.eh_frame*);
        *(.gnu*);
        *(.interp*);
        *(.note*);
        *(.plt*);
    }

    __RMM_END__ = .;

    ASSERT((__RMM_END__ < ORIGIN(REALM_PAS) + LENGTH(REALM_PAS)), "REALM_PAS size exceeded!")
}
