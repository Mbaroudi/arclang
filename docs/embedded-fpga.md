# Embedded FPGA: the parts of a model the programmable logic computes

Lots 1 and 2 generate a model's behavior as C that knows no board
([embedded-c.md](embedded-c.md)) and take it to a processor through target
packages ([embedded-targets.md](embedded-targets.md)). Lot 3 adds the
hardware/software split: a state machine the model gives to the
programmable logic of the system becomes a VHDL entity, computed in fixed
point, and the software calls it through generated registers.

```text
arclang export model.arc -f vhdl -o build/hw      # VHDL, register map, driver, fixed-point C reference
arclang hdl    model.arc                          # each case in GHDL, against the fixed-point C and the model
arclang hdl    model.arc --synth                  # and synthesis for the ECP5: resources, Fmax
arclang export model.arc -f zephyr -o build/app   # the Zephyr application calls the block through its driver
arclang soc    model.arc --board ulx3s            # processor and logic together: simulated, and one bitstream
```

The SoC that puts the processor and the blocks together, simulated cycle
by cycle and built into one bitstream, is in
[embedded-system.md](embedded-system.md).

The three layers stay: the generated code knows no board (the VHDL uses
numeric_std only, the driver takes the block's address as a macro); one I/O
interface (the processor still reads and writes the channels, the block
computes); per-target packages (the device and its clock are the node's).

## Allocation

Nothing new in the language but the nature of a node. A state machine is
the behavior of a component (`owner:`); a physical node performs it when it
`deploys` that component, or the machine itself by name or id, or when the
component is one of its `behavior_component`s, as Arcadia allocates
behavior to physical components. The node is programmable logic when its
`kind:` says so (`"fpga"` or `"programmable_logic"`):

```arclang
physical_architecture "Charger hardware" {
    node "Charger SoC" { processor: "VexRiscv (LiteX)" }
    node "Charger fabric" {
        kind: "fpga"
        device: "LFE5U-25F-6BG381C"     // the part: default this one (ULX3S, Colorlight class)
        clock: 25 MHz                   // the clock of the logic: default 25 MHz
        behavior_component "Charge controller" { id: "PC-CHG-CTRL" }
    }
}
state_machine "Charger" { owner: "PC-CHG-CTRL" ... }
```

or, shorter, `node "Fabric" { kind: "fpga" deploys "Window" }`. A machine
on two programmable logic nodes is an error; so is a `device:` that is no
Lattice ECP5 part (`LFE5U-`, `LFE5UM-`, `LFE5UM5G-` with 12, 25, 45 or 85 k
LUTs, speed 6 to 8, the BG256, BG381, BG554, BG756 or MG285 package).

Why `kind:` and not a new keyword: `deploys`, `behavior_component` and
`owner:` already say which component performs which behavior on which
node; what was missing is that a node is logic rather than a processor. A
node attribute is the smallest addition, it reads like the rest of a node
(`processor:`, `memory:`) and the Capella exchanges keep it as an attribute.

The compiler then checks that the hardware can compute each allocated
machine, and refuses, naming the construct, what it cannot:

| In the hardware | Refused |
|---|---|
| `+ - *`, division by a constant, `gain`, `saturate`, `min`, `max`, `abs` | `/` by an expression that is not a constant |
| `delay`, `integrator` (with limits, or feeding a variable directly), `pid` with limits | a `pid` without limits (its integral has no bound); an `integrator` without limits inside an expression; a block whose gains or limits are not numbers |
| comparisons, `and`, `or`, `not`, Booleans, `Integer` (saturating) | a `Real` with no range (an `in` one, or one fed back on itself); a range needing more than 31 integer bits |
| every state, region and pseudo-state of the C, timers, budgets, actions | a machine with more than 255 events |

```text
state_machine 'M': equation "y = a / b": '/' by an expression that is not a constant: the hardware divides only by constants (it is allocated to programmable logic node 'Fabric': the hardware computes it in fixed point)
state_machine 'M': in variable 'a' (Real) has no range: the hardware holds it in fixed point, its format comes from its range; declare it ("in a: Real range <low> .. <high>")
state_machine 'M': the range of variable 'f' cannot be derived: it is fed back and grows at every pass; declare it ("f: Real range <low> .. <high>")
state_machine 'M': equation "y = pid(e, 1, 1 Hz, 0 s)": pid(...): a pid without limits has an integral without bound; the hardware needs its limits, pid(e, kp, ki, kd, lo, hi)
```

A machine that is not allocated is software: nothing of this is asked of
it, and its C, SIL and PIL are unchanged.

## Fixed point

Every `Real` (and `Float`) value of the hardware is a signed word with a
binary point of its own, its Q format: `Qi.f`, `i` integer bits and `f`
fraction bits, `f = 31 - i` in a 32-bit word unless the variable asks for
a narrower one (below). An `Integer` stays the model's saturating `int32`,
a `Boolean` a bit.

**Ranges.** A variable declaration takes a range and, if wanted, a format:

```arclang
variables: [
    "in voltage: Real [V] range 0 V .. 60 V",            // Q6.25
    "in current: Real [A] range -25 A .. 25 A",          // Q5.26
    "filtered: Real [V] range 0 V .. 60 V = 0 V",        // fed back: declared
    "out charge: Real [Ah] range -10 Ah .. 10 Ah",       // Q4.27
    "target: Real [A] = 0 A",                            // derived: 0 .. 16 A, Q5.26
    "out setpoint: Real [A]"                             // derived: the pid's limits 0 .. 20 A, Q5.26
]
```

- `range lo .. hi` is in any unit of the variable's dimension; the model
  itself does not clamp to it (the floating-point run is unchanged); the
  initial value must be in it.
- Without one, the range is **derived**: a `const` is its value; any other
  variable takes the hull of its initial value and of every value assigned
  to it (its equation, every action), computed again until it stops
  growing. A variable fed back on itself (`filtered = filtered + 0.2 *
  (voltage - filtered)`) grows at every pass: it must be declared. An `in`
  variable has nothing assigned: declared, or an error.
- `format Q<i>.<f>` (i + f at most 31: a word of i + f + 1 bits) sets the
  format instead of the range's.
- `resolution r` (in the variable's unit) asks for the narrowest word that
  holds the range with a step of at most `r` (below).

**The format of a range**: the fewest integer bits `i` for which the
largest magnitude of the range is below `2^i` (60 V: `i = 6`, Q6.25, a
step of 2^-25 V ≈ 30 nV). Every operation has a format too, from the range
of its result (interval arithmetic over its operands' ranges).

**Operations.** Exact up to the final rounding into their format:

- `+` and `-` align both words to the finer of their two formats (when one
  is more than 30 bits finer than the other, it is rounded to 30 bits finer
  first), add in 64 bits and round into the result's format;
- `*` multiplies the two words exactly (64 bits) and rounds;
- `x / c` is `x * (1/c)`, the reciprocal rounded into its own format (an
  `Integer` divided by a constant is the model's truncated division);
- comparisons, `min`, `max` and `saturate` align as `+` does;
- an assignment rounds into the variable's format.

**Rounding** is to nearest, ties toward plus infinity: `(x + 2^(n-1)) >>
n`, an arithmetic shift (63 bits or more give 0). **Saturation**: every
result is held in the range of its 32-bit word, never wrapped; a value
beyond the declared range (an input outside it, a computed variable whose
declared range is too small) saturates at the end of the format, not at the
range.

**Narrow words.** A variable is held in 32 bits unless it says what it
needs:

```arclang
"in voltage: Real [V] range 0 V .. 60 V resolution 1 mV",    // Q6.10, 17 bits
"in current: Real [A] range -25 A .. 25 A resolution 1 mA",  // Q5.10, 16 bits
"filtered: Real [V] range 0 V .. 60 V resolution 1 mV = 0 V", // Q6.10, 17 bits
"out duty: Real range 0 .. 1 format Q1.9"                     // 11 bits
```

The fraction bits are the fewest whose step is at most the resolution (1 mV:
2^-10 V ≈ 0.98 mV), the integer bits those of the range, plus a sign: a
resolution and a range that need more than 32 bits are refused. Only the
*storage* narrows: the VHDL holds the variable in a register of that width
(`signed(16 downto 0)`), the arithmetic that computes it is unchanged
(aligned, 64 bits, rounded into the format), and the value is saturated to
the word when stored, in the VHDL as in the C reference (`arc_fx_narrow`),
so the two stay bit-exact; it is sign-extended where it is read. An input
is saturated to its word where it is formed: by the driver
(`arc_fx_narrow(arc_fx_from(v, f), w)`), by `arclang hdl`'s stimulus, and
by the block when it latches the register. A narrow word keeps the same
bound as a 32-bit one (±2^i): only its step is coarser, and the
quantisation tolerance grows with it.

The charger with those three resolutions (the other variables at 32 bits),
bit-exact in GHDL with its C reference on its three cases:

| Charger | LUT4 | FF | DSP | Fmax |
|---|---|---|---|---|
| every `Real` in 32 bits | 3436 | 1347 | 16 | 32.6 MHz |
| voltage, current, filtered at 1 mV / 1 mA | 3336 | 1185 | 14 | 26.8 MHz |

The saving is in the registers and the multipliers that read them; the
intermediate results keep their 32-bit formats (a value's word is not
narrowed unless it is stored).

**Blocks.** Their gains are constants, folded at the period of the code
(`k*T`, `ki*T`, `kd/T`, each rounded into its own format):

| Block | In fixed point |
|---|---|
| `delay(x, initial)` | the held word, in the format of the hull of x's range and the initial value |
| `integrator(x, k, lo, hi)` | state in the format of `[lo, hi]`, `s + kT*x` then held in `[lo, hi]`, as the model |
| `integrator(x, k)` | assigned to a variable directly (or times a constant: a unit conversion), its state in the format of that variable's range divided by the factor; it saturates at the ends of that format |
| `pid(e, kp, ki, kd, lo, hi)` | output in the format of `[lo, hi]`; the integral in the format of `[lo - M, hi + M]`, `M` the largest proportional, derivative and integral step (clamping anti-windup keeps it there); `kp*e + I + (kd/T)*(e - e_previous)`, the anti-windup test on the words |

The constants of the code are those of one period: the hardware of a
machine is generated for a period (that of its first binding, else 10 ms;
in `arclang hdl`, the case's step), and the C reference refuses to compile
for another (`#error`).

**The quantisation error.** For each case, a bound of how far each `Real`
variable's fixed-point value may be from the floating-point model's is
derived from the formats: every rounding of the code (half a unit in the
last place of the format it rounds into), every constant's rounding, every
input's (the case's values converted to words), propagated through what
reads it step after step, an action counting in every step; plus the
floating-point model's own rounding (2^-40 of the range a step; 2^-20 for a
`Float`).

The propagation is a first-order affine form: an expression's error is a
signed combination of the errors of what it reads, plus the roundings.
Sums, differences, negations, resizes and products by a constant keep the
combination, so an error read twice cancels as the algebra says: `f + 0.2
* (v - f)` carries 0.8 of `f`'s error a step, not 1.2, and the bound
converges instead of growing. Anything else (a product of two variables,
`min`, `max`, `saturate`, a choice) is bounded by magnitudes, as before.
The charger's filter after 600 ms: a bound of 1.85e-7 V for a difference
seen of 7.2e-8 V (at 32 bits; 2.4e-3 V for 9.7e-4 V at 1 mV).
`quantisation: 0.001` in a case's `execute:` sets the
tolerance instead (in each variable's unit), and the outcome says which it
is. The states, actions, `Integer`s, `Boolean`s and budgets must be
equal: a difference there means the quantisation changed a decision (a
guard on a value at its threshold), and the case fails saying so.

## The bit-exact C reference

`arclang export -f vhdl` writes, beside the VHDL, `c/<model>_fx.h/.c`: the
same C generator as lot 1 (`cgen`), each allocated machine's code printed
from its fixed-point lowering, every operation a static helper
(`arc_fx_align`, `arc_fx_resize`...) whose arithmetic is the VHDL
package's. A `Real` is then an `int32_t`, its word (the header says its
format); no `double` remains. The shifts are written as floor divisions,
exact on any C99 compiler. It compiles with `-std=c99 -Wall -Wextra -Werror
-pedantic`. It is the oracle of `arclang hdl`, and fixed-point code a
processor without floating point can run.

## The VHDL

For each allocated machine `<model>_<machine>` (`battery_charger_charger`):

| File | What |
|---|---|
| `arclang_fx_pkg.vhd` | the fixed-point and saturating integer arithmetic, the C reference's helpers bit for bit |
| `<block>.vhd` | the entity of the machine |
| `<block>_wb.vhd` | its Wishbone wrapper, the registers below |
| `<block>_regs.json`, `<block>_csr.csv` | the register map, as JSON and as LiteX `csr.csv` lines |
| `<block>_hw.h` | the C driver |

VHDL-2008, `ieee.numeric_std` only, synthesisable, no vendor primitive
(the multipliers are `*` on `signed`: Yosys maps them to the ECP5's
MULT18X18D). The same model gives the same bytes. Every file passes `ghdl
-a --std=08`, the wrapper `ghdl --synth`.

The entity:

```vhdl
entity battery_charger_charger is
  port (
    clk : in std_logic;
    rst : in std_logic;                    -- synchronous: back to init()
    step : in std_logic;                   -- one step, taken while ready
    ready : out std_logic;                 -- waiting for a step
    done : out std_logic;                  -- one clock: the outputs are those of the step
    cycles : out unsigned(15 downto 0);    -- the clocks the last step took
    ev_count : in unsigned(4 downto 0);    -- the events of the step, in the order they arrived
    ev_queue : in std_logic_vector(127 downto 0);
    i_cond_battery_present : in std_logic; -- conditions
    i_v_voltage : in signed(31 downto 0);  -- in variables, Q6.25
    o_st_charging : out std_logic;         -- states, actions (times run), variables, budgets
    o_act_close_contactors : out unsigned(7 downto 0);
    o_v_setpoint : out signed(31 downto 0);
    ...
  );
```

It does what the C's `step()` does, as lot 1 defines it, the procedures of
one clocked process mirroring the C's functions (entry and exit actions,
`exit_range`, histories, each compound transition, each trigger). On
`step` it latches its inputs and runs the step in phases, one a clock: the
initial states (the first step), each equation, each event of the queue
(one a clock, in the order they arrived), each timed trigger, the
completion transitions, the `do` activities and the time in state; then it
sets its outputs and pulses `done`. The charger takes 11 or 12 clocks a step
(5 equations, the events, a timer, the completion), the power window 2 to
5; at 25 MHz, half a microsecond of a 10 ms period.

## The registers

Thirty-two-bit words on a Wishbone B4 classic bus, word addressed (LiteX's
`wishbone.Interface`, 32 bits: the wrapper takes the low bits of `adr`),
`ack` one clock after `stb`, `err` never; `irq` is high while DONE.

| Register | Access | What |
|---|---|---|
| `CTRL` | write | bit 0 START: one step with the inputs written; bit 1 RESET: back to `init()` |
| `STATUS` | read | bit 0 READY, bit 1 DONE (the outputs are those of the last step; cleared by START) |
| `CYCLES` | read | the clocks the last step took |
| `EV_COUNT`, `EV_QUEUE0..15` | read/write | the events of the step and their codes, in the order they arrived |
| `COND<n>` | read/write | the conditions, 32 a word |
| `IN_<variable>` | read/write | an `in` variable: its word, its integer, bit 0 |
| `STATE<n>` | read | the states, 32 a word |
| `ACT_<action>` | read | the times an action ran in the step |
| `VAR_<variable>` | read | a variable after the step |
| `BUDGET<n>` | read | the budgets, 32 a word |

The JSON names each register, its index, byte offset, access, format and
meaning; the `csr.csv` lines are LiteX's, at base 0 (add the base of the
block's region). The driver, `<block>_hw.h`, is static inline C over the
block's address `<BLOCK>_HW_BASE`:

```c
#define BATTERY_CHARGER_CHARGER_HW_BASE 0xB0000000U
#include "battery_charger_charger_hw.h"

battery_charger_charger_hw_reset();
/* the software's structures of <model>.h: values converted to words and back */
if (battery_charger_charger_hw_step(&inputs.charger, &outputs.charger) != 0) {
    /* the block never became ready or never finished */
}
```

It writes the inputs (a `double` rounded to its word as the reference
rounds it: `arc_fx_from`), START, waits for DONE (a bounded number of
reads), and reads the outputs back (`arc_fx_to`).

**In the Zephyr export.** When a machine is allocated to programmable
logic, `arclang export -f zephyr` writes `<model>.c` with that machine's
step replaced by a call to its block (`<block>_hw_step()`) and its reset by
`<block>_hw_reset()`, the driver in `src/model/`, the VHDL in `hw/`, and
`<BLOCK>_HW_BASE` in `arclang_app.cmake` (0xB0000000, then one 4 KB page a
block; `-D<BLOCK>_HW_BASE=...` for another). The application builds with
west for the LiteX VexRiscv board (`tests/embedded_soc.rs`). `arclang pil`
runs the software C of the machine: the emulated SoC has no such block.

## arclang hdl

```text
$ tools/embedded/fpga/run.sh arclang hdl examples/energy/battery-charger/battery_charger.arc --synth

HDL examples/energy/battery-charger/battery_charger.arc — GHDL 7.0.0-dev (6.0.0.r615.gc9043fcc4); cc (Debian 12.2.0-14+deb12u1) 12.2.0; Yosys 0.70+286; nextpnr-ecp5 (nextpnr-0.11.1-85-ga2c64799)

  ✓ pass  TC-CHG-003
    verifies SYS-CHG-004, SYS-CHG-005 · machine Charger · block battery_charger_charger
    model: pass · VHDL: pass
    timing: 11 to 12 clock cycles a step (the events of the step: one a clock)
    VHDL vs fixed-point C: 61 steps × 34 outputs, largest difference 0 (bit-exact)
    fixed point vs model: largest difference 1.788e-7 on charger.setpoint (tolerance 1.586e-6, derived from the formats)
      charger.voltage Q6.25: 0.000e0 ≤ 3.329e-9
      charger.setpoint Q5.26: 1.788e-7 ≤ 1.586e-6
      charger.filtered Q6.25: 7.154e-8 ≤ 1.228e-2
      charger.charge Q4.27: 6.309e-8 ≤ 2.620e-7
      ...
    ok   SYS-CHG-004: at(100 ms, "Charger".target) == 2 A   model 2 A · VHDL 2 A
    ...

  ✓ synthesis battery_charger_charger_wb on LFE5U-25F-6BG381C (nextpnr-ecp5 --25k --package CABGA381 --speed 6)
    3436 LUT4, 1347 flip-flops, 16 DSP (MULT18X18D), 0 block RAM (DP16KD), 82 I/O
    Fmax 32.63 MHz for a clock of 25 MHz: met
```

For each `execute:` case of an allocated machine (the others are left to
`sil` and `pil`), `arclang hdl`:

1. runs the model (floating point) and judges the expectations on it;
2. generates the fixed-point C reference at the case's step, compiles it
   with the strict flags (`$CC`, else `cc`) and runs the case's inputs, as
   words, through its `step()`;
3. generates the VHDL and **a VHDL testbench** that plays the case on the
   Wishbone wrapper: each step's inputs written to their registers, START,
   STATUS polled until DONE, every output register read back. GHDL
   analyses, elaborates and runs it (`--std=08`). The testbench is
   generated VHDL rather than cocotb: one tool, no Python and no VPI in the
   loop, the same file usable in any VHDL simulator, and it drives the
   registers, so the map and the wrapper are tested with the entity;
4. compares the VHDL's words with the C reference's, every output at every
   step: the difference must be 0;
5. compares the C reference's values with the model's: within the
   quantisation tolerance for the `Real` variables, equal for the rest;
6. judges the expectations on the VHDL run.

`pass`: bit-exact, within the tolerance, every expectation holding and
judged alike on the model; `fail`: a difference, or an expectation that
fails; `error`: a tool failed. `--test` runs one case, `--keep DIR` keeps
the VHDL, testbenches, C reference, stimuli, results and netlists, `--json`
prints the outcomes and the synthesis. The command fails when a case does
not pass or a block misses timing.

**Verdicts** are kept as `<case>.hdl.json` in `.arclang/sim-cache/`
(kind `hdl`, the block named), keyed by the generated VHDL and fixed-point
C at the case's period, the case, the generator's version and the tools'
versions (GHDL, the C compiler, and Yosys and nextpnr when it synthesised).
`check`, `gate` and the tables read them as PIL verdicts: a fresh failure
or error is the case's verdict, a fresh pass is evidence, a stale one a
warning.

## Synthesis

With `--synth`, each allocated machine's wrapper goes through:

1. `ghdl -a --std=08` and `ghdl --synth`: GHDL's synthesis elaborates it as
   hardware;
2. Yosys with the GHDL plugin (`yosys -m ghdl`, `ghdl --std=08 ... -e
   <block>_wb; synth_ecp5`);
3. nextpnr-ecp5 for the node's device (`--25k --package CABGA381 --speed
   6`) at the node's clock (`--freq 25`), a fixed seed;
4. ecppack: the bitstream.

It reports the resources nextpnr counts (LUT4 cells, flip-flops,
MULT18X18D multipliers, DP16KD block RAMs, I/O) and the Fmax of the clock
against the node's `clock:`, and fails when timing is not met.

| Block | LUT4 | FF | DSP | BRAM | Fmax | Clock |
|---|---|---|---|---|---|---|
| `battery_charger_charger_wb` (PID, filter, integrator, delay, two regions, a timer) | 3436 | 1347 | 16 | 0 | 32.63 MHz | 25 MHz: met |
| `power_window_window_wb` (regions, choice, junction, fork, join, histories, terminate) | 2292 | 657 | 0 | 0 | 30.31 MHz | 25 MHz: met |

The block is synthesised alone, its bus on the pins nextpnr chooses (81 or
82 I/O of the 197): the bitstream proves the flow end to end; on a board
the block is inside the SoC, its bus on the SoC's interconnect.

## Partitioning: `arclang partition`

Which machines belong in the logic and which on the processor is a
measurement, not a guess. `arclang partition model.arc` measures every
machine on both sides with the real tools, through the same commands:

- as C on the processor: `arclang pil --json` (the target package of the
  bindings, or `--target`): the longest a `step()` took of the target's
  virtual time over the cases, and the behavior's footprint (`size` of
  its object in the Zephyr build: text, data, bss);
- as VHDL on the logic: `arclang hdl --synth --json --as-if <device>@<MHz>`
  (the first logic node's part and clock, or `--device`/`--clock`): LUT4,
  flip-flops, DSPs, block RAMs, the Fmax against the clock — every
  machine, allocated or not.

Each command relays itself to its Docker image when the host lacks the
tools. Then, per machine:

```text
Partition examples/energy/battery-charger/battery_charger.arc — C on zephyr/litex_vexriscv against VHDL on LFE5U-25F-6BG381C at 25 MHz

  Charger — period 10000 us · fits both; now logic
    C on zephyr/litex_vexriscv: a step takes at most 38.2 us (262× of margin), 4.1 KiB text + 16 B data + 256 B bss, over 2 case(s)
    VHDL on LFE5U-25F-6BG381C: 3436 LUT4, 1347 FF, 16 DSP, 0 BRAM; Fmax 32.6 MHz for 25 MHz: met
```

The C fits when a step stays within the machine's period (its binding's,
else 10 ms); the VHDL fits when timing is met. The command says what fits
and what the model allocates today (`now`); the choice stays in the model
(`node ... deploys`). `--machine` measures one machine, `--dry-run` prints
the two runs, `--json` the measurements.

The measurements are kept in `.arclang/sim-cache/partition.json` beside
the model, keyed by the generated C and VHDL, the period, the target, the
device and the clock: the table kind `partition` reads them (`table
"Partition" { kind: partition }`, or `arclang table model.arc --kind
partition`), a row per machine, stale when the generated code changed
since.

`arclang hdl --synth --as-if LFE5U-25F-6BG381C@25 [--machine M]` is the
synthesis alone, for any machine, as if allocated.

## In Docker

Everything runs in the FPGA image, never on the host:

```text
docker build -t arclang-embedded:1 tools/embedded
docker build -t arclang-embedded-fpga:1 tools/embedded/fpga
tools/embedded/fpga/run.sh cargo test --release --test embedded_fpga -- --include-ignored
tools/embedded/fpga/run.sh cargo run --release -- hdl examples/energy/battery-charger/battery_charger.arc --synth
```

`tools/embedded/fpga/Dockerfile` pins three things taken from the same
days, as the GHDL plugin follows libghdl and Yosys closely: GHDL at a
commit of 2026-11-01, built from source with its LLVM 15 back end
(oss-cad-suite no longer ships GHDL, GHDL publishes no Linux arm64 build,
and its mcode back end is x86 only, so it is built the same way on arm64
and x86_64); oss-cad-suite 2026-11-01 (Yosys, nextpnr-ecp5, ecppack, the
Trellis database) for the host's architecture, pruned of the other
families' place and route, simulators, solvers and GUIs; the
ghdl-yosys-plugin at a commit of 2026-11-01, built against both. The image
is 4.7 GB (1.4 GB over the embedded image). The CI job "Embedded FPGA
(GHDL, Yosys, nextpnr)" builds both images and runs those tests.

## Limits, said plainly

- **One element is a machine.** The unit of allocation is a state machine
  (with its variables, equations and blocks); a single equation or a
  function of the functional analysis is not allocated on its own.
- **One clock, sequential phases.** A step takes a few clocks (one per
  equation and per event); nothing is pipelined across steps, and the
  combinational depth of the largest equation (the PID: three products and
  two sums) sets the Fmax. Enough for control periods in milliseconds;
  not a DSP datapath.
- **Narrow storage, not narrow arithmetic.** A `resolution` narrows a
  variable's register, not the intermediate results that compute it.
- **The tolerance is a first-order bound**: tight for linear code, loose
  through products of two variables and through `min`, `max` and
  `saturate`.
- **ECP5 only** for synthesis (nextpnr-ecp5); the VHDL itself is vendor
  neutral.
- **`arclang hdl --synth` synthesises a block alone.** The SoC with the
  processor and the blocks, simulated together and built into one
  bitstream, is `arclang soc` ([embedded-system.md](embedded-system.md));
  no physical board has run it yet.
