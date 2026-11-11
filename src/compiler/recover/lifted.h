/* What machine code translated to C runs on, for the proof (written by `arclang prove`):
   the memory, the registers and the flash of an ATmega328P, and what the translation
   could not cover. Nothing here knows the Arduino core. */
#ifndef ARCLANG_LIFTED_H
#define ARCLANG_LIFTED_H
#include <stdbool.h>
#include <stdint.h>

#define ARCLANG_MEM_SIZE 0x900u /* the registers r0..r31, the registers of the chip, 2 KiB of RAM */
#define ARCLANG_IO_FIRST 0x20u
#define ARCLANG_RAM_FIRST 0x100u
#define ARCLANG_SP 0x3du /* the stack pointer, among the registers of the translation */

/* The registers r0..r31 are the first 32 addresses of the memory. They are kept apart from it,
   so that the prover follows each of them alone and not through every write to the memory. */
static uint8_t arclang_r[32];
static uint8_t arclang_mem[ARCLANG_MEM_SIZE];
static uint8_t arclang_reg[0x100];
static bool arclang_uncovered;      /* an instruction ran that the translation does not cover */
static bool arclang_outside;        /* an address outside the memory or the flash was used */
static bool arclang_wrote_register; /* a register of the chip was written by the code itself */
static bool arclang_booting;        /* the code runs from reset, before its first loop() */
static bool arclang_stop;           /* the first loop() was reached: every function returns */
/* Any value: what a register of the chip reads (to the prover, a function named nondet_ gives any value). */
unsigned char nondet_uchar(void);
#define arclang_any() nondet_uchar()

static int64_t arclang_signed(uint64_t value, int size)
{
    int shift = 64 - 8 * size;
    return (int64_t)(value << shift) >> shift;
}

static uint64_t arclang_popcount(uint64_t value)
{
    uint64_t count = 0;
    for (int bit = 0; bit < 64; bit++) { count += (value >> bit) & 1u; }
    return count;
}

static uint64_t arclang_reg_rd(unsigned at, int size)
{
    uint64_t value = 0;
    for (int k = size - 1; k >= 0; k--) { value = (value << 8) | arclang_reg[(at + (unsigned)k) & 0xffu]; }
    return value;
}

static void arclang_reg_wr(unsigned at, int size, uint64_t value)
{
    for (int k = 0; k < size; k++) { arclang_reg[(at + (unsigned)k) & 0xffu] = (uint8_t)(value >> (8 * k)); }
}

static uint64_t arclang_mem_rd1(uint64_t at)
{
    at &= 0xffffu;
    if (at < 32u) { return arclang_r[at]; }
    if (at >= ARCLANG_MEM_SIZE) { arclang_outside = true; return 0; }
    if (at >= ARCLANG_IO_FIRST && at < ARCLANG_RAM_FIRST) {
        /* The stack pointer is read where the chip shows it; any other register reads as anything. */
        if (at == 0x5du) { return arclang_reg[ARCLANG_SP]; }
        if (at == 0x5eu) { return arclang_reg[ARCLANG_SP + 1u]; }
        /* The status register is kept flag by flag; what the translation stores at its address is a copy. */
        if (at == 0x5fu) { return arclang_mem[at]; }
        return arclang_any();
    }
    return arclang_mem[at];
}

static void arclang_mem_wr1(uint64_t at, uint64_t value)
{
    at &= 0xffffu;
    if (at < 32u) { arclang_r[at] = (uint8_t)value; return; }
    if (at >= ARCLANG_MEM_SIZE) { arclang_outside = true; return; }
    if (at >= ARCLANG_IO_FIRST && at < ARCLANG_RAM_FIRST) {
        if (at == 0x5du) { arclang_reg[ARCLANG_SP] = (uint8_t)value; return; }
        if (at == 0x5eu) { arclang_reg[ARCLANG_SP + 1u] = (uint8_t)value; return; }
        /* From reset to the first loop(), the code sets up the chip (timers, converter, serial
           port): that is not looked at. The ports are, always: a pin written by the code itself
           is a write the proof would not see. */
        if (at == 0x5fu) { arclang_mem[at] = (uint8_t)value; return; }
        if (arclang_booting && !(at >= 0x23u && at <= 0x2bu)) { return; }
        arclang_wrote_register = true;
        return;
    }
    arclang_mem[at] = (uint8_t)value;
}

static uint64_t arclang_mem_rd(uint64_t at, int size)
{
    uint64_t value = 0;
    for (int k = size - 1; k >= 0; k--) { value = (value << 8) | arclang_mem_rd1(at + (uint64_t)k); }
    return value;
}

static void arclang_mem_wr(uint64_t at, int size, uint64_t value)
{
    for (int k = 0; k < size; k++) { arclang_mem_wr1(at + (uint64_t)k, value >> (8 * k)); }
}

static uint64_t arclang_flash_rd(uint64_t at, int size)
{
    uint64_t value = 0;
    for (int k = size - 1; k >= 0; k--) {
        uint64_t byte = at + (uint64_t)k;
        if (byte < ARCLANG_FLASH_BASE || byte - ARCLANG_FLASH_BASE >= ARCLANG_FLASH_SIZE) { arclang_outside = true; return 0; }
        value = (value << 8) | arclang_flash[byte - ARCLANG_FLASH_BASE];
    }
    return value;
}

/* What an instruction asks of the chip, beyond its registers and its memory. */
void arclang_op_watchdog_reset(void) { }
void arclang_op_sleep(void) { arclang_uncovered = true; }
void arclang_op_break(void) { arclang_uncovered = true; }
void arclang_op_store_program_mem(void) { arclang_uncovered = true; }

/* The memory, as a correspondence reads it: a byte, two (a word) or four (a long), low byte first. */
#define byte(at) ((unsigned char)arclang_mem[(at)])
#define word(at) ((unsigned int)(arclang_mem[(at)] | ((unsigned int)arclang_mem[(at) + 1] << 8)))
#define dword(at) ((unsigned long)word(at) | ((unsigned long)word((at) + 2) << 16))
#endif
