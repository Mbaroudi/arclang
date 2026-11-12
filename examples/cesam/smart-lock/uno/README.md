# The smart lock on an Arduino Uno

The CESAM smart lock of `../smart_lock.arc`, its three state machines
generated as C by ArcLang and running on a real microcontroller: an
Arduino Uno R3 (ATmega328P, 2 KB of RAM). Two sketches, one model:

| Directory | What the Uno runs | Driven by |
|---|---|---|
| `uno_hil/` | the generated C and the direct harness of `arclang export -f c`, behind the serial port | `arclang hil`: the executed case of `smart_lock_bench.arc`, sent step by step, judged as SIL judges the C |
| `uno_live/` | the generated C stepped every 10 ms, conditions and events taken from the serial line, every state change reported with a microsecond timestamp | `bench.py`: the scenario *Resident comes home* replayed, the response times measured, on a PC or on a UDOO KEY |

`smart_lock_bench.arc` imports the lock and adds `TC-7`, the case executed
at fixed step; the lock's own model keeps its declared verdicts and its
gate, and the evidence of the runs lands beside the bench model in
`.arclang/sim-cache/`.

## The evidence: `arclang hil`

```text
arduino-cli core install arduino:avr                     # once
examples/cesam/smart-lock/uno/flash.sh                   # export, build uno_hil, flash
arclang hil examples/cesam/smart-lock/uno/smart_lock_bench.arc --port /dev/cu.usbmodem21101
arclang table examples/cesam/smart-lock/uno/smart_lock_bench.arc --kind requirements_coverage -f md
```

The lock's three machines take 4.7 KB of flash and 768 bytes of RAM; the
31 steps of `TC-7` are answered in well under a second. The verdict is the
**board** stage of the realization ladder of `FUN-1` and `TEC-1`
(`arclang check` prints it), beside the SIL and PIL stages.

## The demonstration: `bench.py`

```text
arduino-cli compile --fqbn arduino:avr:uno uno_live && arduino-cli upload -p <port> --fqbn arduino:avr:uno uno_live
python3 bench.py /dev/cu.usbmodem21101
```

prints, for each step of the scenario, the time the bench measures (round
trip, the USB line included) and the time the board clocks between the
command and the state it reports, against the 25 ms budget of the step:

```text
  C commissioned 1           -> lifecycle Service: bench   11.3 ms, target  5.98 ms
  E SF-1                     -> modes Unlocked: bench   13.1 ms, target  7.62 ms  OK (budget 25 ms)
  E SF-2                     -> modes Locked: bench    5.0 ms, target  0.66 ms  OK (budget 25 ms)
  E SF-1 (no credential)     -> stays Locked: correct
```

`uno_live/` needs the generated `smart_door_lock.c` and `.h` next to the
sketch: `arclang export ../smart_lock.arc -f c -o <dir>` and copy them
(`flash.sh` does it for `uno_hil/`).

### On the UDOO KEY

The same `bench.py` runs on the UDOO KEY's ESP32 under MicroPython, the
Uno wired to its UART2 on the UEXT header P3:

| UDOO KEY P3 | ESP32 | Arduino Uno |
|---|---|---|
| pin 3, `EXT_UART2_TX` | GPIO13 | pin 0, RX |
| pin 4, `EXT_UART2_RX` | GPIO26 | pin 1, TX, through a 5 V to 3.3 V divider (1 kΩ / 2 kΩ) |
| pin 2, `GND` | | GND |

The ESP32's pins are 3.3 V: the Uno's TX must be divided. Copy `bench.py`
to the board's filesystem (any MicroPython tool, or paste it in the REPL)
and run `import bench; bench.main()`. Nothing must hold the Uno's USB serial
port open meanwhile: it is the same UART.

## What is on the board and what is not

The harness checks the model the board names, not the bytes it runs:
flashing is the responsibility of whoever runs `arclang hil`. A case with a
plant or through an `io_binding` is not run on a board yet (the channels
are not wired to pins): run it in SIL or PIL. A model with Real variables
needs the float `printf` on AVR (`-Wl,-u,vfprintf -lprintf_flt`).
