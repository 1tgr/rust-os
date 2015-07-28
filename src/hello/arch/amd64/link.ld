OUTPUT_FORMAT(elf64-x86-64)
ENTRY(start)
phys = 0x00001000;
SECTIONS
{
  .text phys : AT(phys) {
    code = .;
    *(.init)
    *(.text .text.*)
    *(.rodata .rodata.*)
  }
  .data : AT(phys + (data - code))
  {
    data = .;
    *(.data .data.*)
  }
  .bss : AT(phys + (bss - code))
  {
    bss = .;
    *(.bss .bss.*)
  }
  end = .;
}
