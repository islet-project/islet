PAGE_SIZE_4K = 4096;
ENTRY(_entry)

MEMORY {
  RAM (rwx): ORIGIN = 0x88000000, LENGTH = 32 * 1024 * 1024
}

SECTIONS
{
  . = 0x88000000;
  .text : {
    KEEP(*(.head.text))
    . = ALIGN(16);
    *(.text*)
  } >RAM
  .rodata : {
    . = ALIGN(PAGE_SIZE_4K);
    *(.rodata*)
  } >RAM
  .data : {
    . = ALIGN(PAGE_SIZE_4K);
    __RW_START__ = . ;
    *(.data*)
  } >RAM
  .bss (NOLOAD) : {
    . = ALIGN(16);
    __BSS_START__ = .;
    *(.bss*)
    . = ALIGN(16);
    __BSS_END__ = .;
  } >RAM
  __BSS_SIZE__ = SIZEOF(.bss);
  .stacks (NOLOAD) : {
    __STACK_START__ = .;
    KEEP(*(.stack))
    __STACK_END__ = .;
  } >RAM
  __RW_END__ = .;

  /DISCARD/ : {
    *(.comment*)
    *(.dynamic*)
    *(.dynstr*)
    *(.eh_frame*)
    *(.gnu*)
    *(.interp*)
    *(.note*)
    *(.plt*)
  }
}
