OUTPUT_FORMAT(elf64-x86-64)
ENTRY(main)
phys = SEGMENT_START("text-segment", 0x00001000);
SECTIONS
{
  .text phys : AT(phys) {
    *(.text .text.*)
    *(.rodata .rodata.*)
  }
  .tdata : {
    *(.tdata .tdata.* .gnu.linkonce.td.*)
  }
  .tbss : {
    *(.tbss .tbss.* .gnu.linkonce.tb.*)
    *(.tcommon)
  }
  .data ALIGN(0x1000) : AT(ADDR(.data))
  {
    __ctors_start = .;
    KEEP( *(.ctors) )
    __ctors_end = .;
    *(.data .data.*)
  }
  .bss ALIGN(0x1000) : AT(ADDR(.bss))
  {
    *(.bss .bss.*)
  }
}
