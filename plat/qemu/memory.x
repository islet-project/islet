SIZE_1MB = 0x100000;
SIZE_40MB = 0x2800000;
SIZE_4KB = 0x1000;
SIZE_8GB = 0x200000000;

ENTRY(rmm_entry);

/* Memory layout description
 *
 * Partition            | Origin      | Length
 * ---------------------|-------------|-----------
 * NS PAS (Non-Secure)  | 0x40000000  |    0x100000 (1MB)
 * NS PAS (Non-Secure)  | 0x42900000  | 0x600000000 (3GB~)
 * Secure PAS           | 0x0E100000  |    0xE00000 (14MB)
 * Realm PAS            | 0x40100000  |   0x2800000 (40MB)
 * Root PAS             | 0x0E001000  |     0xFF000 (1MB-1KB)
 * Root PAS             | 0x0EDFC000  |    0x204000 (2MB+16KB)
 * ---------------------|-------------|-----------
 */

MEMORY {
    RAM (rwx)      : ORIGIN = 0x40000000, LENGTH = SIZE_1MB
    REALM_PAS (rxw): ORIGIN = ORIGIN(RAM) + SIZE_1MB, LENGTH = SIZE_40MB
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

    /* LENGTH(REALM_PAS) - SIZE_4KB: last page is reserved for the el3 manifest */
    ASSERT((__RMM_END__ < ORIGIN(REALM_PAS) + LENGTH(REALM_PAS) - SIZE_4KB), "REALM_PAS size exceeded!")
}
