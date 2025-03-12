OUTPUT_FORMAT(elf32-littleriscv)
ENTRY(_start)
phys = 0x00000000;
SECTIONS
{
  .text phys : AT(phys) {
    code = .;
    *(.text)
    *(.rodata)
    . = ALIGN(4096);
  }
  .data : AT(phys + (data - code))
  {
    data = .;
    *(.data)
    . = ALIGN(4096);
  }
  .got : AT(phys + (got - code))
  {
    got = .;
    __global_pointer$ = .;
    *(.got)
    . = ALIGN(4096);
  }
  .bss : AT(phys + (bss - code))
  {
    __bss_start = .;
    bss = .;
    *(.bss)
    . = ALIGN(4096);
  }
  _end = .;
}
