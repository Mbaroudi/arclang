# Embedded system: the processor and the programmable logic together

Lot 3 ([embedded-fpga.md](embedded-fpga.md)) turns a state machine the
model gives to programmable logic into VHDL in fixed point, proves it in
GHDL against a bit-exact C reference, and synthesises it alone. This part
puts it where it runs: **one SoC**, a VexRiscv processor running the Zephyr
application with the generated blocks on its bus, used two ways from one
description:

- **simulated cycle by cycle** (Verilator): the processor, the bus and the
  blocks together, each test case run through the driver and judged as
  `arclang hdl` judges the VHDL;
- **built into one bitstream** for a board (Yosys, nextpnr-ecp5,
  ecppack): the same SoC, the same Verilog of each block, the same Zephyr
  image in its RAM.

```text
arclang soc model.arc                       # each case in the simulated SoC
arclang soc model.arc --board ulx3s         # and the bitstream for the board
arclang soc model.arc --board ulx3s --no-sim   # the bitstream only
arclang soc model.arc --port /dev/ttyUSB0   # each case on the board, which holds the bitstream
```

## The SoC

`arclang soc` writes `soc.json` (what the model says) and runs
`soc.py` (fixed, part of ArcLang: `src/compiler/embedded/system/soc.py`) with
LiteX:

| Part | What |
|---|---|
| CPU | VexRiscv (LiteX's `standard` variant), reset into the RAM: no BIOS, no ROM |
| RAM | 64 KiB of block RAM at `0x40000000`, holding the Zephyr application from reset |
| Console | LiteX's UART (in the simulator: a TCP port; on the ULX3S: the FTDI USB serial port, 115200 bauds) |
| Timer | LiteX's `timer0`, Zephyr's system clock |
| Blocks | each allocated machine's wrapper `<model>_<machine>_wb` on the Wishbone bus at `0xB0000000 + 0x1000 k`, uncached: where its driver looks for it (`<BLOCK>_HW_BASE`) |
| Clock | the node's `clock:` (25 MHz on the ULX3S: its oscillator; another frequency: the ECP5's PLL) |

The steps:

1. the Zephyr application of the model (`export -f zephyr`), each
   allocated machine called through its driver, built for the step() API
   (`pil_direct`);
2. each block's VHDL turned into Verilog by GHDL's synthesis (`ghdl
   --synth --out=verilog`): one netlist for the simulator and the
   bitstream;
3. the SoC's map (`csr.json`) and from it Zephyr's devicetree overlay
   (LiteX's `litex_json2dts_zephyr`, and the board's other nodes disabled):
   the application knows where the UART and the timer are;
4. `west build` for `litex_vexriscv` with that overlay: `zephyr.bin`
   (24 KB for the charger);
5. the SoC with `zephyr.bin` in its RAM, compiled by Verilator (a few
   seconds) or synthesised for the board.

The SoC in the bitstream and in the simulator is one Python class; only
the platform differs (LiteX's simulator, or `radiona_ulx3s`).

## In the simulated SoC

```text
$ tools/embedded/system/run.sh arclang soc examples/energy/battery-charger/battery_charger.arc

SoC examples/energy/battery-charger/battery_charger.arc — LiteX 2026.8 (VexRiscv standard); Verilator 5.006 2023-01-22 rev (Debian 5.006-3); GHDL 7.0.0-dev (6.0.0.r615.gc9043fcc4) [Dunoon edition]; cc (Debian 12.2.0-14+deb12u1) 12.2.0

  ✓ pass  TC-CHG-003
    verifies SYS-CHG-004, SYS-CHG-005 · machine Charger · block battery_charger_charger
    model: pass · SoC: pass
    timing: step() took at most 296.6 us in the simulated SoC at 25 MHz (the driver, the bus and the block)
    SoC vs fixed-point C: 61 steps × 34 outputs, largest difference 0 (bit-exact)
    fixed point vs model: largest difference 1.788e-7 on charger.setpoint (tolerance 1.586e-6, derived from the formats)
    ...
    ok   SYS-CHG-004: at(300 ms, "Charger".target) == 16 A   model 16 A · SoC 16 A
    ok   final("Charger".charge) > 0.0001 Ah   model 0.001375 Ah · SoC 0.001375 Ah
    ...
  3 SoC verdict(s) cached in examples/energy/battery-charger/.arclang/sim-cache
```

For each `execute:` case of an allocated machine:

1. the model runs and its expectations are judged; the fixed-point C
   reference is compiled and run on this computer (as in `arclang hdl`);
2. the simulator starts (a fresh reset), Zephyr boots on the simulated
   VexRiscv; the bench connects to the console and sends each step's
   inputs as one line (bit for bit, as PIL does); the application converts
   them to words (`arc_fx_from`, in the VexRiscv's soft floating point),
   writes them to the block's registers, starts the step, waits for DONE,
   reads the outputs back and reports them;
3. the block's outputs must be the fixed-point C's **word for word**
   (difference 0), within the quantisation tolerance of the model, the
   expectations judged alike.

So the conversions on the processor, the driver, the bus, the register map
and the block are proved together, cycle by cycle. The time reported is
step() on the target's clock: the conversions, the bus transactions and
the block's own clocks (the charger: about 180 to 300 us at 25 MHz, mostly
the soft floating point of the conversions; the window, all integers:
23 us).

The verdicts are kept as `<case>.soc.json` (kind `soc`) beside the `hdl`
ones; `check`, `gate` and the tables read them as they read PIL verdicts,
keyed by the HDL key, the SoC script, the RAM size and every file of the
application.

**The console link.** The simulator boots before the bench connects, a
board before its port is opened: the `ARCLANG READY` of the boot is
missed. The PIL application now answers a line `?` with `ARCLANG READY`;
the bench asks once a second until it is answered (the Renode bench of
lot 2 never sends `?`: unchanged).

## The bitstream

```text
$ tools/embedded/system/run.sh arclang soc examples/energy/battery-charger/battery_charger.arc --board ulx3s --no-sim

  ✓ bitstream for ulx3s (LFE5U-25F-6BG381C): VexRiscv, 64 KiB of RAM holding the Zephyr application (24132 bytes), battery_charger_charger_wb
    6989 LUT4, 3385 flip-flops, 20 DSP (MULT18X18D), 42 block RAM (DP16KD)
    Fmax 37.22 MHz for a clock of 25 MHz: met
    bitstream .../soc-10000us/board/gateware/radiona_ulx3s.bit
    to the board: openFPGALoader -b ulx3s .../radiona_ulx3s.bit
```

On the LFE5U-25F: 29 % of the LUTs, 75 % of the block RAM (the 64 KiB of
RAM: 32 of them), 71 % of the multipliers. The resources and Fmax are
nextpnr's, after routing; the command fails when timing is not met.

**On the board.** Write the bitstream with `openFPGALoader -b ulx3s
<bit>` (into the FPGA's SRAM; `-f` writes the flash) from a computer that
sees the board's USB (Docker Desktop on macOS does not pass USB devices
through: run openFPGALoader on the host, or from a Linux host with
`--device`). Then `arclang soc model.arc --port /dev/ttyUSB0` runs every
case on the board through its serial port with the same bench and the same
judgement; the time reported is then the board's.

The node's part chooses the board: an `LFE5U-25F/45F/85F` in a `BG381`
package, speed 6, is the ULX3S's. Other boards are a LiteX platform away
(`litex_boards`), not built in.

## In Docker

```text
docker build -t arclang-embedded:1 tools/embedded
docker build -t arclang-embedded-soc:1 tools/embedded/soc
docker build -t arclang-embedded-fpga:1 tools/embedded/fpga
docker build -t arclang-embedded-system:1 tools/embedded/system
tools/embedded/system/run.sh cargo test --release --test embedded_system -- --include-ignored
```

The system image is the FPGA image (GHDL, Yosys, nextpnr-ecp5) with
Zephyr, its SDK and west copied from the SoC image, LiteX, litex-boards
and their data packages at 2026.08, Migen at e19524c9, Verilator 5.006 and
openFPGALoader 1.2.0. The CI job "Embedded system" builds the four images
and runs those tests.

## Limits, said plainly

- **No board here.** The bitstream is built and meets timing; the SoC it
  holds is the one simulated cycle by cycle, and its Zephyr image the one
  that ran there. Writing it to a ULX3S and `--port` have not been run on
  a board: the first run on hardware is the next proof.
- **One SoC, one clock, one part**: every allocated machine on nodes of
  the same part and clock.
- **The step() API only** in the SoC: the cases run through the console,
  not through the I/O channels (gpio, ADC, PWM, CAN...) of a binding; those
  stay proved by PIL in Renode, where the machine runs as software C.
- **The VexRiscv's soft floating point** converts the inputs and outputs:
  most of step()'s time. A machine whose inputs are integers, or a
  processor with an FPU, would not pay it.
- **ULX3S only** as a built-in board.
