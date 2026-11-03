# Embedded targets: packages, Zephyr on a LiteX SoC, processor in the loop

ArcLang generates the behavior of a model as C that knows no board
([embedded-c.md](embedded-c.md)). Three layers take it to the hardware:

| Layer | What | Where |
|---|---|---|
| 1. The behavior | `<model>.h/.c`: the state machines, `init()` / `step()`, portable C99 | generated |
| 2. The I/O interface | `arclang_io.h` (13 functions), the `io_binding`s of the model, the glue `<model>_<binding>_io.c` | interface fixed, glue generated |
| 3. The target | an implementation of `arclang_io.h`, the build files, the run and flash scripts, the emulator | a **target package** (data) |

```text
arclang export model.arc -f c      -o build/c       # layers 1 and 2, the host board for SIL
arclang sil    model.arc --cc gcc                   # layers 1 and 2 against the model, on this computer
arclang export model.arc -f zephyr -o build/app     # a Zephyr application for the binding's target
arclang pil    model.arc                            # the same C built for the target, run on its processor
```

## Choosing a target

An `io_binding` names the target it is deployed on; without `target:` it is
`host`:

```arclang
io_binding "Charger board" {
    machine: "Charger"
    target: "zephyr/litex_vexriscv"
    period: 10 ms
    watchdog: true
    inputs: ["plug <- gpio 2 rising", "voltage reached <- can 0x210", "above limit <- adc 1 > 3000", ...]
    outputs: ["Charging -> gpio 20", "FanOn -> pwm 0 600", "close contactors -> uart 1 0x43", ...]
}
```

The compiler finds the package and checks every channel the binding uses
against what the package declares, with the reason:

```text
io_binding 'Charger board': 'FanOn -> dac 0 600' cannot be deployed: target 'zephyr/litex_vexriscv' has no dac (it has gpio_in 0-31, gpio_out 0-31, adc 0-7, pwm 0-1, can_rx (16 identifiers), can_tx (2048 identifiers), uart_rx 0-1, uart_tx 0-1, watchdog)
io_binding 'Charger board': 'Charging -> gpio 40' cannot be deployed: target 'zephyr/litex_vexriscv' has gpio_out 0-31, not 40
io_binding 'Charger board': target: 'zephyr/nope' is no target package (known: host, zephyr/litex_vexriscv; more in the directories of ARCLANG_TARGET_PATH, as <dir>/zephyr/nope/target.json)
```

`arclang export -f zephyr` and `arclang pil` take `--target` to deploy
elsewhere (the check is made again for that target), and `export` takes
`--binding` when the model has several. `arclang sil` always runs on the
host board, whatever the target: the host has every channel.

## The package format

A package is a directory with a `target.json` and the files it lists. The
built-in ones are in `src/compiler/embedded/targets/` (compiled into
ArcLang); others are found in the directories of `ARCLANG_TARGET_PATH`
(`<dir>/<family>/<board>/target.json`), and override a built-in one of the
same name.

```json
{
  "name": "zephyr/litex_vexriscv",
  "summary": "Zephyr on a LiteX SoC with a VexRiscv RISC-V core ...",
  "runtime": "zephyr",
  "board": "litex_vexriscv",
  "toolchain": "riscv64-zephyr-elf",
  "channels": {
    "gpio_in":  { "channels": "0-31", "note": "line N of the LiteX GPIO gpio_in" },
    "adc":      { "channels": "0-7",  "note": "a register bank of this target ..." },
    "can_rx":   { "identifiers": 16 },
    "watchdog": {}
  },
  "files": [
    { "from": "../common/arclang_io_zephyr.c", "to": "src/arclang_io_zephyr.c" },
    { "from": "litex_vexriscv.overlay", "to": "boards/litex_vexriscv.overlay" },
    { "from": "run.sh", "to": "run.sh", "executable": true }
  ],
  "build": {
    "command": ["west", "build", "-p", "auto", "-b", "litex_vexriscv", "-d", "{build}", "{app}", "--", "-DARCLANG_MAIN={main}", "{period_define}"],
    "image": "{build}/zephyr/zephyr.elf"
  },
  "emulator": {
    "kind": "renode",
    "command": ["renode", "--disable-gui", "--console", "--plain", "-e", "include @{app}/renode/pil_bench.py"],
    "peripherals": ["renode/ArcLangSJA1000.cs", ...],
    "platform": "renode/arclang_litex_vexriscv.repl",
    "bench": "renode/pil_bench.py",
    "channels": { "gpio_in": "sysbus.gpio_in", "uart": ["sysbus.uart_a0", "sysbus.uart_a1"], ... }
  }
}
```

- `runtime`: what ArcLang generates around the package's files: `host` (the
  C and the host board) or `zephyr` (a Zephyr application: see below).
- `channels`: the channel kinds the target has, among `gpio_in`,
  `gpio_out`, `adc`, `dac`, `pwm`, `can_rx`, `can_tx`, `uart_rx`,
  `uart_tx`, `watchdog`; `channels` lists the numbers (`"0-31"`,
  `"0,2,5-7"`), `identifiers` how many CAN identifiers it takes. A kind
  absent is a kind the target does not have. `note` is shown on export.
- `files`: copied into the export, `from` relative to the package (`../`
  reaches a sibling: the Zephyr packages share `zephyr/common/`).
- `build`, `emulator`: what `arclang pil` runs (absent: the target exports,
  it cannot run in the loop). `{app}` is the exported application, `{build}`
  the build directory, `{main}` `pil_io` or `pil_direct`,
  `{period_define}` `-DARCLANG_PERIOD_US=<step>` for a case without binding.

### Adding a target

- **Another Zephyr board** (an STM32 Nucleo, an nRF, a Zynq's Cortex-R5
  under Zephyr): a directory `zephyr/<board>/` with a `target.json` of
  runtime `zephyr`, the board's devicetree overlay giving the aliases below,
  and the shared files of `zephyr/common/`. Nothing in the compiler changes;
  put it in a directory of `ARCLANG_TARGET_PATH`, or under `targets/` with
  one line per file in `BUILTIN` (`src/compiler/embedded/target.rs`) to ship
  it built in. Without `emulator`, `arclang pil` refuses it; with another
  emulator, its bench reads the same plan.
- **Another runtime** (Linux RT, a bare-metal loop, a PLC runtime): its
  package brings its own `arclang_io` implementation, mains and build
  files, over files ArcLang generates that know no OS (`arclang_app.h`,
  `arclang_io_config.h`, `arclang_pil.c`: the mains include only those).
  What does need the compiler today is the runtime's name: ArcLang accepts
  `host` and `zephyr`, and exports the latter with `-f zephyr`. A package
  of another OS would declare runtime `zephyr` to reuse that export (the
  name is then misleading); a neutral runtime name and export format are
  not done in this lot.

The aliases `arclang_io_zephyr.c` finds the channels by (any Zephyr board):

| arclang channel | devicetree alias | Zephyr API |
|---|---|---|
| `gpio N` read / written | `arclang-gpio-in` / `arclang-gpio-out`, line N | gpio |
| `adc N` | `arclang-adc`, channel N | adc (resolution `ARCLANG_IO_ADC_RESOLUTION`, 16) |
| `dac N` | `arclang-dac`, channel N | dac |
| `pwm N` | `arclang-pwm-N` (0 to 7), channel 0 | pwm (1 kHz, `ARCLANG_IO_PWM_HZ`, duty exact to the per mille) |
| `can ID` | `arclang-can` (an identifier above 0x7FF is extended) | can (one filter per received identifier, at init) |
| `uart N` | `arclang-uart-N` (0 to 7) | uart (polled) |
| the period | the kernel's timer | `k_timer`, periodic |
| the watchdog | `watchdog0` | watchdog (timeout: ten periods, at least 10 ms) |

## The Zephyr application

`arclang export model.arc -f zephyr -o build/app` writes:

```text
CMakeLists.txt  prj.conf  arclang_app.cmake          build files (ARCLANG_MAIN chooses the main)
boards/litex_vexriscv.overlay  .conf                 the SoC's peripherals and the aliases
src/model/<model>.c/.h  src/arclang_io.h             layer 1, the same files as -f c
src/glue/<model>_<binding>_io.c/.h                   layer 2, the binding chosen
src/arclang_io_zephyr.c                              layer 3: arclang_io.h over Zephyr's device APIs
src/arclang_app.h  src/arclang_io_config.h           generated: the model's names, the binding's set-up
src/main.c  src/pil_io.c  src/pil_direct.c           the controller, and the two PIL mains
src/arclang_pil.c/.h  src/pil_link.c                 the PIL report and input line, over the console
arclang_module/                                      a Zephyr module: this SoC's CAN and ADC drivers
renode/                                              the SoC in Renode, its models, the PIL bench
run.sh                                               build and run in Renode
```

`main.c` is the controller: `arclang_io_init()`, then for ever: wait for
the period (`arclang_io_clock_wait_period`, a periodic kernel timer: the
cycles keep the rate whatever each took), then the glue's cycle (read,
step, write, kick the watchdog when all succeeded). `west build -b
litex_vexriscv build/app` builds it; `build/app/run.sh` builds and runs it
in Renode.

## zephyr/litex_vexriscv: what is real

The SoC is LiteX with a VexRiscv core, the board `litex_vexriscv` of Zephyr
v4.4.2, built by the Zephyr SDK 1.0.1 (GCC 14.3, `riscv64-zephyr-elf`,
rv32im, double-precision arithmetic in software), and emulated by Renode
1.17.0. The overlay disables what ArcLang does not use (Ethernet, SD card,
I2S, SPI flash, clock generator, I2C) and reuses three of their interrupt
lines (the SoC has 12).

| Channel | On the SoC | Zephyr driver | In Renode |
|---|---|---|---|
| gpio 0-31 in, 0-31 out | LiteX GPIO cores `gpio_in`, `gpio_out` (32 lines) | upstream `litex,gpio` | Renode's `LiteX_GPIO` |
| uart 0, 1 | two more LiteX UARTs (the console stays on `uart0`) | upstream `litex,uart` | Renode's `LiteX_UART` |
| period | LiteX `timer0` (32-bit CSR bus) | upstream `litex,timer0` | Renode's `LiteX_Timer_CSR32` |
| pwm 0, 1 | LiteX PWM cores (`pwm0` of the board, `pwm1` added) | upstream `litex,pwm` | `renode/LiteXPWM.cs`: the registers; the bench reads the duty programmed, no waveform |
| watchdog | LiteX watchdog core | upstream `litex,watchdog` | `renode/LiteXWatchdog.cs`: counts on the virtual clock, resets the machine on expiry, counts the feeds |
| can | an SJA1000-compatible core (as the OpenCores SJA1000-compatible core on a 32-bit Wishbone bus) | upstream SJA1000 driver, this package's front end `arclang,litex-sja1000` | `renode/ArcLangSJA1000.cs`: PeliCAN registers, receive FIFO, transmit; no bit timing, no arbitration, no error confinement: a frame is sent and acknowledged at once |
| adc 0-7 | **a register bank of this target**, one 32-bit register per channel (`arclang,litex-adc`): LiteX has no ADC core; on a board it is a CSR bank fed by the converter's interface, not part of this lot | this package's `arclang,litex-adc` (Zephyr ADC API) | `renode/RegisterADC.cs`: the bench writes the raw values |
| dac | none: a binding with a `dac` is refused for this target | | |

What Renode models is the SoC's registers and the CPU's instructions; its
virtual time runs the instructions at the CPU model's rate, not the
VexRiscv's cycle timing: times measured in emulation (below) are
deterministic, not a worst-case execution time.

### The bitstream

Not built in this lot. The overlay and the Renode platform describe the SoC
(addresses, interrupt lines); a LiteX SoC for an ECP5 board with those
peripherals (two more UARTs, a second PWM, an SJA1000-compatible core, the
ADC register bank) is to be written as a LiteX target script, and Yosys,
nextpnr-ecp5 and the LiteX toolchain to be added to the image (in a stage of
its own: they are large). On a board, `zephyr.bin` is loaded by the LiteX
BIOS (`litex_term --kernel`) or written to the SPI flash.

## Processor in the loop

```text
$ tools/embedded/soc/run.sh arclang pil examples/energy/battery-charger/battery_charger.arc

PIL examples/energy/battery-charger/battery_charger.arc — zephyr/litex_vexriscv; riscv64-zephyr-elf-gcc (Zephyr SDK 1.0.1) 14.3.0; Zephyr 4.4.2; Renode v1.17.0

  ✓ pass  TC-CHG-001
    verifies SYS-CHG-001, SYS-CHG-002 · machine Charger
    model: pass · C: pass · the C reproduces the model
    timing: step() took at most 19.9 us of the target's virtual time
    back to back: 61 steps × 34 outputs, largest difference 0 (tolerance 0)
    ok   SYS-CHG-001: at(210 ms, "Charger".Charging.Precharge) == 1   model 1 · C 1
    ...

  ✓ pass  TC-CHG-002
    verifies SYS-CHG-003 · machine Charger · through io_binding Charger board
    model: pass · C: pass · the C reproduces the model
    timing: 41 cycles, one every 10000 us of the target's virtual time (the first at 30008 us)
    back to back: 41 steps × 40 outputs, largest difference 0 (tolerance 0)
    ok   at(60 ms, "Charger".FanOn) == 1   model 1 · C 1
    ...

  ✓ pass  TC-CHG-003
    verifies SYS-CHG-004, SYS-CHG-005 · machine Charger
    model: pass · C: pass · the C reproduces the model
    timing: step() took at most 24.4 us of the target's virtual time
    back to back: 61 steps × 34 outputs, largest difference 0 (tolerance 0)
    ok   SYS-CHG-004: at(100 ms, "Charger".target) == 2 A   model 2 A · C 2 A
    ...

  3 PIL verdict(s) cached in examples/energy/battery-charger/.arclang/sim-cache
```

For each case, `arclang pil` exports the application, builds it with the
package's `build` command, writes the bench's plan and runs the package's
emulator; the target's reports are compared with the model-level run as
`arclang sil` compares the host build, and judged the same way (`pass`,
`fail`, `error`).

- **Through the binding** (`binding:` in the case): the controller's loop
  (`pil_io.c`) runs on the target. The bench drives the bound input
  channels on the emulated peripherals from the same channel operations
  as SIL (a gpio line's level, an ADC register, a CAN frame into the
  SJA1000's receive FIFO, a byte into a UART), and after each cycle reads
  the bound output channels from the peripherals (the gpio output register,
  the PWM duty, the bytes each UART sent in the cycle, the last frame of a
  CAN identifier, the watchdog's feeds). The target reports its outputs
  after each cycle on the console; both are compared with the model, the
  channels as SIL compares them.
- **Without a binding**: the step() API (`pil_direct.c`). The bench sends
  each step's inputs on the console as a line, the target steps and reports.
- **Bit for bit**: values cross the console as the hexadecimal digits of
  their IEEE 754 bits (`x404b4ccccccccccd` for 54.6), so a `Real` computed
  by the target's software floating point is compared exactly.

**How steps are synchronised.** On the target's reports, not on time. The
bench runs the emulation in quanta of virtual time (100 us; a twentieth of
the period through a binding) until the report of the step arrives, then
sets the next step's inputs. Through a binding, the target waits for a `G`
on the console to start, so step 0 sees its inputs; every cycle starts on
a boundary of the period of the kernel's timer, so a step's inputs, set
within a twentieth of the period after the report of the step before, are
read by the next cycle and by no other. The emulator's virtual time does not
depend on the host: the same image and plan give the same run, on a laptop
or on CI. The report of a cycle carries its time on the target, and the
outcome says whether every cycle came one period after the one before
(`41 cycles, one every 10000 us`); without a binding, how long `step()`
took on the target.

`--test` runs one case, `--target` another target, `--keep DIR` keeps the
applications, the builds, the plans, the results and the emulator's logs,
`--json` prints the outcomes. The command fails when a case does not pass.

**Verdicts.** Each case's verdict is kept as `<case>.pil.json` in
`.arclang/sim-cache/` beside the model (kind `pil`, the target named),
beside its SIL verdict. Its key is the SIL key (every generated file, the
case) with the target's name and every file of its package. `check`,
`gate`, the tables and the documents read it: a fresh PIL failure or error
is the case's verdict (it is the same C, on the target); a fresh pass
stands for a SIL verdict never run or stale, and is added to the evidence
of a SIL verdict otherwise; a stale one is a warning only.

Run on the four examples with executed cases (`tests/embedded_soc.rs`):

| Example | Cases | Through the I/O | Largest difference |
|---|---|---|---|
| battery charger | 3 | 1 (gpio, can, uart, adc, pwm, watchdog) | 0 |
| power window | 2 | 0 | 0 |
| adaptive cruise control | 2 | 1 (gpio, adc, can in and out, pwm 1, watchdog) | 0 |
| remote start | 2 | 0 | 0 |

## In Docker

Everything builds and runs in the SoC image, never on the host:

```text
docker build -t arclang-embedded:1 tools/embedded
docker build -t arclang-embedded-soc:1 tools/embedded/soc
tools/embedded/soc/run.sh cargo test --test embedded_soc -- --include-ignored
tools/embedded/soc/run.sh cargo run -- pil examples/energy/battery-charger/battery_charger.arc
```

`tools/embedded/soc/Dockerfile` pins Zephyr v4.4.2 (the zephyr repository
only: the LiteX board needs no HAL module, picolibc comes with the
toolchain), the Zephyr SDK 1.0.1 (the RISC-V toolchain only), west in a
Python 3.12 (Zephyr 4.4's minimum), Renode 1.17.0 (portable). The CI job
"Embedded SoC (Zephyr, Renode)" builds both images and runs those tests.

## Limits, said plainly

- **Emulation, not a board.** The run is on Renode's model of the SoC. The
  PWM duty, the watchdog and the CAN controller are register-level models
  written for this target; the ADC is a register bank this target defines.
  No signal leaves the SoC: no waveform, no bit timing on the CAN bus.
- **Virtual time is not cycle timing.** Renode runs instructions at its CPU
  model's rate; the times reported are deterministic and comparable between
  runs, not the VexRiscv's worst-case execution time.
- **No bitstream** in this lot (above). The SoC described has not been
  synthesised.
- **One target ships**, `zephyr/litex_vexriscv`, and one runtime besides the
  host, Zephyr. The image has been built and the tests run on Linux arm64
  (Docker on Apple silicon); the CI job builds it on x86_64 (the x86_64
  downloads of the SDK and Renode are the same releases).
- **PIL instrumentation**: the PIL mains report on the console after each
  cycle, which the controller's `main.c` does not; the cycle itself is the
  same code.
