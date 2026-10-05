/* Linker memory layout for STM32F303VCT6 */
MEMORY
{
  /* 256 KB Flash */
  FLASH : ORIGIN = 0x08000000, LENGTH = 256K

  /* 40 KB System SRAM (8 KB CCM-SRAM mapped separately at 0x10000000) */
  RAM   : ORIGIN = 0x20000000, LENGTH = 40K
}

/* Set heap and stack placement (optional defaults) */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
