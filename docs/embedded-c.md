# Embedded C: from the model to any hardware

ArcLang generates the behavior of a model as C that knows no board, and
reaches the hardware through one small I/O interface. Three layers:

1. **The behavior** — `<model>.h` / `<model>.c`: the state machines of the
   model as portable C99, an `init()` / `step()` API over structures the
   caller owns. No `malloc`, no OS call, no global variable (the only
   file-scope objects are `static const` tables), no recursion, every loop
   bounded by the size of the machine. It never touches a register.
2. **The I/O interface** — `arclang_io.h`, and the bindings of the
   machine's inputs and outputs to its channels, written once in the model
   (`io_binding`), checked by the compiler and generated as glue
   (`<model>_<binding>_io.h/.c`): the only generated code that calls the I/O.
3. **The targets** — a package implements `arclang_io.h` once for a target
   (Zephyr, Linux RT, an FPGA soft core, a PLC runtime...). This first lot
   ships the host (PC) implementation only, `arclang_io_host.c`, used for
   software in the loop. Target packages come later.

```text
arclang export model.arc -f c -o build/c     # the files, for any target
arclang sil    model.arc --cc gcc --cc clang # the C against the model
```

`arclang sil` proves layers 1 and 2: it compiles the generated C on this
computer, runs every test case that `execute:`s with its input vectors,
compares the outputs with the model-level run step by step and judges the
case's `expects:` on both.

## What the generated code computes

A state machine is executed as the debugger (`arclang debug`) executes it,
by the same code (`execution::Machine`): UML and Capella semantics, nothing
invented.

- **Inputs** are the machine's *events* (the triggers of its transitions) and
  its *conditions* (the guards, as written). A guard written in prose
  (`"lead vehicle in range"`) is not code the compiler can decide: it is a
  boolean input the caller (or the binding) sets. A guard written `true` or
  `false` decides itself.
- **Outputs** are the machine's *states* (true while active) and its
  *actions* (entry, exit, do and transition actions: how many times each ran
  in the step, saturating at 255). An action written in prose
  (`"close contactors"`) is reported, not executed: the integrator, or a
  binding, turns it into an effect.
- **One step** (`step()`), in each machine:
  1. on the first step, the initial states are entered (their entry actions
     count in that step);
  2. the events present in the step fire, in the order the machine's
     transitions first name them;
  3. the timed transitions fire when their source has been active long
     enough: `timing: 200 ms` without a trigger waits
     `ceil(200 ms / period)` steps;
  4. the completion transitions (no trigger, no timing) fire, once;
  5. the `do:` activities of the active states run (count once);
  6. every active state has been active one step more.

  For each trigger, the transitions it enables are chosen at the start of
  its firing (source active, guard holding), then fired in order — inner
  source first, then lower `priority`, then declaration order — each only
  if its source is still active. Firing exits the active states under the
  source, deepest first (exit actions), runs the transition's action, then
  enters the target and its initial states (entry actions). A transition
  from a state to itself exits and re-enters it. Orthogonal top-level
  regions (`region "Name" { ... }`) fire in the same step.

The period is `<MODEL>_PERIOD_US` (microseconds), the binding's period by
default, else 10 ms; define it at compile time for another one.

## The API

For a model `Battery Charger` with a machine `Charger`:

```c
#include "battery_charger.h"

battery_charger_state   state;    /* the machines' active states and timers */
battery_charger_inputs  inputs;   /* inputs.charger.ev_plug, .cond_battery_present ... */
battery_charger_outputs outputs;  /* outputs.charger.st_charging, .act_close_contactors ... */

battery_charger_init(&state);
for (;;) {
    /* set the inputs: an event is true in the step it occurs */
    battery_charger_step(&state, &inputs, &outputs);
    /* read the outputs */
}
```

Every field carries, in a comment, the text it comes from (`/* event
"plug" ... */`). The names: `ev_<event>`, `cond_<condition>`,
`st_<state path>`, `act_<action>`, in snake case.

## The I/O interface

`arclang_io.h` holds what a controller needs and no more:

| Function | Why |
|---|---|
| `arclang_io_init` | bring the I/O up before the first cycle |
| `arclang_io_gpio_read/write` | discrete signals: switches, relays, lamps, edges as events |
| `arclang_io_adc_read` | analog sensors, compared with raw thresholds as conditions |
| `arclang_io_dac_write` | analog setpoints |
| `arclang_io_pwm_write` | actuators driven by a duty cycle (fans, valves, motors) |
| `arclang_io_can_receive/send` | the vehicle and industrial bus: frames as events, their first byte as a condition |
| `arclang_io_uart_read/write` | serial links: a byte as an event or as an effect |
| `arclang_io_clock_wait_period` | the step rate: the only call that blocks |
| `arclang_io_watchdog_kick` | the target resets when the cycle stops |

Every function returns `ARCLANG_IO_OK` or an error (`ERR_CHANNEL`,
`ERR_DEVICE`, `ERR_FULL`). Values are raw: converter counts, per mille,
bytes; the model compares them, the target scales them.

## Binding a machine to the channels

```arclang
io_binding "Charger board" {
    machine: "Charger"
    period: 10 ms
    watchdog: true
    inputs: [
        "plug <- gpio 2 rising",          // an event: an edge
        "unplug <- gpio 2 falling",
        "battery present <- gpio 3",      // a condition: a level
        "voltage reached <- can 0x210",   // an event: a frame received
        "fault <- uart 0 0x46",           // an event: a byte received
        "above limit <- adc 1 > 3000",    // a condition: a raw threshold
        "temperature <- can 0x211"
    ]
    outputs: [
        "Charging -> gpio 20",            // a state: high while active
        "FanOn -> pwm 0 600",             // 600 per mille while active
        "close contactors -> uart 1 0x43" // an action: a byte each step it runs
    ]
}
```

| Input channel | Binds | Value |
|---|---|---|
| `gpio N rising` / `falling` | an event | the edge since the last cycle (edges count from a low level) |
| `can ID` | an event | a frame with this identifier received since the last cycle |
| `uart N BYTE` | an event | this byte among those received in the period (at most 16 read) |
| `gpio N` | a condition | the level |
| `adc N > RAW` / `< RAW` | a condition | the raw value against the threshold |
| `can ID` | a condition | the first byte of the last frame not zero (held between frames) |

| Output channel | Writes |
|---|---|
| `gpio N` | high while the value is not zero |
| `pwm N PERMILLE` | the duty while not zero, else 0 |
| `dac N RAW` | the raw value while not zero, else 0 |
| `can ID` | a frame every period, its first byte the value |
| `uart N BYTE` | the byte each step the value is not zero |

The compiler refuses, with the reason: an input or output that names no
event, condition, state or action of the machine (it lists those there
are); an event bound to a level or an analog value; a condition bound to an
edge or a UART byte; an input bound twice; a channel written twice; a pin
both read and written; an `adc` threshold that can never hold; a CAN
identifier above 29 bits, a byte above 255, a duty above 1000; a period that
is not a whole number of microseconds; an unknown machine. An event or a
condition left unbound is a warning: on the target it never occurs, never
holds.

The glue (`<model>_<binding>_io.c`) gives:

```c
void              <b>_glue_init(<b>_glue *glue);
arclang_io_status <b>_read (<b>_glue *glue, <model>_inputs *inputs);
arclang_io_status <b>_write(const <model>_outputs *outputs);
arclang_io_status <b>_cycle(<b>_glue *glue, <model>_state *state,
                            <model>_inputs *inputs, <model>_outputs *outputs);
```

`cycle` reads; if a read failed it returns the error without stepping (the
application decides what a failed read means); it steps, writes, and kicks
the watchdog only when everything succeeded. A channel in error leaves its
events and conditions false. A main loop on a target:

```c
arclang_io_init();
battery_charger_init(&state);
battery_charger_charger_board_glue_init(&glue);
for (;;) {
    if (battery_charger_charger_board_cycle(&glue, &state, &inputs, &outputs) != ARCLANG_IO_OK) {
        /* the application's reaction: count, degrade, stop kicking... */
    }
    (void)arclang_io_clock_wait_period(BATTERY_CHARGER_CHARGER_BOARD_PERIOD_US);
}
```

The glue refuses to compile (`#error`) when the behavior was compiled for
another period than the binding's.

## Test cases executed at fixed step

```arclang
test_case "TC-CHG-001" {
    verifies: ["SYS-CHG-001", "SYS-CHG-002"]
    execute: {
        machine: "Charger"
        step: 10 ms
        stop_time: 600 ms
        inputs: ["0 s: battery present", "20 ms: plug", "400 ms: voltage reached", "500 ms: unplug"]
    }
    expects: [
        "SYS-CHG-001: at(210 ms, \"Charger\".Charging.Precharge) == 1",
        "SYS-CHG-002: final(\"Charger\".Idle) == 1",
        "at(20 ms, \"Charger\".action.close_contactors) == 1"
    ]
}
```

- `inputs`: `"<time>: <event>"` — the event occurs in the step at that
  time; `"<time>: <condition>"`, `"<condition> = false"` — it holds, or not,
  from that step on (conditions start false). Every time is on the step
  grid.
- `expects`: the comparisons of simulated cases (`min`, `max`, `final`,
  `initial`, `mean`, `at(t, ...)`), over the run's variables: a state reads
  `"Machine".State` (`"Machine".Parent.Child` for a nested one; 1 while
  active), an action `"Machine".action.<its text, other characters _>`.
- `binding: "Charger board"`: the run goes through the I/O. The vectors
  become pin levels, converter values, CAN frames and UART bytes on the host
  board; the glue reads them; what the glue writes on the bound output
  channels is compared too. When the I/O cannot produce the vectors (two
  rising edges of one pin in consecutive steps; a value no threshold
  gives), the run says so and is in error.
- `tolerance`: how far a C value may be from the model's (default 0: the
  outputs are discrete and must be equal).

Compiling the model runs each case on the model and warns of a failing one
(`verification: test_case 'X' fails on the model: ...`); it also refuses a
case naming a machine, an input, a binding or a variable that does not exist.

## Software in the loop

```text
$ arclang sil examples/energy/battery-charger/battery_charger.arc --cc gcc-15 --cc clang

SIL examples/energy/battery-charger/battery_charger.arc — gcc-15 (Homebrew GCC 15.2.0)

  ✓ pass  TC-CHG-001
    verifies SYS-CHG-001, SYS-CHG-002 · machine Charger
    model: pass · C: pass · the C reproduces the model
    back to back: 61 steps × 18 outputs, largest difference 0 (tolerance 0)
    ok   SYS-CHG-001: at(210 ms, "Charger".Charging.Precharge) == 1   model 1 · C 1
    ...
```

Each case is compiled with `-std=c99 -Wall -Wextra -Werror -pedantic`, run
with the vectors, and judged:

- `pass`: the C reproduces the model (every output equal at every step,
  within `tolerance`, and every expectation judged alike) and every
  expectation holds;
- `fail`: an expectation fails (on the C as on the model: the model is
  wrong, or the requirement), or the C departs from the model (the first
  departing value is shown);
- `error`: the C did not compile or run, or the vectors cannot go through
  the I/O.

`--test` runs one case, `--keep DIR` keeps the generated files, the
harnesses and the binaries, `--json` prints the outcomes. The command fails
when a case does not pass.

## What is not generated yet

Said plainly:

- **Guards and actions are not code.** ArcLang writes guards and actions as
  text; the generator makes the guards inputs and counts the actions. There
  is no expression language for them yet (no variables, no arithmetic, no
  assignment in an action), so no data flow is computed by the C.
- **No continuous or discrete-time dynamics.** The language describes no
  equations, filters or controllers (the Modelica export writes the
  structure, the physics is hand-written); nothing of the kind is generated.
  Only state machines are.
- **No functional chains, scenarios or functions** are generated as code:
  the debugger executes them, the generator does not.
- **Timing with a trigger** (`trigger: "LF-EXEC" timing: 1500 ms`): the
  trigger fires the transition; the timing is a budget, not a timer, and is
  not generated (a note in the generated file says so). Only `timing:`
  without a trigger is a timer.
- **Transitions between orthogonal regions** are refused (each region is
  generated as a machine of its own). Regions nested inside a state do not
  exist in the language: a composite state holds one region.
- **History states, choice, junction, fork and join pseudo-states,
  deferred events and change events** are not in the executor, so not in the
  C.
- **One event at a time per trigger:** an event present in a step fires
  once; the order of the events within a step is the order of the machine,
  not the order they happened in.
- **Verdicts in the traceability:** a failing executed case is a
  compilation warning; the verdicts of `arclang sil` are printed (and
  `--json`), not yet cached for `check`, `gate` and the tables as
  `arclang verify` caches simulation verdicts, so an executed case reads
  pending there.
- **Target packages** (Zephyr, Linux RT, FPGA, PLC) are not part of this
  lot: only the host implementation of `arclang_io.h` exists. The FMU
  wrapping of the C is not done.
- **No certification claim.** The generated code is written to be simple
  to review (static tables, bounded loops, no recursion, no dynamic memory)
  and is compiled warning-free under the strict flags above; it has not been
  checked against MISRA C, and nothing here is qualified for DO-178C or
  ISO 26262.
