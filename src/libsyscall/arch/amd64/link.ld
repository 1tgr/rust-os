OUTPUT_FORMAT(elf64-x86-64)
ENTRY(entry)
phys = 0x00001000;
SECTIONS
{
  .text phys : AT(phys) {
    code = .;
    *(.text .text.*)
    *(.rodata .rodata.*)
  }
  .data ALIGN(0x1000) : AT(ADDR(.data))
  {
    data = .;
    init_array_start = .;
    KEEP( *(.init_array) )
    init_array_end = .;
    *(.data .data.*)
  }
  .bss ALIGN(0x1000) : AT(ADDR(.bss))
  {
    bss = .;
    *(.bss .bss.*)
  }
  end = .;
}
