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
#if defined(ARCLANG_IDENTITY) || defined(ARCLANG_CHIP)
/* The registers of the chip, when what the code does to them is what is proven:
   a read gives what the register holds, a write is kept. */
static uint8_t arclang_io[0x100];
#endif
#ifdef ARCLANG_CHIP
/* The chip beneath the code (its pins, its time), written by the proof after the code. */
static uint8_t arclang_chip_rd(unsigned at);
static void arclang_chip_wr(unsigned at, uint8_t value);
#endif
static bool arclang_uncovered;      /* an instruction ran that the translation does not cover */
static bool arclang_outside;        /* an address outside the memory or the flash was used */
static bool arclang_wrote_register; /* a register of the chip was written by the code itself */
static bool arclang_booting;        /* the code runs from reset, before its first loop() */
static bool arclang_stop;           /* the first loop() was reached: every function returns */
static bool arclang_resume;         /* enter the function that holds the cut at the cut */
/* The cut: the head of the cycle the code never leaves. Arriving there ends the start, or a turn. */
static bool arclang_cut(void)
{
    arclang_stop = true;
    return true;
}
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

/* A value of up to eight bytes, low byte first. Written without a loop: a prover unwinds a loop turn by turn. */
static uint64_t arclang_reg_rd(unsigned at, int size)
{
    uint64_t value = arclang_reg[at & 0xffu];
    if (size > 1) { value |= (uint64_t)arclang_reg[(at + 1u) & 0xffu] << 8; }
    if (size > 2) { value |= (uint64_t)arclang_reg[(at + 2u) & 0xffu] << 16; }
    if (size > 3) { value |= (uint64_t)arclang_reg[(at + 3u) & 0xffu] << 24; }
    if (size > 4) { value |= (uint64_t)arclang_reg[(at + 4u) & 0xffu] << 32; }
    if (size > 5) { value |= (uint64_t)arclang_reg[(at + 5u) & 0xffu] << 40; }
    if (size > 6) { value |= (uint64_t)arclang_reg[(at + 6u) & 0xffu] << 48; }
    if (size > 7) { value |= (uint64_t)arclang_reg[(at + 7u) & 0xffu] << 56; }
    return value;
}

static void arclang_reg_wr(unsigned at, int size, uint64_t value)
{
    arclang_reg[at & 0xffu] = (uint8_t)value;
    if (size > 1) { arclang_reg[(at + 1u) & 0xffu] = (uint8_t)(value >> 8); }
    if (size > 2) { arclang_reg[(at + 2u) & 0xffu] = (uint8_t)(value >> 16); }
    if (size > 3) { arclang_reg[(at + 3u) & 0xffu] = (uint8_t)(value >> 24); }
    if (size > 4) { arclang_reg[(at + 4u) & 0xffu] = (uint8_t)(value >> 32); }
    if (size > 5) { arclang_reg[(at + 5u) & 0xffu] = (uint8_t)(value >> 40); }
    if (size > 6) { arclang_reg[(at + 6u) & 0xffu] = (uint8_t)(value >> 48); }
    if (size > 7) { arclang_reg[(at + 7u) & 0xffu] = (uint8_t)(value >> 56); }
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
#ifdef ARCLANG_IDENTITY
        return arclang_io[at];
#elif defined(ARCLANG_CHIP)
        return arclang_chip_rd((unsigned)at);
#else
        return arclang_any();
#endif
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
#ifdef ARCLANG_IDENTITY
        arclang_io[at] = (uint8_t)value;
        return;
#elif defined(ARCLANG_CHIP)
        arclang_chip_wr((unsigned)at, (uint8_t)value);
        return;
#endif
        if (arclang_booting && !(at >= 0x23u && at <= 0x2bu)) { return; }
        arclang_wrote_register = true;
        return;
    }
    arclang_mem[at] = (uint8_t)value;
}

static uint64_t arclang_mem_rd(uint64_t at, int size)
{
    uint64_t value = arclang_mem_rd1(at);
    if (size > 1) { value |= arclang_mem_rd1(at + 1u) << 8; }
    if (size > 2) { value |= arclang_mem_rd1(at + 2u) << 16; }
    if (size > 3) { value |= arclang_mem_rd1(at + 3u) << 24; }
    if (size > 4) { value |= arclang_mem_rd1(at + 4u) << 32; }
    if (size > 5) { value |= arclang_mem_rd1(at + 5u) << 40; }
    if (size > 6) { value |= arclang_mem_rd1(at + 6u) << 48; }
    if (size > 7) { value |= arclang_mem_rd1(at + 7u) << 56; }
    return value;
}

static void arclang_mem_wr(uint64_t at, int size, uint64_t value)
{
    arclang_mem_wr1(at, value);
    if (size > 1) { arclang_mem_wr1(at + 1u, value >> 8); }
    if (size > 2) { arclang_mem_wr1(at + 2u, value >> 16); }
    if (size > 3) { arclang_mem_wr1(at + 3u, value >> 24); }
    if (size > 4) { arclang_mem_wr1(at + 4u, value >> 32); }
    if (size > 5) { arclang_mem_wr1(at + 5u, value >> 40); }
    if (size > 6) { arclang_mem_wr1(at + 6u, value >> 48); }
    if (size > 7) { arclang_mem_wr1(at + 7u, value >> 56); }
}

static uint64_t arclang_flash_rd1(uint64_t at)
{
    if (at < ARCLANG_FLASH_BASE || at - ARCLANG_FLASH_BASE >= ARCLANG_FLASH_SIZE) { arclang_outside = true; return 0; }
    return arclang_flash[at - ARCLANG_FLASH_BASE];
}

static uint64_t arclang_flash_rd(uint64_t at, int size)
{
    uint64_t value = arclang_flash_rd1(at);
    if (size > 1) { value |= arclang_flash_rd1(at + 1u) << 8; }
    if (size > 2) { value |= arclang_flash_rd1(at + 2u) << 16; }
    if (size > 3) { value |= arclang_flash_rd1(at + 3u) << 24; }
    if (size > 4) { value |= arclang_flash_rd1(at + 4u) << 32; }
    if (size > 5) { value |= arclang_flash_rd1(at + 5u) << 40; }
    if (size > 6) { value |= arclang_flash_rd1(at + 6u) << 48; }
    if (size > 7) { value |= arclang_flash_rd1(at + 7u) << 56; }
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
