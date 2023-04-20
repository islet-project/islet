4K = 4096;

ENTRY(_entry)

MEMORY {
  RAM (rwx): ORIGIN = 0x0, LENGTH = (0x100000)
}

SECTIONS
{
  ASSERT(. == ALIGN(4K), "Not aligned")

  .text : {
    KEEP(*(.head.text))
    . = ALIGN(16);
    *(.text*)
  } >RAM

  .rodata : {
    . = ALIGN(4K);
    *(.rodata*)
  } >RAM

  .data : {
    . = ALIGN(4K);
    *(.data*)
  } >RAM

  .bss (NOLOAD) : {
    . = ALIGN(4K);
    __BSS_START__ = .;
    *(.bss*)
    . = ALIGN(4K);
    __BSS_END__ = .;
  } >RAM
  __BSS_SIZE__ = SIZEOF(.bss);

  .stacks (NOLOAD) : {
    __STACK_START__ = .;
    KEEP(*(.stack))
    __STACK_END__ = .;
  } >RAM

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
