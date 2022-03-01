/* Layout for the RISC-V 32 boards, used by the examples in this repository. */

/* Memory Spaces Definitions, 448K flash, 64K ram */
PROG_LENGTH = 32M;
RAM_LENGTH  = 0x1800;

TBF_HEADER_SIZE = 0x48;
INCLUDE libtock_layout.ld
