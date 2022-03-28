PAGE_SIZE_4K = 4096;
ENTRY(rmm_entry)

MEMORY {
 RAM (rwx): ORIGIN = (((0x80000000) + (0x80000000) - ((0x0) + (0x00300000) + (0x02000000) + (0x00100000)))), LENGTH = ((((0x80000000) + (0x80000000) - ((0x0) + (0x00300000) + (0x02000000) + (0x00100000)))) + (0x02000000)) - (((0x80000000) + (0x80000000) - ((0x0) + (0x00300000) + (0x02000000) + (0x00100000))))
}

SECTIONS
{
 . = (((0x80000000) + (0x80000000) - ((0x0) + (0x00300000) + (0x02000000) + (0x00100000))));
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
  __RMM_STACK_START__ = .;
  KEEP(*(.stack))
  __RMM_STACK_END__ = .;
 } >RAM
 __RW_END__ = .;
 /DISCARD/ : { *(.dynstr*) }
 /DISCARD/ : { *(.dynamic*) }
 /DISCARD/ : { *(.plt*) }
 /DISCARD/ : { *(.interp*) }
 /DISCARD/ : { *(.gnu*) }
 /DISCARD/ : { *(.note*) }
 /DISCARD/ : { *(.comment*) }
}
