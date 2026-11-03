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

- **Inputs** are the machine's *events* (the triggers of its transitions),
  its *conditions* (the guards written in prose) and its `in` *variables*. A
  guard written in prose (`"lead vehicle in range"`) is not code the
  compiler can decide: it is a boolean input the caller (or the binding)
  sets. A guard written `true` or `false` decides itself; a guard written as
  an expression over the variables (`"speed > 30 km/h"`) is computed (see
  [the action language](#the-action-language-variables-expressions-equations)).
- **Outputs** are the machine's *states* (true while active), its
  *actions* (entry, exit, do and transition actions: how many times each ran
  in the step, saturating at 255) and its *variables* (their value after the
  step). An action written in prose (`"close contactors"`) is reported, not
  executed: the integrator, or a binding, turns it into an effect. An action
  written as assignments (`"count := count + 1"`) is executed, and counted.
- **One step** (`step()`), in each machine:
  1. on the first step, the initial states are entered (their entry actions
     run in that step);
  2. the equations are evaluated, in the order written;
  3. the events of the step fire one after the other, in the order they
     arrived (see [the order of events](#the-order-of-events));
  4. the timed transitions fire when their source has been active long
     enough: `after: 200 ms` waits `ceil(200 ms / period)` steps (see
     [timers and budgets](#timers-and-budgets));
  5. the completion transitions (no trigger, no timing) fire, once;
  6. the `do:` activities of the active states run (count once), in the
     order of the states;
  7. every active state has been active one step more.

  For each trigger, the transitions it enables are chosen at the start of
  its firing (sources active, guards holding), then fired in order — inner
  source first, then lower `priority`, then declaration order — each only
  if its sources are still active. Firing exits the active states of the
  transition's scope, the last state of the scope first (so the deepest
  first; exit actions), runs the transitions' actions, then enters the
  targets, their ancestors under the scope and the initial state of every
  other region entered (entry actions, outermost first, regions in the
  order written). Orthogonal regions fire in the same step. See
  [regions and pseudo-states](#regions-and-pseudo-states) for the scope of
  a transition.

The period is `<MODEL>_PERIOD_US` (microseconds), the binding's period by
default, else 10 ms; define it at compile time for another one.

## The action language: variables, expressions, equations

A machine may declare typed variables and difference equations; a guard
can then be an expression and an action assignments. The syntax follows
SysML v2 (`in`/`out` attributes, `:=` assignment, `and`/`or`/`not`), written
in ArcLang's attribute style:

```arclang
state_machine "Charger" {
    variables: [
        "in voltage: Real [V]",                 // set by the caller every step
        "in current: Real [A]",
        "const v_absorption: Real [V] = 54.6 V",
        "target: Real [A] = 0 A",               // kept by the machine
        "error: Real [A]",
        "out setpoint: Real [A]",
        "filtered: Real [V] = 0 V",
        "out charge: Real [Ah]",
        "out plugs: Integer = 0"
    ]
    equations: [
        "error = target - current",
        "setpoint = pid(error, 0.5, 20 Hz, 0 s, 0 A, 20 A)",
        "filtered = filtered + 0.2 * (voltage - filtered)",
        "charge = integrator(current, 1)"
    ]
    ...
        state Charging {
            entry: ["close contactors", "plugs := plugs + 1"]
            state Bulk { entry: ["ramp current", "target := 16 A"] }
            transition Bulk -> Absorption { guard: "filtered >= v_absorption and target > 0 A" }
        }
}
```

**Variables**: `[in|out|const] name: Type [unit] = initial`.

| Type | C type | Arithmetic |
|---|---|---|
| `Boolean` | `bool` | `and`, `or`, `not` (or `&&`, `\|\|`, `!`), `==`, `!=` |
| `Integer` | `int32_t` | saturating: `+ - * /` and negation saturate to the range of `int32_t` instead of overflowing; a division by zero gives 0 (C truncation toward zero otherwise) |
| `Real` | `double` | IEEE 754 binary64 |
| `Float` | `float` | IEEE 754 binary32 |

No fixed point is generated: the type of each variable is the one declared.
The unit (a symbol of the unit table: `V`, `km/h`, `Ah`, `m/s`...) fixes the
dimension and the unit the value is *stored* in, in the C as in the result
columns (`speed` declared `[km/h]` holds km/h). A unit with an offset (`°C`)
is refused: write temperatures in `K`. The initial value is zero (false)
when omitted. `in`: an input, in the `inputs` structure, set by the caller
every step, never assigned by the machine. `const`: never assigned. Every
other variable is kept in the `state` structure, and every variable is in
the `outputs` structure after the step.

**Expressions**: `+ - * /`, `< <= > >= == !=`, `and or not`, parentheses,
numbers with a unit (`30 km/h`, `2 A`), `true`, `false`, and the pure
functions `min(a, b)`, `max(a, b)`, `abs(a)`, `saturate(x, lo, hi)` and
`gain(x, k)` (= `x * k`). The compiler types every operation and checks the
units with the unit system of the language:

- `+`, `-`, comparisons, `min`, `max`, `saturate` take operands of one
  dimension (a plain `0` goes with any); `*` and `/` multiply and divide
  dimensions (`2 N / 1 m` is a stiffness);
- values of different units of one dimension are converted where they
  meet, the right side to the unit of the left (a number is folded at
  compile time: `speed > 30 km/h` with `speed [m/s]` compares with
  `8.333...`); a conversion of an `Integer` by a factor that is not whole
  makes it a `Real`;
- a number written in the model takes the type of the other side when it is
  a value of it (`0.5` beside a `Float` is `0.5f`); otherwise `Integer`
  meets `Float` as `Float`, anything meets `Real` as `Real`;
- an assignment converts to the target's unit and type: an `Integer` takes
  only an `Integer` (no silent truncation), a `Float` takes a `Real`
  (rounded to nearest).

The C prints the same typed tree as the model evaluates, operator for
operator, so the two compute the same bits: the integer operations are
static helpers (`arc_add_i32`...), `min`/`max`/`abs`/`saturate` are static
functions (each argument evaluated once), and the source turns
floating-point contraction off (`#pragma STDC FP_CONTRACT OFF` under Clang;
GCC does not contract in ISO C mode).

**Guards and actions as code**: a guard is an expression when it parses as
one and names a variable of the machine; it must be a `Boolean`. An action
is code when it is assignments `target := expression` (or `=`), separated by
`;`. A text that names no variable stays prose, exactly as before: old
models keep their meaning. A text that names a variable and also a name that
is none is an error (a typo), not prose. Actions run when the semantics runs
them: exit actions, then the transition's action, then entry actions; a
guard is evaluated when its trigger is processed, before any transition of
that trigger fires.

**Equations** are `target = expression`, evaluated every step in the order
written, after the initial states are entered and before the transitions;
an equation reads the values of the step so far (a variable assigned later
in the list, or by an action, reads its value of the previous step: a
sequential program, no algebraic loop). One equation per variable; an
action does not assign a variable an equation computes. In equations only,
the blocks of discrete-time data flow, each call a block with its own
state:

| Block | Output and update (step `T` = the period) |
|---|---|
| `delay(x)`, `delay(x, initial)` | the input of the previous step (the initial value on the first) |
| `integrator(x, k)` | forward Euler: `y = s; s := s + k*T*x` (unit: x·k·time) |
| `integrator(x, k, lo, hi)` | the same, output and state held in `[lo, hi]` |
| `pid(e, kp, ki, kd)` | parallel: `kp*e + I + kd*(e - e_prev)/T`, `I := I + ki*T*e`; no derivative on the first step |
| `pid(e, kp, ki, kd, lo, hi)` | the same, output held in `[lo, hi]`, clamping anti-windup (no integration further into the saturation) |

The gains carry their units: `kp*e`, `ki*e*time` and `kd*e/time` must be
of one dimension (the output's). Gain and sum are plain `*` and `+`. A block
cannot sit in the arguments of another (give the inner one its own
equation), so C's unspecified order of evaluation of arguments changes
nothing. Their arguments are converted to SI units, their output (a `Real`)
is in SI units of its dimension, converted on assignment.

The compiler's messages say what is wrong and where:

```text
state_machine 'M': guard "speed > 3 s": '>' between a speed and a time
state_machine 'M': action "n := speed": a Real assigned to Integer 'n' (an Integer holds whole numbers: declare it Real, or compute in Integers)
state_machine 'M': guard "speed > limt": 'limt' is no variable of the machine
state_machine 'M': equation "y = pid(e, 2 N / 1 m, 1 N / 1 m, 0 s)": pid(...): ki*e*time is a quantity in kg·s^-1, kp*e a force
```

In an executed case, an `in` variable takes a value from a step on
(`"100 ms: current = 1.5 A"`, in any unit of its dimension), and an
expectation reads a variable as `"Charger".setpoint`, in its unit:

```arclang
expects: ["at(100 ms, \"Charger\".target) == 2 A", "max(\"Charger\".setpoint) <= 20 A"]
```

`arclang sil` compares every variable, step by step, with the model's,
bit for bit by default (`tolerance: 0`).

## Regions and pseudo-states

A state may hold several orthogonal regions, written as `region` blocks
inside it (as the machine itself may), each with its initial state:

```arclang
state Active {
    region "Motion" {
        initial: "Stopped"
        state Stopped {}
        state Moving { initial: "Up" state Up {} state Down {} }
        state Resume { kind: "deep_history" }
        state Decide { kind: "choice" }
        transition Stopped -> Decide { trigger: "auto" }
        transition Decide -> Up { guard: "position < 50" }
        transition Decide -> Down { guard: "else" }
    }
    region "Safety" {
        initial: "Watching"
        state Watching {}
        state Pinched {}
        state Monitor { kind: "shallow_history" }
        transition Watching -> Pinched { trigger: "obstacle" }
    }
    transition Pinched -> Stopped { trigger: "release" }   // across the regions
}
```

Pseudo-states are written as the Capella import writes them: a state with
a `kind:` (Capella's `ChoicePseudoState`, `ForkPseudoState`...). They
are never active, so no output reports them. The executor, the C and the
SIL run them alike (`examples/automotive/power-window/power_window.arc`):

| `kind:` | Semantics (UML / Capella) |
|---|---|
| `junction` | static: its branches are chosen before the transition fires, with the values of that moment; the transition is enabled only when a branch holds |
| `choice` | dynamic: its branch is chosen when it is reached, after the exits and actions before it (it sees what they assigned); the transition is enabled only when a branch holds then; if the actions made none hold, the first branch |
| `fork` | one transition to several states of orthogonal regions; the regions not named enter their initial state |
| `join` | fires when every source is active and every incoming transition's guard holds, on their common trigger; all their actions run, then the outgoing one's |
| `shallow_history` | entering it enters the state its region was in when its owner was last exited (by its initial states); else its default transition, else the region's initial state |
| `deep_history` | the same with every state that was active in the region, down to the leaves |
| `entry_point`, `exit_point` | pass through: the transition goes on along their outgoing transition |
| `terminate` | the machine stops: nothing is active, no exit action runs, no transition fires any more |

Branches of a junction or a choice are tried in `priority` then
declaration order; a branch guarded `else` is tried last and always holds.
A `final` state is a state (it has no outgoing transition).

The scope of a transition (what it exits and what it re-enters) is the
region that holds both ends, as UML defines it:

- in one region of their common ancestor: the active state of that region
  is exited; the target is entered with its ancestors under it;
- a target inside its source (local): the source stays active, only the
  region holding the target changes;
- a source inside its target, or a state to itself: the target is exited
  and entered again, by its initial states;
- between orthogonal regions: the state holding both regions is exited and
  entered again (the whole machine for top-level regions), the target in
  its region, the other regions by their initial state.

A pseudo-state used against its meaning is refused with the reason, and no
C is generated for the machine: a join whose incoming transitions have
different triggers or with more than one outgoing transition, a fork whose
targets share a region, a junction or a choice without an outgoing
transition, transitions that go round through pseudo-states.

In the C, a history is a field of the machine's state (`hist_<n>` for a
shallow one, `deep_<n>[]` for a deep one), kept when its owner is exited;
exits run over a range of the state table, last state first; every other
entry and exit is a list computed once from the executor.

Capella has every one of these pseudo-states but the junction: a junction
is exported as a Capella choice (its branches are then chosen when
reached). SysML v2 export writes the states of every region of a state
in one region, as it does for the machine's top-level regions.

## Timers and budgets

A time on a transition is one of two things, said explicitly:

```arclang
transition Arming -> Armed { after: 30 ms }                        // a timer: it fires
transition Done -> Idle { trigger: "after 20 ms" }                 // the same (SysML v2 `accept after`)
transition Armed -> Done { trigger: "confirm" budget: 50 ms }      // a budget: it is watched
```

- **A timer** (`after:`, or `trigger: "after <time>"`) is the trigger of the
  transition: it fires when its source has been active `ceil(time /
  period)` steps (guards holding). A transition has one trigger: an event
  and `after:` together are refused.
- **A budget** (`budget:`) is how long the source may stay active before
  the transition fires. It is never a timer: nothing fires because of it.
  It is watched: the machine reports, for each budget, an output true in
  every step its source has been active longer than `ceil(budget /
  period)` steps (in the C, `bool bud_<transition>`; in a case,
  `"Machine".budget.<transition>`, `Armed -> Done` written `Armed_Done`), so
  an expectation can require `max("W".budget.Armed_Done) == 0`.
- `timing:` keeps its meaning: without a trigger a timer, beside a trigger
  a budget (now generated and watched).

## The API

For a model `Battery Charger` with a machine `Charger`:

```c
#include "battery_charger.h"

battery_charger_state   state;    /* the machines' active states and timers */
battery_charger_inputs  inputs;   /* inputs.charger.queue, .cond_battery_present ... */
battery_charger_outputs outputs;  /* outputs.charger.st_charging, .act_close_contactors ... */

battery_charger_init(&state);
for (;;) {
    inputs.charger.queued = 0U;   /* the events of this step, in arrival order: */
    (void)battery_charger_charger_post(&inputs.charger, BATTERY_CHARGER_CHARGER_EV_PLUG);
    inputs.charger.cond_battery_present = true;
    battery_charger_step(&state, &inputs, &outputs);
    /* read the outputs */
}
```

Every field carries, in a comment, the text it comes from (`/* event
"plug" ... */`). The names: `<MODEL>_<MACHINE>_EV_<EVENT>` (an event's
code), `cond_<condition>`, `v_<variable>`, `st_<state path>`,
`act_<action>`, in snake case.

## The order of events

The events of a machine reach a step through a queue: `queue[]` and
`queued` in its inputs, filled by `<model>_<machine>_post()` in the order the
events arrived (false, and the event dropped, when `<MODEL>_<MACHINE>_QUEUE_SIZE`
events, 16 by default, are already queued). One step processes them in that
order, each as a run to completion: its transitions are chosen (sources
active, guards holding) and fired before the next event is looked at. An
event posted twice fires twice. Then come the timed transitions and the
completion transitions, as before. The caller empties the queue
(`queued = 0`) before posting the events of the next step.

In an executed case, an event may arrive at any time (`"13 ms: plug"`): the
first step at or after it takes it, and the events of one step are ordered
by their time, then as the inputs list them. A condition or a variable
changes on the step grid only.

Through an `io_binding`, the glue posts the events in the order it reads
them, which is all the I/O tells of their order within a period: the edges
of the gpio pins (by pin), then the CAN frames (by identifier), then the
UART bytes (by port, in the order they were received: two bytes are two
events). A case whose order of events in a step the I/O cannot give (a UART
byte before a gpio edge of the same step, an edge twice in a step) is in
error and says so.

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
the watchdog only when everything succeeded. A channel in error posts no
event and leaves its conditions false; an event the queue has no room for
makes the read fail with `ARCLANG_IO_ERR_FULL`. A main loop on a target:

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

- `inputs`: `"<time>: <event>"` — the event arrives at that time, taken by
  the first step at or after it, in order of arrival; `"<time>:
  <condition>"`, `"<condition> = false"` — it holds, or not, from that step
  on (conditions start false); `"<time>: <variable> = <value>"` — an `in`
  variable. A condition's or a variable's time is on the step grid.
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

The verdict of each case run is kept in `.arclang/sim-cache/` beside the
model, as `arclang verify` keeps simulation verdicts, keyed by a hash of
every generated file, the case's `execute:` and its expectations (over all
the compilers of the run: an error with one is an error, a failure a
failure). `check`, `gate`, the tables and the documents read it: a fresh
verdict is the case's verdict (by requirement when its expectations name
requirements), with its values as evidence; a case never run in SIL, or
whose C or case changed since (stale, with a warning), is inconclusive and
says what to run.

## MISRA C 2012

`tools/embedded/misra.sh` (run in the embedded image: `tools/embedded/run.sh
tools/embedded/misra.sh`) exports the C of every example that has a state
machine and runs cppcheck 2.10's MISRA C 2012 addon on it (`--addon=misra
--std=c99 --enable=style`; the host implementation of `arclang_io.h`, used
only for SIL, is not checked). `tests/embedded_c.rs` keeps four examples at
the deviations below.

Over the 18 examples:

| Rule | Before | After | What changed |
|---|---|---|---|
| 2.5 (advisory) unused macro | 42 | 26 | `NO_STATE` only where a history uses it; the rest is a deviation |
| 5.2 identifiers distinct (63 chars) | 5 | 0 | identifiers bounded: a long one keeps its start and ends with a hash |
| 5.4 macro identifiers distinct | 25 | 0 | the same, for the event codes |
| 8.7 (advisory) external linkage used in one unit | 4 | 4 | deviation |
| 8.9 (advisory) object at block scope | 2 | 0 | the history tables inside the one function that reads them |
| 9.3 partially initialised array | 6 | 0 | CAN buffers initialised element by element |
| 10.3 assignment of another essential type | 2 | 0 | `INT32_MIN`/`INT32_MAX` cast to `int32_t` |
| 10.8 composite expression cast | 15 | 0 | no cast of the tick count of timers and budgets |
| 12.1 implicit precedence | 3 | 0 | the integrator and PID arithmetic parenthesised (same evaluation order) |
| 15.5 single point of exit | 5 | 0 | `run()` of a terminating machine and the glue's `cycle()` |
| **Total** | **109** | **30** | |

The deviations that remain, with their reason:

- **Rule 2.5** (advisory), `<MODEL>_<MACHINE>_STATE_COUNT` and the
  binding's `<BINDING>_PERIOD_US`: they are the API of the generated header
  for the integrator (sizing a log of the states; the period of the main
  loop); the generated code itself has no use for them.
- **Rule 8.7** (advisory), `<binding>_read()` and `<binding>_write()`: the
  glue offers them to an application that reads and writes around its own
  step (instead of `<binding>_cycle()`), so they keep external linkage
  although the glue calls them only from its own unit.

The addon checks the rules it implements without their texts; rules it
does not decide (the directives, and the rules needing a whole program or
a human reading) are not covered by these numbers.

## What is not generated yet

Said plainly:

- **No fixed point, no continuous time.** Variables are `bool`,
  `int32_t`, `float` or `double` as declared; the data flow is discrete
  (difference equations and the blocks above, at the step period).
  Continuous dynamics stay in the Modelica export. Variables are not bound
  to I/O channels yet (an `io_binding` binds events, conditions, states and
  actions): a case that sets a variable runs without `binding:`.
- **No functional chains, scenarios or functions** are generated as code:
  the debugger executes them, the generator does not.
- **Deferred events and change events** are not in the executor, so not in
  the C. A completion transition leaving a composite state does not wait
  for its regions to reach their final states: it fires as soon as its
  source is active.
- **Target packages** (Zephyr, Linux RT, FPGA, PLC) are not part of this
  lot: only the host implementation of `arclang_io.h` exists. The FMU
  wrapping of the C is not done.
- **No certification claim.** The generated code is written to be simple
  to review (static tables, bounded loops, no recursion, no dynamic memory),
  is compiled warning-free under the strict flags above and is checked with
  cppcheck's MISRA C 2012 addon (below); nothing here is qualified for
  DO-178C or ISO 26262, and an addon run is not a MISRA compliance claim.
